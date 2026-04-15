# cgrep

> Confluence + Jira + Gerrit を同時検索する横断 grep TUI ツール

![hero demo](docs/gifs/demo.gif)

---

## 概要

**cgrep** は、Confluence・Jira・Gerrit（Server/Data Center）を **単一の検索バー** から同時に検索し、結果をインタラクティブな TUI（Terminal UI）でリアルタイムに一覧表示する Rust 製ツールです。

- タグ形式のキーワード入力（AND / OR 切替）
- 3 サービスの **並列非同期検索**（tokio）
- 検索中はトラボルタが踊る 🕺
- タブ・ブックマーク・検索履歴の永続化

---

## デモ

### メイン画面

![main screen](docs/gifs/demo.gif)

### タグ形式検索バー（AND/OR 切替）

![search bar](docs/gifs/search-bar.gif)

```
🔍 AND │ ❰kubernetes❱ ❰deployment❱ _
         ↑タグ            ↑タグ       ↑入力中
```

Enter でキーワードをタグ化、`Ctrl+M` で AND/OR を切り替え。

### 🕺 トラボルタアニメーション（検索中）

![travolta animation](docs/gifs/travolta.gif)

検索中はフッターにトラボルタが 150ms ごとにフレームを切り替え。

```
フレーム1       フレーム2       フレーム3       フレーム4
  o              \o/             o/              \o
 /|>              |             <|\              |
/ \              / \             / \            / \
```

完了時 → `✨o✨`、失敗時 → `x_x`。

### リアルタイム並列検索プログレス

![parallel search](docs/gifs/parallel-search.gif)

3 サービスの取得状況をフッターに同時表示：

```
Confluence: 取得中...  |  Jira: 8件 ✅  |  Gerrit: 取得中...
```

完了後：

```
Confluence: 12件 ✅  |  Jira: 8件 ✅  |  Gerrit: 6件 ✅  |  計26件
```

### フィルタパネル（Tab キー）

![filter panel](docs/gifs/filter-panel.gif)

ソース・スペース・プロジェクト・ステータス・リポジトリをその場で絞り込み。

### タブ機能（Ctrl+T）

![tabs](docs/gifs/tabs.gif)

複数の検索セッションをタブで管理。各タブは独立した検索状態・結果・フィルタを保持。

### ブックマーク（`b` キー / Ctrl+B）

![bookmarks](docs/gifs/bookmarks.gif)

結果アイテムを `b` でブックマーク追加／削除（トグル）。`Ctrl+B` で一覧を開く。

### 検索履歴（`↑` キー）

![history](docs/gifs/history.gif)

検索を実行するたびに自動保存（最大 100 件）。`↑` でパネルを開き、Enter で再利用。

### プレビューパネル

![preview](docs/gifs/preview.gif)

選択アイテムの詳細を右パネルに表示（Confluence の HTML → プレーンテキスト変換含む）。

### URL コピー（`y` キー）

![copy url](docs/gifs/copy-url.gif)

`y` で選択中アイテムの URL をクリップボードにコピー。成功時にフッターへ通知表示。

### ヘルプ画面（`?` キー）

![help](docs/gifs/help.gif)

全キーバインドを `?` で確認。

---

## インストール

```bash
# Rust がインストールされていない場合
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# ビルド
git clone https://github.com/nigoh/cgrep
cd cgrep
cargo build --release

# パスへ追加
cp target/release/cgrep ~/.local/bin/
```

---

## 設定（環境変数）

| 変数名 | 必須 | 説明 |
|---|---|---|
| `CONFLUENCE_URL` | ✅ | `https://confluence.example.com` |
| `CONFLUENCE_USER` | ✅ | メールアドレス |
| `CONFLUENCE_TOKEN` | ✅ | API トークン |
| `CONFLUENCE_DEFAULT_SPACES` | - | デフォルト対象スペース（カンマ区切り） |
| `JIRA_URL` | ✅ | `https://jira.example.com` |
| `JIRA_USER` | ✅ | メールアドレス |
| `JIRA_TOKEN` | ✅ | API トークン |
| `JIRA_DEFAULT_PROJECTS` | - | デフォルト対象プロジェクト（カンマ区切り） |
| `JIRA_DEFAULT_STATUSES` | - | デフォルト対象ステータス（カンマ区切り） |
| `GERRIT_URL` | ✅ | `https://gerrit.example.com` |
| `GERRIT_USER` | ✅ | ユーザー名 |
| `GERRIT_PASSWORD` | ✅ | パスワード or API トークン |
| `GERRIT_DEFAULT_REPOS` | - | デフォルト対象リポジトリ（カンマ区切り） |

