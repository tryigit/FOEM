🎯 What
This PR fixes a security vulnerability where the GitHub API and releases URLs were hardcoded as constants (`GITHUB_API_URL` and `RELEASES_URL`). They have been removed and the `UpdateManager` has been modified to support dynamically configuring these URLs via environment variables (`FOEM_UPDATE_API_URL` and `FOEM_UPDATE_RELEASES_URL`), while falling back to the previous URLs if none are specified. A compilation error in `src/features/tools.rs` has also been fixed by properly importing `std::fmt::Write`.

⚠️ Risk
Hardcoding update URLs limits flexibility and makes revocation/rotation harder in the event of a compromised repository, DNS hijacking, or moving to a different release hosting provider. The application would be unable to point to a new, secure update server without issuing a new binary update (which wouldn't be accessible from the compromised URL).

🛡️ Solution
The `UpdateManager` struct was modified to store `api_url` and `releases_url` as fields instead of relying on global constants. The constructor `UpdateManager::new()` reads these URLs from the `FOEM_UPDATE_API_URL` and `FOEM_UPDATE_RELEASES_URL` environment variables, using the original GitHub URLs as fallbacks to preserve existing functionality out-of-the-box. The `fetch_release_string` and `check_for_updates` methods were updated to use these new fields.
