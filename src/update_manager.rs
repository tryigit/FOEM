/// Application update checking against GitHub releases.
use serde::Deserialize;

use crate::VERSION;

const GITHUB_API_URL: &str = "https://api.github.com/repos/tryigit/FOEM/releases/latest";
const RELEASES_URL: &str = "https://github.com/tryigit/FOEM/releases";

#[derive(Debug, PartialEq)]
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

    fn fetch_release_string(&self) -> Result<String, String> {
        let response = ureq::get(GITHUB_API_URL)
            .set("Accept", "application/vnd.github.v3+json")
            .set("User-Agent", "FOEM-UpdateChecker")
            .call()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        response.into_string().map_err(|e| format!("Failed to read response: {}", e))
    }

    /// Query the GitHub API for the latest release.
    ///
    /// Returns `Ok(Some(info))` when a newer version is available,
    /// `Ok(None)` when the current version is already the latest,
    /// or `Err` on network / parse failures.
    pub fn check_for_updates(&self) -> Result<Option<UpdateInfo>, String> {
        let json_str = {
            #[cfg(test)]
            {
                let mut mock_result = None;
                tests::MOCK_HTTP_RESPONSE.with(|mock| {
                    if let Some(res) = mock.borrow().as_ref() {
                        mock_result = Some(res.clone());
                    }
                });
                if let Some(res) = mock_result {
                    res?
                } else {
                    self.fetch_release_string()?
                }
            }
            #[cfg(not(test))]
            {
                self.fetch_release_string()?
            }
        };

        let release: GithubRelease = serde_json::from_str(&json_str)
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
pub mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        pub static MOCK_HTTP_RESPONSE: RefCell<Option<Result<String, String>>> = const { RefCell::new(None) };
    }

    #[test]
    fn test_new() {
        let _manager = UpdateManager::new();
    }

    #[test]
    fn test_update_available() {
        MOCK_HTTP_RESPONSE.with(|mock| {
            *mock.borrow_mut() = Some(Ok(r#"{
                    "tag_name": "v9.9.9",
                    "html_url": "https://github.com/tryigit/FOEM/releases/tag/v9.9.9"
                }"#.to_string()));
        });

        let manager = UpdateManager::new();
        let result = manager.check_for_updates().unwrap();
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.latest_version, "9.9.9");
        assert_eq!(info.download_url, "https://github.com/tryigit/FOEM/releases/tag/v9.9.9");

        MOCK_HTTP_RESPONSE.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }

    #[test]
    fn test_no_update_available() {
        MOCK_HTTP_RESPONSE.with(|mock| {
            let current_version = crate::VERSION;
            *mock.borrow_mut() = Some(Ok(format!(r#"{{
                    "tag_name": "v{}",
                    "html_url": "https://github.com/tryigit/FOEM/releases/tag/v{}"
                }}"#, current_version, current_version)));
        });

        let manager = UpdateManager::new();
        let result = manager.check_for_updates().unwrap();
        assert!(result.is_none());

        MOCK_HTTP_RESPONSE.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }

    #[test]
    fn test_network_error() {
        MOCK_HTTP_RESPONSE.with(|mock| {
            *mock.borrow_mut() = Some(Err("Network failure".to_string()));
        });

        let manager = UpdateManager::new();
        let result = manager.check_for_updates();
        assert!(result.is_err());

        MOCK_HTTP_RESPONSE.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }

    #[test]
    fn test_parse_error() {
        MOCK_HTTP_RESPONSE.with(|mock| {
            *mock.borrow_mut() = Some(Ok("invalid json".to_string()));
        });

        let manager = UpdateManager::new();
        let result = manager.check_for_updates();
        assert!(result.is_err());

        MOCK_HTTP_RESPONSE.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }
}
