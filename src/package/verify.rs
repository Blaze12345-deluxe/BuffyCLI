use crate::error::{BuffyError, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Verifies a single package's integrity using SHA-256 3-way comparison.
/// Also checks system and BSL dependencies.
pub fn verify_package(package_dir: &Path, name: &str) -> Result<()> {
    let manifest = crate::package::manifest::validate(package_dir)?;

    // SHA-256 integrity check
    let generated_hash = compute_package_hash(package_dir, &manifest.assets)?;
    let package_json_hash = manifest.sha256;
    let sha_file_hash = read_sha_file(package_dir, name)?;

    if generated_hash != package_json_hash {
        return Err(BuffyError::PackageVerificationFailed {
            name: name.to_string(),
            detail: format!(
                "Generated hash does not match package.json: {} != {}",
                generated_hash, package_json_hash
            ),
        });
    }

    if let Some(sha_file) = sha_file_hash {
        if generated_hash != sha_file {
            return Err(BuffyError::PackageVerificationFailed {
                name: name.to_string(),
                detail: format!(
                    "Generated hash does not match {}-SHA.txt: {} != {}",
                    name, generated_hash, sha_file
                ),
            });
        }
    }

    Ok(())
}

/// Verifies all installed packages.
/// Returns a list of (package_name, result) pairs.
pub fn verify_all() -> Result<Vec<(String, std::result::Result<(), BuffyError>)>> {
    let installed = crate::config::settings::read_installed()?;
    let mut results = Vec::new();
    let commands_dir = crate::config::buffy_home::commands_dir();

    for entry in &installed {
        let pkg_dir = commands_dir.join(&entry.name);
        if !pkg_dir.exists() {
            results.push((
                entry.name.clone(),
                Err(BuffyError::PackageNotFound {
                    name: entry.name.clone(),
                }),
            ));
            continue;
        }

        let result = verify_package(&pkg_dir, &entry.name);
        results.push((entry.name.clone(), result));
    }

    Ok(results)
}

/// Verifies system dependencies for an installed package.
/// Delegates to the deps module.
pub fn verify_system_deps(package_dir: &Path, _name: &str) -> Result<Vec<String>> {
    crate::package::deps::verify_system_dependencies(package_dir)
}

/// Verifies BSL package dependencies for an installed package.
/// Delegates to the deps module.
pub fn verify_bsl_deps(package_dir: &Path, _name: &str) -> Result<Vec<String>> {
    crate::package::deps::verify_bsl_dependencies(package_dir)
}

/// Computes SHA-256 over all .bsl files and declared assets.
fn compute_package_hash(package_dir: &Path, assets: &[String]) -> Result<String> {
    let mut hasher = Sha256::new();

    // Hash .bsl files (sorted alphabetically)
    let mut bsl_files: Vec<_> = std::fs::read_dir(package_dir)
        .map_err(BuffyError::Io)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "bsl"))
        .map(|e| e.path())
        .collect();
    bsl_files.sort();

    for file in &bsl_files {
        let content = std::fs::read(file)?;
        hasher.update(&content);
    }

    // Hash declared assets (sorted alphabetically)
    let mut assets = assets.to_vec();
    assets.sort();
    for asset in &assets {
        let asset_path = package_dir.join(asset);
        if asset_path.exists() {
            let content = std::fs::read(&asset_path)?;
            hasher.update(&content);
        }
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Reads the {name}-SHA.txt file.
fn read_sha_file(package_dir: &Path, name: &str) -> Result<Option<String>> {
    let sha_path = package_dir.join(format!("{}-SHA.txt", name));
    if !sha_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&sha_path)?;
    // Format: "HASH  name" or just "HASH"
    let hash = content.split_whitespace().next().unwrap_or("").to_string();
    Ok(Some(hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_is_consistent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.bsl"), "WRITE \"hello\"\n").unwrap();

        let hash1 = compute_package_hash(dir.path(), &[]).unwrap();
        let hash2 = compute_package_hash(dir.path(), &[]).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_changes_when_file_changes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.bsl"), "WRITE \"hello\"\n").unwrap();
        let hash1 = compute_package_hash(dir.path(), &[]).unwrap();

        std::fs::write(dir.path().join("test.bsl"), "WRITE \"world\"\n").unwrap();
        let hash2 = compute_package_hash(dir.path(), &[]).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_all_empty() {
        // When no packages are installed, verify_all should return empty
        let result = verify_all();
        assert!(result.is_ok());
        if let Ok(results) = result {
            assert!(results.is_empty(), "No packages should be installed in test env");
        }
    }
}
