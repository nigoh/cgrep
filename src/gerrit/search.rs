use std::collections::HashSet;

use crate::app::{CommitResult, SearchLogic, SearchResult, TagResult};
use crate::gerrit::client::GerritClient;
use crate::gerrit::models::{
    gerrit_change_to_commit_result, gerrit_tag_to_tag_result, strip_xssi, GerritChange, GerritTag,
};

// ────────────────────────────────────────────────────────────────────────────
// Query builder
// ────────────────────────────────────────────────────────────────────────────

/// Build a Gerrit search query from a list of keywords and a logic operator.
///
/// # Examples
///
/// - AND: `message:"kw1" AND message:"kw2"`
/// - OR:  `message:"kw1" OR message:"kw2"`
pub fn build_query(tags: &[String], logic: &SearchLogic) -> String {
    let op = match logic {
        SearchLogic::And => " AND ",
        SearchLogic::Or => " OR ",
    };
    tags.iter()
        .map(|t| format!(r#"message:"{}""#, t))
        .collect::<Vec<_>>()
        .join(op)
}

/// Append project filters to an existing query string.
///
/// If `repos` is non-empty the result looks like:
/// `{base_query} AND (project:infra OR project:platform)`
fn append_project_filter(base: &str, repos: &HashSet<String>) -> String {
    if repos.is_empty() {
        return base.to_string();
    }
    let projects = repos
        .iter()
        .map(|r| format!("project:{}", r))
        .collect::<Vec<_>>()
        .join(" OR ");
    format!("{} AND ({})", base, projects)
}

// ────────────────────────────────────────────────────────────────────────────
// Commit search
// ────────────────────────────────────────────────────────────────────────────

async fn search_commits(
    client: &GerritClient,
    tags: &[String],
    logic: &SearchLogic,
    repos: &HashSet<String>,
) -> anyhow::Result<Vec<CommitResult>> {
    let base_query = build_query(tags, logic);
    let query = append_project_filter(&base_query, repos);

    let url = format!(
        "{}/a/changes/",
        client.base_url().trim_end_matches('/'),
    );

    let response = client
        .client()
        .get(&url)
        .query(&[("q", query.as_str()), ("o", "DETAILED_ACCOUNTS"), ("n", "50")])
        .basic_auth(client.user(), Some(client.password()))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let json = strip_xssi(&response);
    let changes: Vec<GerritChange> = serde_json::from_str(json)?;

    let base_url = client.base_url().to_string();
    let results = changes
        .into_iter()
        .map(|c| gerrit_change_to_commit_result(c, &base_url))
        .collect();

    Ok(results)
}

// ────────────────────────────────────────────────────────────────────────────
// Tag search
// ────────────────────────────────────────────────────────────────────────────

async fn search_tags(
    client: &GerritClient,
    tags: &[String],
    repos: &Vec<String>,
) -> anyhow::Result<Vec<TagResult>> {
    if repos.is_empty() {
        return Ok(Vec::new());
    }

    let mut results: Vec<TagResult> = Vec::new();

    for repo in repos {
        let url = format!(
            "{}/a/projects/{}/tags",
            client.base_url().trim_end_matches('/'),
            repo,
        );

        let response = client
            .client()
            .get(&url)
            .basic_auth(client.user(), Some(client.password()))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let json = strip_xssi(&response);
        let gerrit_tags: Vec<GerritTag> = serde_json::from_str(json)?;

        for gt in gerrit_tags {
            // The display name is the portion after "refs/tags/".
            let name = gt.ref_.strip_prefix("refs/tags/").unwrap_or(&gt.ref_);
            // Keep the tag only if its name contains at least one keyword.
            let matches = tags.iter().any(|kw| name.contains(kw.as_str()));
            if matches {
                results.push(gerrit_tag_to_tag_result(gt, client.base_url(), repo));
            }
        }
    }

    Ok(results)
}

// ────────────────────────────────────────────────────────────────────────────
// Public search entry-point
// ────────────────────────────────────────────────────────────────────────────

/// Run commit and/or tag searches against Gerrit, returning a mixed
/// `Vec<SearchResult>`.
pub async fn search(
    client: &GerritClient,
    tags: &[String],
    logic: &SearchLogic,
    repos: &HashSet<String>,
    include_commits: bool,
    include_tags: bool,
) -> anyhow::Result<Vec<SearchResult>> {
    let mut results: Vec<SearchResult> = Vec::new();

    if include_commits {
        let commits = search_commits(client, tags, logic, repos).await?;
        results.extend(commits.into_iter().map(SearchResult::Commit));
    }

    if include_tags {
        let repo_list: Vec<String> = repos.iter().cloned().collect();
        let tag_results = search_tags(client, tags, &repo_list).await?;
        results.extend(tag_results.into_iter().map(SearchResult::Tag));
    }

    Ok(results)
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gerrit::models::strip_xssi;

    #[test]
    fn test_build_query_and() {
        let tags = vec!["kw1".to_string(), "kw2".to_string()];
        let result = build_query(&tags, &SearchLogic::And);
        assert_eq!(result, r#"message:"kw1" AND message:"kw2""#);
    }

    #[test]
    fn test_build_query_or() {
        let tags = vec!["kw1".to_string(), "kw2".to_string()];
        let result = build_query(&tags, &SearchLogic::Or);
        assert_eq!(result, r#"message:"kw1" OR message:"kw2""#);
    }

    #[test]
    fn test_strip_xssi() {
        let input = ")]}'\n[{\"id\":\"foo\"}]";
        assert_eq!(strip_xssi(input), "[{\"id\":\"foo\"}]");
    }

    #[test]
    fn test_strip_xssi_no_prefix() {
        let input = "[{\"id\":\"bar\"}]";
        assert_eq!(strip_xssi(input), input);
    }

    #[test]
    fn test_tag_name_extraction() {
        let ref_ = "refs/tags/v1.4.2";
        let name = ref_.strip_prefix("refs/tags/").unwrap_or(ref_);
        assert_eq!(name, "v1.4.2");
    }
}
