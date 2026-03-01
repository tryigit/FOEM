/// Application update checking against GitHub releases.
use serde::Deserialize;

use crate::VERSION;

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/tryigit/FOEM/releases/latest";
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

pub struct UpdateManager;

impl UpdateManager {
    pub fn new() -> Self {
        Self
    }

    /// Query the GitHub API for the latest release.
    ///
    /// Returns `Ok(Some(info))` when a newer version is available,
    /// `Ok(None)` when the current version is already the latest,
    /// or `Err` on network / parse failures.
    pub fn check_for_updates(&self) -> Result<Option<UpdateInfo>, String> {
        let response = ureq::get(GITHUB_API_URL)
            .set("Accept", "application/vnd.github.v3+json")
            .set("User-Agent", "FOEM-UpdateChecker")
            .call()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let release: GithubRelease = response
            .into_json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let latest = release.tag_name.trim_start_matches('v').to_string();
        let download_url = release
            .html_url
            .unwrap_or_else(|| RELEASES_URL.to_string());

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
