use crate::error::{BuffyError, Result};
use crate::repository::index::RepositoryIndex;
use std::path::{Path, PathBuf};

/// Fetches the repository index.json from a GitHub repository (always from network).
pub fn fetch_index(owner: &str, repo: &str) -> Result<RepositoryIndex> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/main/index.json",
        owner, repo
    );
    let response = ureq::get(&url).call().map_err(|e| BuffyError::RepositoryConnection {
        url: url.clone(),
        detail: format!("HTTP error: {}", e),
    })?;
    let index: RepositoryIndex =
        serde_json::from_reader(response.into_reader()).map_err(|e| BuffyError::RepositoryConnection {
            url,
            detail: format!("Invalid index JSON: {}", e),
        })?;
    Ok(index)
}

/// Fetches the repository index with local caching.
/// Tries the local cache first. Falls back to network if cache is missing.
pub fn fetch_index_cached(owner: &str, repo: &str) -> Result<RepositoryIndex> {
    let cache_path = cached_index_path(owner, repo);

    // Try cache first
    if cache_path.exists() {
        match std::fs::read_to_string(&cache_path) {
            Ok(content) => {
                if let Ok(index) = serde_json::from_str::<RepositoryIndex>(&content) {
                    return Ok(index);
                }
                // Corrupt cache — fall through to re-fetch
            }
            Err(_) => {} // Can't read cache — fall through
        }
    }

    // Fetch from network
    let index = fetch_index(owner, repo)?;

    // Write to cache
    if let Some(parent) = cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&index) {
        let _ = std::fs::write(&cache_path, &json);
    }

    Ok(index)
}

/// Forces a fresh fetch of a repository index, updating the cache.
pub fn refresh_index(owner: &str, repo: &str) -> Result<RepositoryIndex> {
    let cache_path = cached_index_path(owner, repo);

    // Remove stale cache
    let _ = std::fs::remove_file(&cache_path);

    // Fetch fresh
    let index = fetch_index(owner, repo)?;

    // Write to cache
    if let Some(parent) = cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&index) {
        let _ = std::fs::write(&cache_path, &json);
    }

    Ok(index)
}

/// Validates that a repository URL is reachable and returns a valid index.
/// Accepts formats: "github.com/owner/repo", "https://github.com/owner/repo"
pub fn validate_repository(url: &str) -> Result<()> {
    let (owner, repo) = crate::repository::source::parse_github_url(url)
        .ok_or_else(|| BuffyError::RepositoryConnection {
            url: url.to_string(),
            detail: "Invalid repository URL format. Expected: github.com/owner/repo or https://github.com/owner/repo".to_string(),
        })?;

    // Try to fetch the index to validate
    match fetch_index(&owner, &repo) {
        Ok(_) => Ok(()),
        Err(e) => Err(BuffyError::RepositoryConnection {
            url: url.to_string(),
            detail: format!("Cannot connect to repository: {}", e),
        }),
    }
}

