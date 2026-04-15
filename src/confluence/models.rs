use serde::Deserialize;

use crate::app::PageResult;

// ────────────────────────────────────────────────────────────────────────────
// Serde deserialization types for the Confluence REST API
//
// GET /rest/api/content/search returns:
//   { "results": [ ... ], "size": N }
//
// Each element has the shape modelled by `ConfluenceItem`.
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ConfluenceSearchResponse {
    pub results: Vec<ConfluenceItem>,
    pub size: u32,
}

#[derive(Debug, Deserialize)]
pub struct ConfluenceItem {
    pub id: String,
    pub title: String,
    pub space: ConfluenceSpace,
    #[serde(rename = "_links")]
    pub links: ConfluenceLinks,
    pub version: ConfluenceVersion,
    pub body: Option<ConfluenceBody>,
}

#[derive(Debug, Deserialize)]
pub struct ConfluenceSpace {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfluenceLinks {
    pub webui: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfluenceVersion {
    pub when: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfluenceBody {
    pub storage: ConfluenceStorage,
}

#[derive(Debug, Deserialize)]
pub struct ConfluenceStorage {
    pub value: String,
}

// ────────────────────────────────────────────────────────────────────────────
// Conversion
//
// The tuple `(&ConfluenceItem, &str)` carries the item and the base URL so
// that the absolute webui link can be constructed.
// ────────────────────────────────────────────────────────────────────────────

impl From<(&ConfluenceItem, &str)> for PageResult {
    fn from((item, base_url): (&ConfluenceItem, &str)) -> Self {
        let url = format!("{}{}", base_url.trim_end_matches('/'), item.links.webui);
        PageResult {
            id: item.id.clone(),
            title: item.title.clone(),
            space_key: item.space.key.clone(),
            url,
            last_modified: item.version.when.clone(),
            body: item.body.as_ref().map(|b| b.storage.value.clone()),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(webui: &str) -> ConfluenceItem {
        ConfluenceItem {
            id: "123".into(),
            title: "My Page".into(),
            space: ConfluenceSpace { key: "DS".into() },
            links: ConfluenceLinks {
                webui: webui.into(),
            },
            version: ConfluenceVersion {
                when: "2026-01-01T00:00:00Z".into(),
            },
            body: None,
        }
    }

    #[test]
    fn test_page_result_from_item_no_trailing_slash() {
        let item = make_item("/wiki/spaces/DS/pages/123");
        let result = PageResult::from((&item, "https://confluence.example.com"));
        assert_eq!(
            result.url,
            "https://confluence.example.com/wiki/spaces/DS/pages/123"
        );
        assert_eq!(result.id, "123");
        assert_eq!(result.space_key, "DS");
        assert!(result.body.is_none());
    }

    #[test]
    fn test_page_result_from_item_trailing_slash() {
        let item = make_item("/wiki/spaces/DS/pages/123");
        let result = PageResult::from((&item, "https://confluence.example.com/"));
        // trailing slash on base_url must not produce a double slash
        assert_eq!(
            result.url,
            "https://confluence.example.com/wiki/spaces/DS/pages/123"
        );
    }

    #[test]
    fn test_page_result_body_propagated() {
        let mut item = make_item("/wiki/spaces/DS/pages/456");
        item.id = "456".into();
        item.body = Some(ConfluenceBody {
            storage: ConfluenceStorage {
                value: "<p>Hello</p>".into(),
            },
        });
        let result = PageResult::from((&item, "https://confluence.example.com"));
        assert_eq!(result.body.as_deref(), Some("<p>Hello</p>"));
    }
}
