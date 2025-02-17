use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// GitHub API base URL
const GITHUB_API_URL: &str = "https://api.github.com";
const DEFAULT_LIMIT: u32 = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct GistFile {
    pub filename: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub language: String,
    pub raw_url: String,
    pub size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GistOwner {
    pub login: String,
    pub id: u64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub site_admin: bool,
    pub user_view_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gist {
    pub url: String,
    pub forks_url: String,
    pub commits_url: String,
    pub id: String,
    pub node_id: String,
    pub git_pull_url: String,
    pub git_push_url: String,
    pub html_url: String,
    pub files: HashMap<String, GistFile>,
    pub public: bool,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
    pub comments: u32,
    pub user: Option<GistOwner>,
    pub comments_enabled: bool,
    pub comments_url: String,
    pub owner: GistOwner,
    pub truncated: bool,
}
// Add Display implementation for Gist
impl fmt::Display for Gist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {} ({})",
            self.id,
            if self.description.is_empty() {
                "<no description>"
            } else {
                &self.description
            },
            self.files
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
pub type Gists = Vec<Gist>;

/// Lists all Gists for a given GitHub username.
///
/// # Arguments
/// * `username` - GitHub username to fetch gists for
/// * `limit` - Optional maximum number of gists to return (defaults to 10)
pub async fn list_gists(username: &str, limit: Option<u32>) -> Result<Gists, Error> {
    // Create a client with custom headers
    let client: Client = Client::builder().user_agent("RustRequestClient").build()?;

    // Use the provided limit or default to 10
    let per_page: u32 = limit.unwrap_or(DEFAULT_LIMIT);

    let url = format!(
        "{}/users/{}/gists?per_page={}",
        GITHUB_API_URL, username, per_page
    );

    Ok(client.get(&url).send().await?.json().await?)
}