`.env` ファイルを使う場合：

```bash
# ~/.config/cgrep/.env
export CONFLUENCE_URL=https://confluence.example.com
export CONFLUENCE_USER=your@email.com
export CONFLUENCE_TOKEN=your_api_token
export CONFLUENCE_DEFAULT_SPACES=DS,OPS

export JIRA_URL=https://jira.example.com
export JIRA_USER=your@email.com
export JIRA_TOKEN=your_api_token
export JIRA_DEFAULT_PROJECTS=OPS,DS
export JIRA_DEFAULT_STATUSES=In Progress,Open

export GERRIT_URL=https://gerrit.example.com
export GERRIT_USER=your_username
export GERRIT_PASSWORD=your_password_or_token
export GERRIT_DEFAULT_REPOS=infra,platform
```

```bash
source ~/.config/cgrep/.env && cgrep
```

---

## 使い方

```bash
cgrep
```

起動後：

1. キーワードを入力して **Enter** でタグ化
2. 複数タグを入力後、空のまま **Enter** で検索（Normal モード）
3. Incremental モード（デフォルト）ではタグ追加のたびに自動検索

---

## キーバインド

### グローバル

| キー | 動作 |
|---|---|
| `q` / `Ctrl+C` | 終了 |
| `?` | ヘルプ表示 |
| `Ctrl+T` | 新規タブ |
| `Ctrl+W` | タブを閉じる |
| `Ctrl+←` / `Ctrl+→` | タブ切替 |
| `Ctrl+S` | タブを保存 |
| `Alt+1〜9` | タブを番号で直接選択 |
| `Ctrl+B` | ブックマーク一覧 |

### 検索バー

| キー | 動作 |
|---|---|
| 文字入力 | テキスト入力 |
| `Enter` | タグ化 / 検索実行（Normal モード） |
| `↑` | 検索履歴パネルを開く |
| `Backspace` | 最後のタグを削除（テキスト空のとき） |
| `Ctrl+M` | AND / OR 切替 |
| `/` | Incremental ↔ Normal モード切替 |
| `Tab` | フィルタパネル展開/折りたたみ |

### 結果リスト

| キー | 動作 |
|---|---|
| `↑` / `↓` | 移動 |
| `Enter` | ブラウザで開く |
| `Space` | グループの折りたたみ/展開 |
| `y` | URL をクリップボードにコピー |
| `b` | ブックマーク追加/削除（トグル） |
| `p` | プレビューにフォーカス移動 |

### フィルタパネル内

| キー | 動作 |
|---|---|
| `↑` / `↓` | 項目移動 |
| `Space` | ON/OFF 切替 |
| `Esc` | フィルタパネルを閉じる |

### ブックマーク一覧内

| キー | 動作 |
|---|---|
| `↑` / `↓` | 移動 |
| `Enter` | ブラウザで開く |
| `y` | URL をクリップボードにコピー |
| `d` | ブックマークを削除 |
| `Esc` | 閉じる |

### 検索履歴内

| キー | 動作 |
|---|---|
| `↑` / `↓` | 移動 |
| `Enter` | タグ・ロジックを検索バーに復元 |
| `d` | 削除 |
| `Esc` | 閉じる |

---

## 永続化ファイル

```
~/.config/cgrep/
├── history.json      # 検索履歴（最大 100 件）
├── bookmarks.json    # ブックマーク
└── tabs.json         # 保存済みタブセッション（Ctrl+S で保存）
```

---

## デモ GIF の録画

GIF は [vhs](https://github.com/charmbracelet/vhs) で録画しています。

```bash
# vhs をインストール
brew install charmbracelet/tap/vhs  # macOS
# または
go install github.com/charmbracelet/vhs@latest

# 録画
vhs docs/tapes/demo.tape
```

テープファイルは `docs/tapes/` 以下にあります。

---

## アーキテクチャ

```
キーワードタグ確定 / フィルタ変更
         │
         ├──────────────────┬───────────────────┐
         ▼                  ▼                   ▼
  Confluence 検索      Jira 検索          Gerrit 検索
  （tokio::spawn）   （tokio::spawn）   （tokio::spawn）
         │                  │                   │
         └──────────────────┼───────────────────┘
                            ▼
                  mpsc::channel で UI へ結果を送信
                            │
                    全タスク完了 → ✨ アニメーション終了
```

---

## ライセンス

MIT
