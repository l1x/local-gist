use reqwest::header::HeaderMap;
use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io::Error as IoError;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum GistError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] ReqwestError),
    #[error("IO operation failed: {0}")]
    IoError(#[from] IoError),
    #[error("JSON parsing failed: {0}\nResponse text: {1}")]
    JsonError(serde_json::Error, String),
}

// GitHub API base URL
const GITHUB_API_URL: &str = "https://api.github.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct GistFile {
    pub filename: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub language: Option<String>,
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
    pub description: Option<String>,
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
        let s = match &self.description {
            Some(d) => &d,
            None => "<no description>",
        };
        write!(
            f,
            "{} - {} ({})",
            self.id,
            s,
            self.files
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
pub type Gists = Vec<Gist>;

fn has_next_page(headers: &HeaderMap) -> bool {
    headers
        .get("link")
        .and_then(|link| link.to_str().ok())
        .map(|link| link.contains(r#"rel="next"#))
        .unwrap_or(false)
}

fn get_url(username: &str, per_page: u32, page: u32) -> String {
    return format!(
        "{}/users/{}/gists?per_page={}&page={}",
        GITHUB_API_URL, username, per_page, page
    );
}

fn get_rate_limit(headers: &HeaderMap) -> Option<&str> {
    let rate_limit = headers
        .get("x-ratelimit-limit")
        .and_then(|h| h.to_str().ok());
    let rate_remaining = headers
        .get("x-ratelimit-remaining")
        .and_then(|h| h.to_str().ok());

    info!(
        "rate_limit: {:?} rate_remaining: {:?}",
        rate_limit, rate_remaining
    );

    rate_remaining
}

fn should_continue(remaining: Option<&str>) -> bool {
    remaining
        .and_then(|r| r.parse::<u32>().ok())
        .map_or(false, |n| n > 0)
}

/// Lists all Gists for a given GitHub username.
///
/// # Arguments
/// * `username` - GitHub username to fetch gists for
/// * `limit` - Optional maximum number of gists to return)
pub async fn list_gists(username: &str, limit: Option<u32>) -> Result<Gists, GistError> {
    let client: Client = Client::builder().user_agent("RustRequestClient").build()?;
    let mut all_gists: Vec<Gist> = Vec::new();
    let mut page: u32 = 1;
    let per_page: u32 = limit.unwrap_or(100);

    info!("Limit: {:?}, per page: {:?} ", limit, per_page);

    loop {
        let url: String = get_url(username, per_page, page);
        info!("Requesting URL: {}", url);
        let response: reqwest::Response = client.get(&url).send().await?;
        info!("Status: {}", response.status());
        let has_next_page: bool = has_next_page(response.headers());
        if has_next_page {
            info!("Wait, there is more!")
        } else {
            info!("There are no more gists")
        }
        let rate_remaining = get_rate_limit(response.headers());
        match should_continue(rate_remaining) {
            true => debug!("We can continue, there is rate limit left to use"),
            false => {
                info!("We need to slow down");
                sleep(Duration::from_millis(3000)).await;
            }
        };

        let text: String = response.text().await?;

        match serde_json::from_str::<Vec<Gist>>(&text) {
            Ok(mut gists) => {
                all_gists.append(&mut gists);
            }
            Err(e) => {
                // Print error context
                info!("Error details: {}", e);
                info!("Error location: line {}, column {}", e.line(), e.column());

                // Get a snippet of the JSON around the error
                let start_pos = e.column().saturating_sub(50);
                let end_pos = (e.column() + 50).min(text.len());
                let context = &text[start_pos..end_pos];
                info!("JSON context around error: {}", context);

                return Err(GistError::JsonError(e, text));
            }
        }

        if let Some(limit) = limit {
            if all_gists.len() >= limit as usize {
                all_gists.truncate(limit as usize);
                break;
            }
        }

        if !has_next_page {
            break;
        }

        page += 1;
    }

    Ok(all_gists)
}
/// Downloads a single gist to a specified path
///
/// # Arguments
/// * `gist` - The Gist to download
/// * `output_path` - Directory where the gist should be saved
pub async fn download_gist(gist: &Gist, output_path: &str) -> Result<(), GistError> {
    let client = Client::builder().user_agent("RustRequestClient").build()?;

    // Create the parent directory if it doesn't exist
    let base_dir = format!("{}/{}", output_path, gist.id);
    std::fs::create_dir_all(&base_dir)?;

    // Download each file in the gist
    for (filename, file) in &gist.files {
        // Get the file content
        let response = client.get(&file.raw_url).send().await?.text().await?;

        // Create the full path for the file
        let file_path = format!("{}/{}", base_dir, filename);

        // Write the content to a file
        std::fs::write(file_path, response)?;
    }

    Ok(())
}
