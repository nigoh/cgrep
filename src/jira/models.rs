use serde::Deserialize;

use crate::app::IssueResult;

#[derive(Debug, Deserialize)]
pub struct JiraSearchResponse {
    pub issues: Vec<JiraIssue>,
    pub total: u32,
}

#[derive(Debug, Deserialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    pub fields: JiraFields,
    #[serde(rename = "self")]
    pub self_url: String,
}

#[derive(Debug, Deserialize)]
pub struct JiraFields {
    pub summary: String,
    pub status: JiraStatus,
    pub project: JiraProject,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JiraStatus {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct JiraProject {
    pub key: String,
    pub name: String,
}

impl From<JiraIssue> for IssueResult {
    fn from(issue: JiraIssue) -> Self {
        IssueResult {
            key: issue.key.clone(),
            summary: issue.fields.summary,
            project: issue.fields.project.key,
            status: issue.fields.status.name,
            url: issue.self_url,
            description: issue.fields.description,
        }
    }
}