/// Searches for packages matching a query across all configured repositories.
/// Returns (repository_url, package_entry) pairs.
pub fn search_across_repositories(query: &str) -> Result<Vec<(String, crate::repository::index::PackageEntry)>> {
    let repos = crate::config::settings::read_repositories()?;
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for repo_url in &repos {
        let full_repo = if repo_url.starts_with("https://") {
            repo_url.clone()
        } else {
            format!("https://github.com/{}", repo_url)
        };

        if let Some((owner, repo)) = crate::repository::source::parse_github_url(&full_repo) {
            if let Ok(index) = fetch_index_cached(&owner, &repo) {
                for pkg in &index.packages {
                    let name_match = pkg.name.to_lowercase().contains(&query_lower);
                    let desc_match = pkg.description.to_lowercase().contains(&query_lower);
                    let tag_match = pkg.tags.iter().any(|t| t.to_lowercase().contains(&query_lower));

                    if name_match || desc_match || tag_match {
                        results.push((full_repo.clone(), pkg.clone()));
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Finds a specific package by name across all configured repositories.
/// Returns the repository URL and package entry, or an error if not found.
pub fn find_across_repositories(name: &str) -> Result<(String, crate::repository::index::PackageEntry)> {
    let repos = crate::config::settings::read_repositories()?;

    for repo_url in &repos {
        let full_repo = if repo_url.starts_with("https://") {
            repo_url.clone()
        } else {
            format!("https://github.com/{}", repo_url)
        };

        if let Some((owner, repo)) = crate::repository::source::parse_github_url(&full_repo) {
            if let Ok(index) = fetch_index_cached(&owner, &repo) {
                if let Some(entry) = index.find_package(name) {
                    return Ok((full_repo, entry.clone()));
                }
            }
        }
    }

    // Try a fresh fetch (in case cache is stale or repo was added recently)
    for repo_url in &repos {
        let full_repo = if repo_url.starts_with("https://") {
            repo_url.clone()
        } else {
            format!("https://github.com/{}", repo_url)
        };

        if let Some((owner, repo)) = crate::repository::source::parse_github_url(&full_repo) {
            if let Ok(index) = refresh_index(&owner, &repo) {
                if let Some(entry) = index.find_package(name) {
                    return Ok((full_repo, entry.clone()));
                }
            }
        }
    }

    Err(BuffyError::PackageNotFound {
        name: name.to_string(),
    })
}

/// Downloads a specific package file from a GitHub repository.
pub fn download_file(owner: &str, repo: &str, file_path: &str, target: &Path) -> Result<()> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/main/{}",
        owner, repo, file_path
    );
    let response = ureq::get(&url).call().map_err(|e| BuffyError::RepositoryConnection {
        url: url.clone(),
        detail: format!("HTTP error: {}", e),
    })?;
    let mut file = std::fs::File::create(target)?;
    let mut reader = response.into_reader();
    std::io::copy(&mut reader, &mut file)?;
    Ok(())
}

/// Returns the cache path for a repository index.
fn cached_index_path(owner: &str, repo: &str) -> PathBuf {
    crate::config::buffy_home::cache_dir()
        .join("indexes")
        .join(owner)
        .join(repo)
        .join("index.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::index::PackageEntry;
    use std::sync::Mutex;

    // Serializes tests that modify HOME
    static HOME_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_home() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let buffy_home = tmp.path().join(".buffy");
        std::fs::create_dir_all(buffy_home.join("cache").join("indexes")).unwrap();
        std::env::set_var("HOME", tmp.path());
        (tmp, buffy_home)
    }

    #[test]
    fn test_cached_index_roundtrip() {
        let _lock = HOME_LOCK.lock().unwrap();
        let (_tmp, buffy_home) = with_temp_home();

        // Write a fake index to the cache
        let cache_path = buffy_home.join("cache").join("indexes").join("testowner").join("testrepo").join("index.json");
        std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();

        let index = RepositoryIndex {
            packages: vec![PackageEntry {
                name: "test-pkg".to_string(),
                version: "2026.01.01".to_string(),
                description: "Test".to_string(),
                author: "Test Author".to_string(),
                path: "packages/test-pkg".to_string(),
                dependencies: crate::repository::index::DependenciesInfo::default(),
                tags: vec!["test".to_string()],
                commands: vec!["test-pkg".to_string()],
            }],
            meta: crate::repository::index::IndexMeta::default(),
        };

        let json = serde_json::to_string_pretty(&index).unwrap();
        std::fs::write(&cache_path, &json).unwrap();

        // Read it back via cached_index_path
        let read_path = cached_index_path("testowner", "testrepo");
        assert!(read_path.exists());

        let content = std::fs::read_to_string(&read_path).unwrap();
        let parsed: RepositoryIndex = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.packages.len(), 1);
        assert_eq!(parsed.packages[0].name, "test-pkg");
    }

    #[test]
    fn test_cached_index_returns_from_cache() {
        let _lock = HOME_LOCK.lock().unwrap();
        let (_tmp, buffy_home) = with_temp_home();

        // Write a fake index to cache
        let cache_dir = buffy_home.join("cache").join("indexes").join("owner").join("repo");
        std::fs::create_dir_all(&cache_dir).unwrap();
        let index = RepositoryIndex {
            packages: vec![PackageEntry {
                name: "cached-pkg".to_string(),
                version: "2026.01.01".to_string(),
                description: "Cached".to_string(),
                author: "Author".to_string(),
                path: "pkg".to_string(),
                dependencies: crate::repository::index::DependenciesInfo::default(),
                tags: vec![],
                commands: vec!["cached-pkg".to_string()],
            }],
            meta: crate::repository::index::IndexMeta::default(),
        };
        let json = serde_json::to_string_pretty(&index).unwrap();
        std::fs::write(cache_dir.join("index.json"), &json).unwrap();

        // fetch_index_cached should read from cache (no network needed)
        let result = fetch_index_cached("owner", "repo");
        // Since it reads from cache, it should work without network
        assert!(result.is_ok());
        if let Ok(idx) = result {
            assert_eq!(idx.packages[0].name, "cached-pkg");
        }
    }

    #[test]
    fn test_cache_corrupt_revalidates() {
        let _lock = HOME_LOCK.lock().unwrap();
        let (_tmp, buffy_home) = with_temp_home();

        // Write corrupt data to cache
        let cache_dir = buffy_home.join("cache").join("indexes").join("owner").join("repo");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("index.json"), "this is not valid json").unwrap();

        // Since the repo doesn't exist, fetch_index_cached should return an error
        // (it tried the cache, found corrupt data, tried network, network also failed)
        let result = fetch_index_cached("owner", "repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_search_across_repositories_empty() {
        let _lock = HOME_LOCK.lock().unwrap();
        let (_tmp, _) = with_temp_home();

        // No repos configured (default is Blaze12345-deluxe/Buffy-Plugins, which doesn't exist in test)
        // Since the default repo is unreachable, search should return empty results
        let results = search_across_repositories("test");
        assert!(results.is_ok());
        // Results might be empty since no network is available in tests
    }

    #[test]
    fn test_find_across_repositories_not_found() {
        let _lock = HOME_LOCK.lock().unwrap();
        let (_tmp, _) = with_temp_home();

        // Package doesn't exist in any reachable repo
        let result = find_across_repositories("nonexistent-package-xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_repository_invalid_url() {
        let result = validate_repository("not-a-repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_repository_unreachable() {
        let result = validate_repository("github.com/thisdefinitelydoesnotexist123/packages");
        assert!(result.is_err());
    }
}
