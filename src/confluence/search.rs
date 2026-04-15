use std::collections::HashSet;

use anyhow::{Context, Result};

use crate::app::{PageResult, SearchLogic};
use super::client::ConfluenceClient;
use super::models::{ConfluenceItem, ConfluenceSearchResponse};

// ────────────────────────────────────────────────────────────────────────────
// CQL builder
// ────────────────────────────────────────────────────────────────────────────

/// Build a CQL query string for a full-text Confluence search.
///
/// * `tags`   – keywords to search for (empty → returns `""`).
/// * `logic`  – whether keywords are combined with AND or OR.
/// * `spaces` – optional set of Confluence space keys to restrict results.
///
/// The query always ends with `ORDER BY lastmodified DESC`.
pub fn build_cql(tags: &[String], logic: &SearchLogic, spaces: &HashSet<String>) -> String {
    if tags.is_empty() {
        return String::new();
    }

    let op = match logic {
        SearchLogic::And => " AND ",
        SearchLogic::Or => " OR ",
    };

    let keyword_clause = tags
        .iter()
        .map(|t| format!(r#"text ~ "{}""#, t.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(op);

    let mut cql = keyword_clause;

    if !spaces.is_empty() {
        let keys = spaces
            .iter()
            .map(|s| format!(r#""{}""#, s.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(",");
        cql.push_str(&format!(" AND space.key in ({keys})"));
    }

    cql.push_str(" ORDER BY lastmodified DESC");
    cql
}

// ────────────────────────────────────────────────────────────────────────────
// HTTP helpers
// ────────────────────────────────────────────────────────────────────────────

/// Search Confluence with the given keywords and filters.
///
/// Calls `GET {base_url}/rest/api/content/search?cql=…&limit=50&expand=space,version`
/// and converts each result to a [`PageResult`].
pub async fn search(
    client: &ConfluenceClient,
    tags: &[String],
    logic: &SearchLogic,
    spaces: &HashSet<String>,
) -> Result<Vec<PageResult>> {
    let cql = build_cql(tags, logic, spaces);
    if cql.is_empty() {
        return Ok(Vec::new());
    }

    let url = format!("{}/rest/api/content/search", client.base_url());

    let response = client
        .client()
        .get(&url)
        .header("Authorization", client.auth_header())
        .query(&[
            ("cql", cql.as_str()),
            ("limit", "50"),
            ("expand", "space,version"),
        ])
        .send()
        .await
        .context("HTTP request to Confluence search failed")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Confluence search returned {status}: {body}");
    }

    let data: ConfluenceSearchResponse = response
        .json()
        .await
        .context("Failed to parse Confluence search response")?;

    let base_url = client.base_url();
    let results = data
        .results
        .iter()
        .map(|item| PageResult::from((item, base_url)))
        .collect();

    Ok(results)
}

/// Fetch the HTML body of a single Confluence page by its numeric ID.
///
/// Calls `GET {base_url}/rest/api/content/{id}?expand=body.storage`
/// and returns the raw HTML stored in `body.storage.value`.
/// The UI layer is responsible for converting this to plain text (e.g. with
/// `html2text`).
pub async fn fetch_body(client: &ConfluenceClient, id: &str) -> Result<String> {
    let url = format!("{}/rest/api/content/{id}", client.base_url());

    let response = client
        .client()
        .get(&url)
        .header("Authorization", client.auth_header())
        .query(&[("expand", "body.storage")])
        .send()
        .await
        .context("HTTP request to Confluence content endpoint failed")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Confluence content/{id} returned {status}: {body}");
    }

    let item: ConfluenceItem = response
        .json()
        .await
        .context("Failed to parse Confluence content response")?;

    let html = item
        .body
        .map(|b| b.storage.value)
        .unwrap_or_default();

    Ok(html)
}

// ────────────────────────────────────────────────────────────────────────────
// Tests  (no network calls — only the pure CQL builder is exercised)
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── AND mode ─────────────────────────────────────────────────────────────

    #[test]
    fn test_build_cql_and() {
        let tags = vec!["kubernetes".to_string(), "deploy".to_string()];
        let cql = build_cql(&tags, &SearchLogic::And, &HashSet::new());
        assert!(
            cql.contains(r#"text ~ "kubernetes" AND text ~ "deploy""#),
            "unexpected cql: {cql}"
        );
        assert!(cql.ends_with("ORDER BY lastmodified DESC"), "missing ORDER BY: {cql}");
    }

    // ── OR mode ──────────────────────────────────────────────────────────────

    #[test]
    fn test_build_cql_or() {
        let tags = vec!["kubernetes".to_string(), "deploy".to_string()];
        let cql = build_cql(&tags, &SearchLogic::Or, &HashSet::new());
        assert!(
            cql.contains(r#"text ~ "kubernetes" OR text ~ "deploy""#),
            "unexpected cql: {cql}"
        );
        assert!(cql.ends_with("ORDER BY lastmodified DESC"), "missing ORDER BY: {cql}");
    }

    // ── Space filter ──────────────────────────────────────────────────────────

    #[test]
    fn test_build_cql_with_spaces() {
        let tags = vec!["runbook".to_string()];
        let spaces: HashSet<String> = ["DS".to_string(), "OPS".to_string()].into();
        let cql = build_cql(&tags, &SearchLogic::And, &spaces);

        // The keyword clause must be present
        assert!(cql.contains(r#"text ~ "runbook""#), "missing keyword: {cql}");

        // Both space keys must appear inside a space.key in (...) clause
        assert!(cql.contains("space.key in ("), "missing space filter: {cql}");
        assert!(cql.contains(r#""DS""#), "missing DS: {cql}");
        assert!(cql.contains(r#""OPS""#), "missing OPS: {cql}");

        assert!(cql.ends_with("ORDER BY lastmodified DESC"), "missing ORDER BY: {cql}");
    }

    // ── Empty tags ────────────────────────────────────────────────────────────

    #[test]
    fn test_build_cql_empty_tags() {
        // Empty tag list → empty string (caller should skip the search)
        let cql = build_cql(&[], &SearchLogic::And, &HashSet::new());
        assert!(cql.is_empty(), "expected empty string for no tags, got: {cql:?}");
    }

    // ── Single tag ────────────────────────────────────────────────────────────

    #[test]
    fn test_build_cql_single_tag() {
        let tags = vec!["incident".to_string()];
        let cql = build_cql(&tags, &SearchLogic::And, &HashSet::new());
        assert_eq!(
            cql,
            r#"text ~ "incident" ORDER BY lastmodified DESC"#,
            "unexpected cql: {cql}"
        );
    }
}
