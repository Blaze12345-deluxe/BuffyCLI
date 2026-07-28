use crate::error::{BuffyError, Result};

/// Updates all installed packages.
pub fn update_all() -> Result<()> {
    let installed = crate::config::settings::read_installed()?;
    let mut updated = 0u64;
    let mut already_current = 0u64;
    let mut failed = 0u64;

    for entry in &installed {
        if entry.source == "local" {
            continue;
        }
        match update_single(entry) {
            Ok(true) => updated += 1,
            Ok(false) => already_current += 1,
            Err(e) => {
                eprintln!("Failed to update {}: {}", entry.name, e);
                failed += 1;
            }
        }
    }

    if updated > 0 {
        println!("Updated {} package(s).", updated);
    }
    if already_current > 0 {
        println!("{} package(s) already up to date.", already_current);
    }
    if failed > 0 {
        println!("{} package(s) failed to update.", failed);
    }

    if updated == 0 && already_current == 0 && failed == 0 {
        println!("All packages up to date.");
    }

    Ok(())
}

/// Updates a single package by name.
pub fn update_one(name: &str) -> Result<()> {
    let installed = crate::config::settings::read_installed()?;
    let entry = installed.iter().find(|e| e.name == name).ok_or_else(|| {
        BuffyError::PackageNotFound {
            name: name.to_string(),
        }
    })?;

    if entry.source == "local" {
        return Err(BuffyError::PackageNotFound {
            name: format!("{} (installed from local file, cannot auto-update)", name),
        });
    }

    match update_single(entry)? {
        true => println!("Updated {}.", name),
        false => println!("{} is already up to date.", name),
    }

    Ok(())
}

/// Attempts to update a single package.
/// Returns Ok(true) if updated, Ok(false) if already current.
fn update_single(
    entry: &crate::config::settings::InstalledEntry,
) -> Result<bool> {

    // Determine the repository URL from the source
    let repo_url = resolve_source_url(entry)?;

    // Check for newer version in the index
    let (owner, repo) = crate::repository::source::parse_github_url(&repo_url)
        .ok_or_else(|| BuffyError::RepositoryConnection {
            url: repo_url.clone(),
            detail: "Invalid repository URL".to_string(),
        })?;

    let index = crate::repository::github::fetch_index(&owner, &repo)?;
    let entry_in_index = index.find_package(&entry.name).ok_or_else(|| {
        BuffyError::PackageNotFound {
            name: entry.name.clone(),
        }
    })?;

    // Compare versions
    use crate::repository::index::compare_versions;
    if compare_versions(&entry_in_index.version, &entry.version) != std::cmp::Ordering::Greater {
        return Ok(false);
    }

    println!("Updating {}: {} -> {}", entry.name, entry.version, entry_in_index.version);

    // Reinstall via the install pipeline
    let install_args = vec![repo_url.clone(), entry.name.clone()];
    crate::package::install::install(&install_args)?;

    // Update dependencies if the manifest changed
    let pkg_dir = crate::config::buffy_home::commands_dir().join(&entry.name);
    if pkg_dir.exists() {
        if let Ok(manifest) = crate::package::manifest::validate(&pkg_dir) {
            // Check if the new version has different BSL dependencies
            let old_deps: std::collections::HashSet<String> = entry.dependencies.packages.iter().cloned().collect();
            let new_deps: std::collections::HashSet<String> = manifest.dependencies.packages.iter().cloned().collect();

            // New dependencies that weren't there before
            for dep in new_deps.difference(&old_deps) {
                if !crate::package::deps::is_package_installed(dep) {
                    println!("  New dependency detected: {}", dep);
                    println!("  Installing {}...", dep);
                    let dep_args = vec![dep.clone()];
                    if let Err(e) = crate::package::install::install(&dep_args) {
                        eprintln!("  Failed to install dependency {}: {}", dep, e);
                    }
                }
            }

            // Update installed.json with new version info and deps
            let combined_hash = manifest.combined_hash();
            let mut installed = crate::config::settings::read_installed()?;
            if let Some(existing) = installed.iter_mut().find(|e| e.name == entry.name) {
                existing.version = manifest.version.clone();
                existing.installed = chrono::Local::now().format("%Y-%m-%d").to_string();
                existing.dependencies = crate::config::settings::Dependencies {
                    system: manifest.dependencies.system.clone(),
                    packages: manifest.dependencies.packages.clone(),
                };
                existing.sha256 = combined_hash;
            }
            crate::config::settings::write_installed(&installed)?;
        }
    }

    Ok(true)
}

/// Resolves an installed entry's source field to a full repository URL.
fn resolve_source_url(entry: &crate::config::settings::InstalledEntry) -> Result<String> {
    match entry.source.as_str() {
        "official" | "" => {
            let repos = crate::config::settings::read_repositories()?;
            repos.first().cloned().ok_or_else(|| BuffyError::RepositoryConnection {
                url: "default".to_string(),
                detail: "No repositories configured".to_string(),
            })
        }
        "local" => Err(BuffyError::RepositoryConnection {
            url: "local".to_string(),
            detail: "Cannot update packages installed from local files".to_string(),
        }),
        s if s.starts_with("github.com/") => Ok(format!("https://{}", s)),
        s if s.starts_with("https://") => Ok(s.to_string()),
        _ => Err(BuffyError::RepositoryConnection {
            url: entry.source.clone(),
            detail: "Unknown package source".to_string(),
        }),
    }
}
