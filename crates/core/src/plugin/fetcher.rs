// Plugin fetching and building

use super::PluginError;
use std::time::Duration;

pub struct PluginFetcher {
    client: reqwest::Client,
}

impl PluginFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Fetch plugin WASM from a GitHub release
    /// Returns the WASM bytes
    pub async fn fetch_release(
        &self,
        git_url: &str,
        git_ref: &str,
    ) -> Result<Vec<u8>, PluginError> {
        // Parse GitHub URL to extract owner and repo
        // Example: https://github.com/web-tree/rw-plugins or git@github.com:web-tree/rw-plugins
        let (owner, repo) = parse_github_url(git_url)?;

        // Try to fetch from GitHub Releases
        // First try with the git_ref as a tag
        let release_url = format!(
            "https://github.com/{}/{}/releases/download/{}/plugin.wasm",
            owner, repo, git_ref
        );

        self.fetch_from_url(&release_url).await
    }

    /// Fetch WASM from a direct URL with retry logic
    pub async fn fetch_from_url(&self, url: &str) -> Result<Vec<u8>, PluginError> {
        let mut last_error = None;
        let max_retries = 3;

        for attempt in 0..max_retries {
            if attempt > 0 {
                // Exponential backoff
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                tokio::time::sleep(delay).await;
            }

            match self.try_fetch(url).await {
                Ok(data) => return Ok(data),
                Err(e) => last_error = Some(e),
            }
        }

        Err(last_error.unwrap_or_else(|| PluginError::FetchError {
            message: "Failed to fetch plugin after retries".to_string(),
            source: None,
        }))
    }

    /// Single fetch attempt
    async fn try_fetch(&self, url: &str) -> Result<Vec<u8>, PluginError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| PluginError::FetchError {
                message: format!("Failed to download from {}", url),
                source: Some(e.into()),
            })?;

        if !response.status().is_success() {
            return Err(PluginError::FetchError {
                message: format!("HTTP {} when fetching {}", response.status(), url),
                source: None,
            });
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| PluginError::FetchError {
                message: "Failed to read response bytes".to_string(),
                source: Some(e.into()),
            })?;

        Ok(bytes.to_vec())
    }
}

impl Default for PluginFetcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse GitHub URL to extract owner and repo
fn parse_github_url(url: &str) -> Result<(String, String), PluginError> {
    // Remove .git suffix if present
    let url = url.trim_end_matches(".git");

    // Handle HTTPS URLs
    if let Some(path) = url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Handle SSH URLs
    if let Some(path) = url.strip_prefix("git@github.com:") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    Err(PluginError::ConfigError {
        message: format!("Invalid GitHub URL: {}", url),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_https_url() {
        let (owner, repo) = parse_github_url("https://github.com/web-tree/rw-plugins").unwrap();
        assert_eq!(owner, "web-tree");
        assert_eq!(repo, "rw-plugins");
    }

    #[test]
    fn test_parse_github_ssh_url() {
        let (owner, repo) = parse_github_url("git@github.com:web-tree/rw-plugins").unwrap();
        assert_eq!(owner, "web-tree");
        assert_eq!(repo, "rw-plugins");
    }

    #[test]
    fn test_parse_github_url_with_git_suffix() {
        let (owner, repo) = parse_github_url("https://github.com/web-tree/rw-plugins.git").unwrap();
        assert_eq!(owner, "web-tree");
        assert_eq!(repo, "rw-plugins");
    }
}
