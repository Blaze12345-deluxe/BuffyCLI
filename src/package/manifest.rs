use crate::error::{BuffyError, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    /// Per-file SHA-256 hashes keyed by .bsl filename.
    /// Can also be a single combined hash string for legacy compatibility.
    #[serde(default, deserialize_with = "deserialize_sha256")]
    pub sha256: HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub dependencies: PackageDependencies,
    #[serde(default)]
    pub assets: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub homepage: String,
}

/// Wrapper enum for backward-compatible sha256 deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sha256Format {
    PerFile(HashMap<String, String>),
    Legacy(String),
}

/// Deserializes sha256 which can be either a single hash string (legacy)
/// or an object mapping filenames to hashes.
fn deserialize_sha256<'de, D>(deserializer: D) -> std::result::Result<HashMap<String, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let format = Sha256Format::deserialize(deserializer)?;
    match format {
        Sha256Format::PerFile(map) => Ok(map),
        Sha256Format::Legacy(h) => {
            let mut map = HashMap::new();
            map.insert("_combined".to_string(), h);
            Ok(map)
        }
    }
}

impl PackageManifest {
    /// Returns the hash for a specific .bsl file, or the combined legacy hash.
    pub fn get_file_hash(&self, filename: &str) -> Option<&str> {
        // Try per-file first
        if let Some(h) = self.sha256.get(filename) {
            return Some(h.as_str());
        }
        // Try combined legacy hash
        if filename.ends_with(".bsl") && self.sha256.contains_key("_combined") {
            return self.sha256.get("_combined").map(|s| s.as_str());
        }
        // Fallback: return any single-entry value if there's only one
        if self.sha256.len() == 1 && !self.sha256.contains_key("_combined") {
            return self.sha256.values().next().map(|s| s.as_str());
        }
        None
    }

    /// Returns true if this manifest uses per-file hashes.
    pub fn is_per_file(&self) -> bool {
        !self.sha256.contains_key("_combined") && !self.sha256.is_empty()
    }

    /// Returns a deterministic combined hash string from per-file hashes.
    /// Used for conflict detection and storage in InstalledEntry.
    pub fn combined_hash(&self) -> String {
        // If legacy combined format, return it directly
        if let Some(h) = self.sha256.get("_combined") {
            return h.clone();
        }
        // Sort keys for deterministic output, then concatenate all hashes
        let mut keys: Vec<&String> = self.sha256.keys().collect();
        keys.sort();
        let combined = keys.iter()
            .filter_map(|k| self.sha256.get(*k))
            .cloned()
            .collect::<Vec<_>>()
            .join("");
        // Hash the concatenated hashes for a fixed-length combined hash
        let hash = sha2::Sha256::digest(combined.as_bytes());
        format!("{:x}", hash)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PackageDependencies {
    #[serde(default)]
    pub system: Vec<String>,
    #[serde(default)]
    pub packages: Vec<String>,
}

/// Validates a package manifest from a package directory.
pub fn validate(package_dir: &Path) -> Result<PackageManifest> {
    let manifest_path = package_dir.join("package.json");
    if !manifest_path.exists() {
        return Err(BuffyError::InvalidManifest {
            path: package_dir.to_string_lossy().to_string(),
            detail: "package.json not found".to_string(),
        });
    }

    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: PackageManifest = serde_json::from_str(&content)?;

    // Validate required fields
    if manifest.name.is_empty() {
        return Err(BuffyError::InvalidManifest {
            path: manifest_path.to_string_lossy().to_string(),
            detail: "Package name is required".to_string(),
        });
    }
    if manifest.version.is_empty() {
        return Err(BuffyError::InvalidManifest {
            path: manifest_path.to_string_lossy().to_string(),
            detail: "Package version is required".to_string(),
        });
    }

    // Check that at least one .bsl file exists
    let has_bsl = std::fs::read_dir(package_dir)
        .map(|entries| {
            entries.filter_map(|e| e.ok())
                .any(|e| e.path().extension().map_or(false, |ext| ext == "bsl"))
        })
        .unwrap_or(false);

    if !has_bsl {
        return Err(BuffyError::InvalidManifest {
            path: package_dir.to_string_lossy().to_string(),
            detail: "No .bsl files found in package".to_string(),
        });
    }

    Ok(manifest)
}

/// Generates a minimal manifest from a standalone .bsl file.
pub fn generate_from_bsl(name: &str, bsl_path: &Path) -> Result<PackageManifest> {
    let content = sha2::Sha256::digest(std::fs::read(bsl_path)?);
    let bsl_filename = bsl_path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    let hash = format!("{:x}", content);
    let mut sha256 = HashMap::new();
    sha256.insert(bsl_filename, hash);

    Ok(PackageManifest {
        name: name.to_string(),
        version: chrono::Local::now().format("%Y.%m.%d").to_string(),
        description: format!("Local BSL script installed from {}", bsl_path.display()),
        author: "unknown".to_string(),
        sha256,
        tags: vec![],
        dependencies: PackageDependencies::default(),
        assets: vec![],
        license: String::new(),
        homepage: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = PackageManifest {
            name: "test-pkg".to_string(),
            version: "2026.01.01".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        std::fs::write(dir.path().join("package.json"), &json).unwrap();
        std::fs::write(dir.path().join("test.bsl"), "WRITE \"hello\"\nEXIT").unwrap();

        let result = validate(dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_missing_package_json() {
        let dir = tempfile::tempdir().unwrap();
        let result = validate(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_bsl_files() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = PackageManifest {
            name: "test-pkg".to_string(),
            version: "2026.01.01".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        std::fs::write(dir.path().join("package.json"), &json).unwrap();
        // No .bsl files

        let result = validate(dir.path());
        assert!(result.is_err());
    }
}
