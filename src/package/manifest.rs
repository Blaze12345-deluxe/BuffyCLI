use crate::error::{BuffyError, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    #[serde(default)]
    pub sha256: String,
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
    let sha256 = format!("{:x}", content);

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
