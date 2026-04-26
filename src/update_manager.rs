/// Application update checking against GitHub releases.
use serde::Deserialize;

use crate::VERSION;
use std::time::Duration;

const GITHUB_API_URL: &str = "https://api.github.com/repos/tryigit/FOEM/releases/latest";
const RELEASES_URL: &str = "https://github.com/tryigit/FOEM/releases";

#[derive(Debug)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub download_url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: Option<String>,
}

pub struct UpdateManager {
    pub agent: ureq::Agent,
}

impl UpdateManager {
    pub fn new() -> Self {
        Self {
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(10))
                .build(),
        }
    }

    /// Query the GitHub API for the latest release.
    ///
    /// Returns `Ok(Some(info))` when a newer version is available,
    /// `Ok(None)` when the current version is already the latest,
    /// or `Err` on network / parse failures.
    pub fn check_for_updates(&self) -> Result<Option<UpdateInfo>, String> {
        let response = self.agent.get(GITHUB_API_URL)
            .set("Accept", "application/vnd.github.v3+json")
            .set("User-Agent", "FOEM-UpdateChecker")
            .call()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let release: GithubRelease = response
            .into_json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let latest = release.tag_name.trim_start_matches('v').to_string();
        let download_url = release.html_url.unwrap_or_else(|| RELEASES_URL.to_string());

        if latest != VERSION {
            Ok(Some(UpdateInfo {
                latest_version: latest,
                download_url,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let manager = UpdateManager::new();
        // Since we cannot inspect the internal timeout value of ureq::Agent directly,
        // we can verify the struct was successfully created and the agent is present
        // by checking if it can build a request.
        let agent = manager.agent;
        let req = agent.request("GET", "http://test.com");
        assert_eq!(req.method(), "GET");
    }
}
