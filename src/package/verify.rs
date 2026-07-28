use crate::error::{BuffyError, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Verifies a single package's integrity using SHA-256 per-file comparison.
/// Each .bsl file in the package is hashed individually and compared against
/// both the package.json and SHA.txt entries.
pub fn verify_package(package_dir: &Path, name: &str) -> Result<()> {
    let manifest = crate::package::manifest::validate(package_dir)?;

    // Compute per-file hashes for all .bsl files
    let generated_hashes = compute_file_hashes(package_dir)?;

    // Read per-file hashes from SHA.txt
    let sha_file_hashes = read_sha_file(package_dir, name)?;

    // Determine whether we're using per-file or legacy combined verification
    let is_per_file = !generated_hashes.is_empty()
        && manifest.is_per_file()
        && sha_file_hashes.as_ref().map_or(false, |h| h.len() > 1 || (h.len() == 1 && !h.contains_key("_combined")));

    if is_per_file {
        // Per-file verification: check each .bsl file individually
        for (filename, computed_hash) in &generated_hashes {
            // Check against package.json
            let manifest_hash = manifest.get_file_hash(filename).ok_or_else(|| {
                BuffyError::PackageVerificationFailed {
                    name: name.to_string(),
                    detail: format!(
                        "File {} not found in package.json sha256 entries",
                        filename
                    ),
                }
            })?;

            if computed_hash != manifest_hash {
                return Err(BuffyError::PackageVerificationFailed {
                    name: name.to_string(),
                    detail: format!(
                        "Generated hash for {} does not match package.json: {} != {}",
                        filename, computed_hash, manifest_hash
                    ),
                });
            }

            // Check against SHA.txt
            if let Some(ref sha_hashes) = sha_file_hashes {
                let expected_hash = sha_hashes.get(filename).ok_or_else(|| {
                    BuffyError::PackageVerificationFailed {
                        name: name.to_string(),
                        detail: format!(
                            "File {} not found in {}-SHA.txt",
                            filename, name
                        ),
                    }
                })?;

                if computed_hash != expected_hash {
                    return Err(BuffyError::PackageVerificationFailed {
                        name: name.to_string(),
                        detail: format!(
                            "Generated hash for {} does not match {}-SHA.txt: {} != {}",
                            filename, name, computed_hash, expected_hash
                        ),
                    });
                }
            }
        }
    } else {
        // Legacy combined-hash verification (backward compatible)
        let generated_hash = compute_combined_hash(package_dir)?;

        // Check against package.json (legacy combined format)
        if let Some(manifest_hash) = manifest.get_file_hash("_combined") {
            if generated_hash != manifest_hash {
                return Err(BuffyError::PackageVerificationFailed {
                    name: name.to_string(),
                    detail: format!(
                        "Generated hash does not match package.json: {} != {}",
                        generated_hash, manifest_hash
                    ),
                });
            }
        }

        // Check against SHA.txt (legacy single-line format)
        if let Some(ref sha_hashes) = sha_file_hashes {
            if let Some(sha_file_hash) = sha_hashes.get("_combined") {
                if generated_hash != *sha_file_hash {
                    return Err(BuffyError::PackageVerificationFailed {
                        name: name.to_string(),
                        detail: format!(
                            "Generated hash does not match {}-SHA.txt: {} != {}",
                            name, generated_hash, sha_file_hash
                        ),
                    });
                }
            }
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
pub fn verify_system_deps(package_dir: &Path, _name: &str) -> Result<Vec<String>> {
    crate::package::deps::verify_system_dependencies(package_dir)
}

/// Verifies BSL package dependencies for an installed package.
pub fn verify_bsl_deps(package_dir: &Path, _name: &str) -> Result<Vec<String>> {
    crate::package::deps::verify_bsl_dependencies(package_dir)
}

/// Computes per-file SHA-256 hashes for all .bsl files in the package.
/// Returns a map of filename -> hash. Excludes package.json and SHA.txt.
fn compute_file_hashes(package_dir: &Path) -> Result<HashMap<String, String>> {
    let mut hashes = HashMap::new();

    let mut bsl_files: Vec<_> = std::fs::read_dir(package_dir)
        .map_err(BuffyError::Io)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "bsl"))
        .map(|e| e.path())
        .collect();
    bsl_files.sort();

    for file in &bsl_files {
        let filename = file.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let content = std::fs::read(file)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());
        hashes.insert(filename, hash);
    }

    Ok(hashes)
}

/// Computes the legacy combined SHA-256 hash over all .bsl files and assets.
/// Used for backward compatibility with old package format.
fn compute_combined_hash(package_dir: &Path) -> Result<String> {
    let mut hasher = Sha256::new();

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

    Ok(format!("{:x}", hasher.finalize()))
}

/// Reads the {name}-SHA.txt file and returns per-file hashes.
/// Supports both:
///   1. Per-file format: "<hash>  filename.bsl" (one per line)
///   2. Legacy format:  "<hash>  name" (single line)
fn read_sha_file(package_dir: &Path, name: &str) -> Result<Option<HashMap<String, String>>> {
    let sha_path = package_dir.join(format!("{}-SHA.txt", name));
    if !sha_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&sha_path)?;
    let mut hashes = HashMap::new();
    let mut line_count = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Format: "<hash>  <filename>"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let hash = parts[0].to_string();
            let identifier = parts[1..].join(" ");
            // If the identifier doesn't end with .bsl, treat as legacy combined hash
            if identifier.ends_with(".bsl") {
                hashes.insert(identifier, hash);
            } else {
                hashes.insert("_combined".to_string(), hash);
            }
            line_count += 1;
        } else if parts.len() == 1 {
            // Just a bare hash with no filename
            hashes.insert("_combined".to_string(), parts[0].to_string());
            line_count += 1;
        }
    }

    if line_count == 1 && hashes.contains_key("_combined") {
        // Legacy single-line format: return as-is with _combined key
        Ok(Some(hashes))
    } else if line_count > 0 {
        // Per-file format or single-entry named format
        Ok(Some(hashes))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_file_hashes_consistent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.bsl"), "WRITE \"hello\"\n").unwrap();

        let hashes1 = compute_file_hashes(dir.path()).unwrap();
        let hashes2 = compute_file_hashes(dir.path()).unwrap();

        assert_eq!(hashes1, hashes2);
        assert!(hashes1.contains_key("test.bsl"));
    }

    #[test]
    fn test_hash_changes_when_file_changes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.bsl"), "WRITE \"hello\"\n").unwrap();
        let hashes1 = compute_file_hashes(dir.path()).unwrap();

        std::fs::write(dir.path().join("test.bsl"), "WRITE \"world\"\n").unwrap();
        let hashes2 = compute_file_hashes(dir.path()).unwrap();

        assert_ne!(hashes1.get("test.bsl"), hashes2.get("test.bsl"));
    }

    #[test]
    fn test_multiple_files_have_separate_hashes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.bsl"), "WRITE \"a\"\n").unwrap();
        std::fs::write(dir.path().join("b.bsl"), "WRITE \"b\"\n").unwrap();

        let hashes = compute_file_hashes(dir.path()).unwrap();
        assert_eq!(hashes.len(), 2);
        assert_ne!(hashes.get("a.bsl"), hashes.get("b.bsl"));
    }

    #[test]
    fn test_read_sha_file_per_file_format() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("test-SHA.txt"),
            "abc123  a.bsl\ndef456  b.bsl\n",
        ).unwrap();

        let result = read_sha_file(dir.path(), "test").unwrap().unwrap();
        assert_eq!(result.get("a.bsl").unwrap(), "abc123");
        assert_eq!(result.get("b.bsl").unwrap(), "def456");
    }

    #[test]
    fn test_read_sha_file_legacy_format() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("test-SHA.txt"),
            "abc123  test\n",
        ).unwrap();

        let result = read_sha_file(dir.path(), "test").unwrap().unwrap();
        assert_eq!(result.get("_combined").unwrap(), "abc123");
    }

    #[test]
    fn test_verify_all_empty() {
        let result = verify_all();
        assert!(result.is_ok());
        if let Ok(results) = result {
            assert!(results.is_empty());
        }
    }
}
