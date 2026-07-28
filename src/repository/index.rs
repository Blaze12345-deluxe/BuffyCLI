use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepositoryIndex {
    pub packages: Vec<PackageEntry>,
    #[serde(default)]
    pub meta: IndexMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub path: String,
    #[serde(default)]
    pub dependencies: DependenciesInfo,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DependenciesInfo {
    #[serde(default)]
    pub system: Vec<String>,
    #[serde(default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct IndexMeta {
    #[serde(default)]
    pub updated: String,
    #[serde(default)]
    pub package_count: u64,
}

impl RepositoryIndex {
    /// Finds a package by name in the index.
    pub fn find_package(&self, name: &str) -> Option<&PackageEntry> {
        self.packages.iter().find(|p| p.name == name)
    }
}

/// Compares two date-based version strings (YYYY.MM.DD).
/// Returns Ordering::Greater if v1 > v2.
pub fn compare_versions(v1: &str, v2: &str) -> std::cmp::Ordering {
    let parts1: Vec<u64> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
    let parts2: Vec<u64> = v2.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..parts1.len().max(parts2.len()) {
        let p1 = parts1.get(i).copied().unwrap_or(0);
        let p2 = parts2.get(i).copied().unwrap_or(0);
        if p1 != p2 {
            return p1.cmp(&p2);
        }
    }
    std::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        use std::cmp::Ordering;
        assert_eq!(compare_versions("2026.07.27", "2026.07.26"), Ordering::Greater);
        assert_eq!(compare_versions("2026.07.26", "2026.07.27"), Ordering::Less);
        assert_eq!(compare_versions("2026.07.27", "2026.07.27"), Ordering::Equal);
        assert_eq!(compare_versions("2026.07.27", "2025.12.25"), Ordering::Greater);
        assert_eq!(compare_versions("2025.12.25", "2026.01.01"), Ordering::Less);
    }

    #[test]
    fn test_find_package() {
        let index = RepositoryIndex {
            packages: vec![
                PackageEntry {
                    name: "pip-env".to_string(),
                    version: "2026.07.27".to_string(),
                    description: "Test".to_string(),
                    author: "Test".to_string(),
                    path: "packages/pip-env".to_string(),
                    dependencies: DependenciesInfo::default(),
                    tags: vec![],
                    commands: vec!["pip-env create".to_string()],
                },
            ],
            meta: IndexMeta::default(),
        };

        assert!(index.find_package("pip-env").is_some());
        assert!(index.find_package("nonexistent").is_none());
    }
}
