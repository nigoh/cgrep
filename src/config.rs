use anyhow::{anyhow, Result};
use std::env;

/// Application-wide configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    // Confluence
    pub confluence_url: String,
    pub confluence_user: String,
    pub confluence_token: String,
    pub confluence_default_spaces: Vec<String>,

    // Jira
    pub jira_url: String,
    pub jira_user: String,
    pub jira_token: String,
    pub jira_default_projects: Vec<String>,
    pub jira_default_statuses: Vec<String>,

    // Gerrit
    pub gerrit_url: String,
    pub gerrit_user: String,
    pub gerrit_password: String,
    pub gerrit_default_repos: Vec<String>,
}

impl Config {
    /// Load configuration from environment variables.
    /// Returns an error listing all missing required variables.
    pub fn from_env() -> Result<Self> {
        let mut missing: Vec<&str> = Vec::new();

        macro_rules! require {
            ($var:expr) => {
                match env::var($var) {
                    Ok(v) => v,
                    Err(_) => {
                        missing.push($var);
                        String::new()
                    }
                }
            };
        }

        let confluence_url = require!("CONFLUENCE_URL");
        let confluence_user = require!("CONFLUENCE_USER");
        let confluence_token = require!("CONFLUENCE_TOKEN");
        let jira_url = require!("JIRA_URL");
        let jira_user = require!("JIRA_USER");
        let jira_token = require!("JIRA_TOKEN");
        let gerrit_url = require!("GERRIT_URL");
        let gerrit_user = require!("GERRIT_USER");
        let gerrit_password = require!("GERRIT_PASSWORD");

        if !missing.is_empty() {
            return Err(anyhow!(
                "Missing required environment variables:\n{}",
                missing
                    .iter()
                    .map(|v| format!("  - {v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        Ok(Self {
            confluence_url,
            confluence_user,
            confluence_token,
            confluence_default_spaces: parse_csv(
                &env::var("CONFLUENCE_DEFAULT_SPACES").unwrap_or_default(),
            ),
            jira_url,
            jira_user,
            jira_token,
            jira_default_projects: parse_csv(
                &env::var("JIRA_DEFAULT_PROJECTS").unwrap_or_default(),
            ),
            jira_default_statuses: parse_csv(
                &env::var("JIRA_DEFAULT_STATUSES").unwrap_or_default(),
            ),
            gerrit_url,
            gerrit_user,
            gerrit_password,
            gerrit_default_repos: parse_csv(&env::var("GERRIT_DEFAULT_REPOS").unwrap_or_default()),
        })
    }
}

fn parse_csv(s: &str) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn set_env(vars: &HashMap<&str, &str>) {
        for (k, v) in vars {
            env::set_var(k, v);
        }
    }

    fn unset_env(keys: &[&str]) {
        for k in keys {
            env::remove_var(k);
        }
    }

    fn all_required_vars() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            ("CONFLUENCE_URL", "https://confluence.example.com"),
            ("CONFLUENCE_USER", "user@example.com"),
            ("CONFLUENCE_TOKEN", "conf_token"),
            ("JIRA_URL", "https://jira.example.com"),
            ("JIRA_USER", "user@example.com"),
            ("JIRA_TOKEN", "jira_token"),
            ("GERRIT_URL", "https://gerrit.example.com"),
            ("GERRIT_USER", "gerrit_user"),
            ("GERRIT_PASSWORD", "gerrit_pass"),
        ])
    }

    #[test]
    fn test_all_vars_present() {
        let vars = all_required_vars();
        set_env(&vars);
        env::set_var("CONFLUENCE_DEFAULT_SPACES", "DS,OPS");
        env::set_var("JIRA_DEFAULT_PROJECTS", "OPS,DS");
        env::set_var("JIRA_DEFAULT_STATUSES", "In Progress,Open");
        env::set_var("GERRIT_DEFAULT_REPOS", "infra,platform");

        let cfg = Config::from_env().expect("Config should load");
        assert_eq!(cfg.confluence_url, "https://confluence.example.com");
        assert_eq!(cfg.confluence_user, "user@example.com");
        assert_eq!(cfg.confluence_token, "conf_token");
        assert_eq!(cfg.confluence_default_spaces, vec!["DS", "OPS"]);
        assert_eq!(cfg.jira_url, "https://jira.example.com");
        assert_eq!(cfg.jira_default_projects, vec!["OPS", "DS"]);
        assert_eq!(cfg.jira_default_statuses, vec!["In Progress", "Open"]);
        assert_eq!(cfg.gerrit_url, "https://gerrit.example.com");
        assert_eq!(cfg.gerrit_default_repos, vec!["infra", "platform"]);

        unset_env(&[
            "CONFLUENCE_URL",
            "CONFLUENCE_USER",
            "CONFLUENCE_TOKEN",
            "CONFLUENCE_DEFAULT_SPACES",
            "JIRA_URL",
            "JIRA_USER",
            "JIRA_TOKEN",
            "JIRA_DEFAULT_PROJECTS",
            "JIRA_DEFAULT_STATUSES",
            "GERRIT_URL",
            "GERRIT_USER",
            "GERRIT_PASSWORD",
            "GERRIT_DEFAULT_REPOS",
        ]);
    }

    #[test]
    fn test_missing_required_vars() {
        // Ensure required vars are not set
        unset_env(&[
            "CONFLUENCE_URL",
            "CONFLUENCE_USER",
            "CONFLUENCE_TOKEN",
            "JIRA_URL",
            "JIRA_USER",
            "JIRA_TOKEN",
            "GERRIT_URL",
            "GERRIT_USER",
            "GERRIT_PASSWORD",
        ]);
        let result = Config::from_env();
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("CONFLUENCE_URL"));
        assert!(msg.contains("GERRIT_PASSWORD"));
    }

    #[test]
    fn test_optional_vars_default_to_empty() {
        let vars = all_required_vars();
        set_env(&vars);
        unset_env(&[
            "CONFLUENCE_DEFAULT_SPACES",
            "JIRA_DEFAULT_PROJECTS",
            "JIRA_DEFAULT_STATUSES",
            "GERRIT_DEFAULT_REPOS",
        ]);

        let cfg = Config::from_env().expect("Config should load");
        assert!(cfg.confluence_default_spaces.is_empty());
        assert!(cfg.jira_default_projects.is_empty());
        assert!(cfg.gerrit_default_repos.is_empty());

        unset_env(&[
            "CONFLUENCE_URL",
            "CONFLUENCE_USER",
            "CONFLUENCE_TOKEN",
            "JIRA_URL",
            "JIRA_USER",
            "JIRA_TOKEN",
            "GERRIT_URL",
            "GERRIT_USER",
            "GERRIT_PASSWORD",
        ]);
    }

    #[test]
    fn test_parse_csv() {
        assert_eq!(parse_csv("DS,OPS"), vec!["DS", "OPS"]);
        assert_eq!(parse_csv("DS, OPS , HR"), vec!["DS", "OPS", "HR"]);
        assert_eq!(parse_csv(""), Vec::<String>::new());
        assert_eq!(parse_csv("single"), vec!["single"]);
    }
}
