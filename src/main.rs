#![allow(dead_code)]
mod app;
mod config;
mod confluence;
mod event;
mod gerrit;
mod jira;
mod storage;
mod ui;

use anyhow::Result;
use app::{App, GroupStatus};
use config::Config;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::{apply_search_event, handle_key, record_history, HandleResult, SearchEvent};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, sync::Arc, time::Duration};
use tokio::{
    sync::mpsc,
    time::{interval, sleep, Instant},
};

#[tokio::main]
async fn main() -> Result<()> {
    // ── Load config ────────────────────────────────────────────────────
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("設定エラー:\n{e}");
            std::process::exit(1);
        }
    };
    let config = Arc::new(config);

    // ── Load stored data ───────────────────────────────────────────────
    let storage_dir = storage::config_dir()?;
    let mut app = App::new();

    app.history = storage::history::load(&storage_dir).unwrap_or_else(|e| {
        app.pending_storage_warning = Some(format!("history.json 読込失敗: {e}"));
        Vec::new()
    });
    app.bookmarks = storage::bookmarks::load(&storage_dir).unwrap_or_else(|e| {
        app.pending_storage_warning = Some(format!("bookmarks.json 読込失敗: {e}"));
        Vec::new()
    });

    // Restore tabs from saved sessions
    if let Ok(sessions) = storage::tabs::load(&storage_dir) {
        if !sessions.is_empty() {
            app.tabs.clear();
            for session in sessions {
                let mut tab = app::Tab::new(session.id);
                tab.name = session.name;
                tab.search = session.search;
                tab.filter = session.filter;
                app.tabs.push(tab);
            }
        }
    }

    // Prefill available options from config defaults
    app.options.spaces = config.confluence_default_spaces.clone();
    app.options.projects = config.jira_default_projects.clone();
    app.options.statuses = config.jira_default_statuses.clone();
    app.options.repos = config.gerrit_default_repos.clone();

    // ── Terminal setup ─────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, app, config, storage_dir).await;

    // ── Teardown ───────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    config: Arc<Config>,
    storage_dir: std::path::PathBuf,
) -> Result<()> {
    // Channels
    let (search_tx, mut search_rx) = mpsc::channel::<SearchEvent>(32);

    // Timers
    let mut anim_tick = interval(Duration::from_millis(150));
    let mut status_tick = interval(Duration::from_millis(200));
    // Debounce: we track a future Option<Instant> for when debounce fires
    let mut debounce_deadline: Option<Instant> = None;

    // Key event stream
    let mut key_stream = EventStream::new();

    // Show storage warning if any
    if let Some(warn) = app.pending_storage_warning.take() {
        app.status_msg.set(warn, 15);
    }

    loop {
        // ── Render ───────────────────────────────────────────────────
        terminal.draw(|f| ui::render(f, &app))?;

        // Compute debounce sleep future
        let debounce_future = async {
            if let Some(deadline) = debounce_deadline {
                let now = Instant::now();
                if deadline > now {
                    sleep(deadline - now).await;
                } else {
                    sleep(Duration::from_millis(0)).await;
                }
            } else {
                // Never fires
                sleep(Duration::from_secs(3600)).await;
            }
        };

        tokio::select! {
            // Key events
            maybe_event = key_stream.next() => {
                match maybe_event {
                    Some(Ok(crossterm::event::Event::Key(key))) => {
                        let result = handle_key(&mut app, key, &config, &storage_dir);
                        match result {
                            HandleResult::TriggerSearch => {
                                debounce_deadline = None;
                                trigger_search(&mut app, Arc::clone(&config), search_tx.clone(), &storage_dir).await;
                            }
                            HandleResult::DebounceSearch => {
                                debounce_deadline = Some(Instant::now() + Duration::from_millis(300));
                            }
                            HandleResult::None => {}
                        }
                        if app.should_quit {
                            break;
                        }
                    }
                    Some(Err(_)) | None => break,
                    _ => {}
                }
            }

            // Search results
            Some(ev) = search_rx.recv() => {
                apply_search_event(&mut app, ev, &storage_dir);
            }

            // Animation tick (150ms)
            _ = anim_tick.tick() => {
                app.travolta.tick();
            }

            // Status message tick (200ms)
            _ = status_tick.tick() => {
                app.status_msg.tick();
            }

            // Debounce fired
            _ = debounce_future => {
                if debounce_deadline.is_some() {
                    debounce_deadline = None;
                    if app.active_tab().search.has_tags() {
                        trigger_search(&mut app, Arc::clone(&config), search_tx.clone(), &storage_dir).await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn trigger_search(
    app: &mut App,
    config: Arc<Config>,
    tx: mpsc::Sender<SearchEvent>,
    storage_dir: &std::path::Path,
) {
    let tab = app.active_tab_mut();
    let tags = tab.search.tags.clone();
    let logic = tab.search.logic.clone();
    let filter = tab.filter.clone();

    // Reset group statuses
    for group in &mut tab.groups {
        group.results.clear();
        group.status = GroupStatus::Loading;
    }
    tab.selected_index = 0;
    tab.preview_scroll = 0;

    // Start animation
    app.travolta.start();

    // Record history
    record_history(app, storage_dir);

    // Confluence
    if filter.confluence_enabled && !tags.is_empty() {
        let cfg = Arc::clone(&config);
        let tx2 = tx.clone();
        let tags2 = tags.clone();
        let logic2 = logic.clone();
        let spaces = filter.spaces.clone();
        tokio::spawn(async move {
            let client = confluence::client::ConfluenceClient::new(
                cfg.confluence_url.clone(),
                cfg.confluence_user.clone(),
                cfg.confluence_token.clone(),
            );
            let result = confluence::search::search(&client, &tags2, &logic2, &spaces).await;
            let _ = tx2.send(SearchEvent::ConfluenceResult(result)).await;
        });
    } else {
        let _ = tx.send(SearchEvent::ConfluenceResult(Ok(vec![]))).await;
    }

    // Jira
    if filter.jira_enabled && !tags.is_empty() {
        let cfg = Arc::clone(&config);
        let tx2 = tx.clone();
        let tags2 = tags.clone();
        let logic2 = logic.clone();
        let projects = filter.projects.clone();
        let statuses = filter.statuses.clone();
        tokio::spawn(async move {
            let client = jira::client::JiraClient::new(
                cfg.jira_url.clone(),
                cfg.jira_user.clone(),
                cfg.jira_token.clone(),
            );
            let result = jira::search::search(&client, &tags2, &logic2, &projects, &statuses).await;
            let _ = tx2.send(SearchEvent::JiraResult(result)).await;
        });
    } else {
        let _ = tx.send(SearchEvent::JiraResult(Ok(vec![]))).await;
    }

    // Gerrit
    if filter.gerrit_enabled && !tags.is_empty() {
        let cfg = Arc::clone(&config);
        let tx2 = tx.clone();
        let tags2 = tags.clone();
        let logic2 = logic.clone();
        let repos = filter.repos.clone();
        let commits = filter.gerrit_commits;
        let tag_search = filter.gerrit_tags;
        tokio::spawn(async move {
            let client = gerrit::client::GerritClient::new(
                cfg.gerrit_url.clone(),
                cfg.gerrit_user.clone(),
                cfg.gerrit_password.clone(),
            );
            let result =
                gerrit::search::search(&client, &tags2, &logic2, &repos, commits, tag_search).await;
            let _ = tx2.send(SearchEvent::GerritResult(result)).await;
        });
    } else {
        let _ = tx.send(SearchEvent::GerritResult(Ok(vec![]))).await;
    }
}
