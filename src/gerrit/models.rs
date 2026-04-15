use serde::Deserialize;

use crate::app::{CommitResult, TagResult};

// ────────────────────────────────────────────────────────────────────────────
// XSSI prefix stripping
//
// Gerrit's REST API prepends ")]}'\n" to every JSON response to prevent
// Cross-Site Script Inclusion attacks.  Strip it before parsing.
// ────────────────────────────────────────────────────────────────────────────

const XSSI_PREFIX: &str = ")]}'\n";

pub fn strip_xssi(s: &str) -> &str {
    s.strip_prefix(XSSI_PREFIX).unwrap_or(s)
}

// ────────────────────────────────────────────────────────────────────────────
// Gerrit change (commit) deserialization
//
// GET /a/changes/?q={query}&o=DETAILED_ACCOUNTS
// Returns: [GerritChange, ...]
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GerritChange {
    pub id: String,
    pub project: String,
    pub subject: String,
    pub owner: GerritOwner,
    pub created: String,
}

#[derive(Debug, Deserialize)]
pub struct GerritOwner {
    pub name: Option<String>,
    pub email: Option<String>,
}

// ────────────────────────────────────────────────────────────────────────────
// Gerrit tag deserialization
//
// GET /a/projects/{repo}/tags
// Returns: [GerritTag, ...]
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GerritTag {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub revision: String,
}

// ────────────────────────────────────────────────────────────────────────────
// Conversions
// ────────────────────────────────────────────────────────────────────────────

/// Extract the tag name from a full ref path like `refs/tags/v1.4.2`.
pub fn tag_name_from_ref(ref_: &str) -> &str {
    ref_.strip_prefix("refs/tags/").unwrap_or(ref_)
}

/// Convert a `GerritChange` into a `CommitResult` given the Gerrit base URL.
///
/// The URL uses `{base_url}/q/{change_id}` because the `id` field in the
/// DETAILED_ACCOUNTS response is in the form `<project>~<branch>~<Change-Id>`,
/// not a numeric change number.  The `/q/` search URL accepts Change-Id strings
/// and always resolves to the correct change page.
pub fn gerrit_change_to_commit_result(change: GerritChange, base_url: &str) -> CommitResult {
    let owner = change.owner.name.or(change.owner.email).unwrap_or_default();

    let url = format!("{}/q/{}", base_url.trim_end_matches('/'), change.id);

    CommitResult {
        change_id: change.id,
        subject: change.subject,
        repo: change.project,
        owner,
        created: change.created,
        url,
    }
}

/// Convert a `GerritTag` into a `TagResult` given the Gerrit base URL and repo name.
pub fn gerrit_tag_to_tag_result(tag: GerritTag, base_url: &str, repo: &str) -> TagResult {
    let name = tag_name_from_ref(&tag.ref_).to_string();
    let url = format!(
        "{}/admin/repos/{}/+/refs/tags/{}",
        base_url.trim_end_matches('/'),
        repo,
        name
    );
    TagResult {
        name,
        repo: repo.to_string(),
        revision: tag.revision,
        url,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_xssi() {
        let input = ")]}'\nsome json here";
        assert_eq!(strip_xssi(input), "some json here");
    }

    #[test]
    fn test_strip_xssi_no_prefix() {
        let input = r#"[{"id": "foo"}]"#;
        assert_eq!(strip_xssi(input), input);
    }

    #[test]
    fn test_tag_name_extraction() {
        assert_eq!(tag_name_from_ref("refs/tags/v1.4.2"), "v1.4.2");
    }

    #[test]
    fn test_gerrit_change_to_commit_result_url() {
        let change = GerritChange {
            id: "infra~main~I1234abcd".into(),
            project: "infra".into(),
            subject: "Fix something".into(),
            owner: GerritOwner {
                name: Some("Alice".into()),
                email: Some("alice@example.com".into()),
            },
            created: "2026-01-01 00:00:00.000000000".into(),
        };
        let result = gerrit_change_to_commit_result(change, "https://gerrit.example.com");
        assert_eq!(
            result.url,
            "https://gerrit.example.com/q/infra~main~I1234abcd"
        );
        assert_eq!(result.owner, "Alice");
        assert_eq!(result.repo, "infra");
    }

    #[test]
    fn test_gerrit_change_owner_falls_back_to_email() {
        let change = GerritChange {
            id: "infra~main~Iabc".into(),
            project: "infra".into(),
            subject: "Subject".into(),
            owner: GerritOwner {
                name: None,
                email: Some("bob@example.com".into()),
            },
            created: "2026-01-01 00:00:00.000000000".into(),
        };
        let result = gerrit_change_to_commit_result(change, "https://gerrit.example.com");
        assert_eq!(result.owner, "bob@example.com");
    }

    #[test]
    fn test_gerrit_tag_to_tag_result_url() {
        let tag = GerritTag {
            ref_: "refs/tags/v1.4.2".into(),
            revision: "deadbeef".into(),
        };
        let result = gerrit_tag_to_tag_result(tag, "https://gerrit.example.com", "platform");
        assert_eq!(result.name, "v1.4.2");
        assert_eq!(
            result.url,
            "https://gerrit.example.com/admin/repos/platform/+/refs/tags/v1.4.2"
        );
        assert_eq!(result.revision, "deadbeef");
    }
}
