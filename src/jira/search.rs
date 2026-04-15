use std::collections::HashSet;

use anyhow::Context;

use crate::app::{IssueResult, SearchLogic};
use super::client::JiraClient;
use super::models::{JiraIssue, JiraSearchResponse};

/// Build a JQL query string from the given tags, logic, project filter, and status filter.
pub fn build_jql(
    tags: &[String],
    logic: &SearchLogic,
    projects: &HashSet<String>,
    statuses: &HashSet<String>,
) -> String {
    let connector = match logic {
        SearchLogic::And => " AND ",
        SearchLogic::Or => " OR ",
    };

    let keyword_clause = tags
        .iter()
        .map(|t| format!("text ~ \"{}\"", t))
        .collect::<Vec<_>>()
        .join(connector);

    let mut clauses: Vec<String> = Vec::new();
    if !keyword_clause.is_empty() {
        clauses.push(keyword_clause);
    }

    if !projects.is_empty() {
        let mut sorted: Vec<&String> = projects.iter().collect();
        sorted.sort();
        let list = sorted
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(",");
        clauses.push(format!("project in ({})", list));
    }

    if !statuses.is_empty() {
        let mut sorted: Vec<&String> = statuses.iter().collect();
        sorted.sort();
        let list = sorted
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",");
        clauses.push(format!("status in ({})", list));
    }

    let base = clauses.join(" AND ");
    format!("{} ORDER BY updated DESC", base)
}

/// Search Jira issues using the given filters.
pub async fn search(
    client: &JiraClient,
    tags: &[String],
    logic: &SearchLogic,
    projects: &HashSet<String>,
    statuses: &HashSet<String>,
) -> anyhow::Result<Vec<IssueResult>> {
    let jql = build_jql(tags, logic, projects, statuses);
    let url = format!("{}/rest/api/2/search", client.base_url());

    let response = client
        .client()
        .get(&url)
        .query(&[
            ("jql", jql.as_str()),
            ("maxResults", "50"),
            ("fields", "summary,status,project,description"),
        ])
        .header("Authorization", client.auth_header())
        .header("Accept", "application/json")
        .send()
        .await
        .context("Failed to send Jira search request")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Jira search returned {}: {}", status, body);
    }

    let search_response: JiraSearchResponse = response
        .json()
        .await
        .context("Failed to parse Jira search response")?;

    let results = search_response
        .issues
        .into_iter()
        .map(IssueResult::from)
        .collect();

    Ok(results)
}

/// Fetch a single Jira issue by key.
pub async fn fetch_issue(client: &JiraClient, key: &str) -> anyhow::Result<IssueResult> {
    let url = format!(
        "{}/rest/api/2/issue/{}?fields=summary,status,project,description",
        client.base_url(),
        key,
    );

    let response = client
        .client()
        .get(&url)
        .header("Authorization", client.auth_header())
        .header("Accept", "application/json")
        .send()
        .await
        .context("Failed to send Jira fetch_issue request")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Jira fetch_issue returned {}: {}", status, body);
    }

    let issue: JiraIssue = response
        .json()
        .await
        .context("Failed to parse Jira issue response")?;

    Ok(IssueResult::from(issue))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_projects() -> HashSet<String> {
        HashSet::new()
    }

    fn empty_statuses() -> HashSet<String> {
        HashSet::new()
    }

    fn tags(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn project_set(v: &[&str]) -> HashSet<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn status_set(v: &[&str]) -> HashSet<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_build_jql_and() {
        let jql = build_jql(
            &tags(&["kw1", "kw2"]),
            &SearchLogic::And,
            &empty_projects(),
            &empty_statuses(),
        );
        assert_eq!(
            jql,
            "text ~ \"kw1\" AND text ~ \"kw2\" ORDER BY updated DESC"
        );
    }

    #[test]
    fn test_build_jql_or() {
        let jql = build_jql(
            &tags(&["kw1", "kw2"]),
            &SearchLogic::Or,
            &empty_projects(),
            &empty_statuses(),
        );
        assert_eq!(
            jql,
            "text ~ \"kw1\" OR text ~ \"kw2\" ORDER BY updated DESC"
        );
    }

    #[test]
    fn test_build_jql_with_projects() {
        let jql = build_jql(
            &tags(&["kw1"]),
            &SearchLogic::And,
            &project_set(&["OPS", "DS"]),
            &empty_statuses(),
        );
        // Projects are sorted alphabetically: DS, OPS
        assert!(jql.contains("project in (\"DS\",\"OPS\")"), "got: {}", jql);
        assert!(jql.ends_with("ORDER BY updated DESC"), "got: {}", jql);
    }

    #[test]
    fn test_build_jql_with_statuses() {
        let jql = build_jql(
            &tags(&["kw1"]),
            &SearchLogic::And,
            &empty_projects(),
            &status_set(&["In Progress", "Open"]),
        );
        // Statuses are sorted: "In Progress", "Open"
        assert!(
            jql.contains("status in (\"In Progress\",\"Open\")"),
            "got: {}",
            jql
        );
        assert!(jql.ends_with("ORDER BY updated DESC"), "got: {}", jql);
    }

    #[test]
    fn test_build_jql_combined() {
        let jql = build_jql(
            &tags(&["incident"]),
            &SearchLogic::And,
            &project_set(&["OPS"]),
            &status_set(&["Open"]),
        );
        assert!(jql.contains("text ~ \"incident\""), "got: {}", jql);
        assert!(jql.contains("project in (\"OPS\")"), "got: {}", jql);
        assert!(jql.contains("status in (\"Open\")"), "got: {}", jql);
        assert!(jql.ends_with("ORDER BY updated DESC"), "got: {}", jql);
    }
}
