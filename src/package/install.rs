use crate::error::{BuffyError, Result};
use crate::package::deps::{resolve_dependencies, DependencyReport};
use crate::logger;
use std::collections::HashSet;
use std::path::Path;

/// Installs a package from the given source arguments.
pub fn install(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }

    let first = &args[0];

    if first.starts_with("./") {
        install_local(first)
    } else if first.starts_with("github.com/") || first.starts_with("https://github.com/") {
        let packages = if args.len() > 1 && args[1] == "@" {
            install_all_from_github(first)?
        } else {
            install_from_github(first, &args[1..])?
        };
        println!("Installed {} package(s).", packages);
        Ok(())
    } else {
        let count = install_from_repositories(args)?;
        println!("Installed {} package(s).", count);
        Ok(())
    }
}

/// Installs a package from a local .bsl file.
fn install_local(path: &str) -> Result<()> {
    let bsl_path = Path::new(path);

    if !bsl_path.exists() {
        return Err(BuffyError::PackageNotFound {
            name: path.to_string(),
        });
    }
    if bsl_path.extension().map_or(true, |ext| ext != "bsl") {
        return Err(BuffyError::InvalidManifest {
            path: path.to_string(),
            detail: "File must have .bsl extension".to_string(),
        });
    }

    let name = bsl_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    println!("Installing \"{}\"...", name);

    // Generate manifest AND write it to the destination
    let manifest = crate::package::manifest::generate_from_bsl(&name, bsl_path)?;

    // Install to commands directory
    let dest_dir = crate::config::buffy_home::commands_dir().join(&name);
    std::fs::create_dir_all(&dest_dir)?;

    // Copy the .bsl file
    let dest_file = dest_dir.join(format!("{}.bsl", name));
    std::fs::copy(bsl_path, &dest_file)?;
    println!("  -> {}.bsl", name);

    // Write package.json so verify works later
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(dest_dir.join("package.json"), &manifest_json)?;
    println!("  -> package.json");

    // Register in installed.json
    let mut installed = crate::config::settings::read_installed()?;
    installed.push(crate::config::settings::InstalledEntry {
        name: name.clone(),
        version: manifest.version,
        installed: chrono::Local::now().format("%Y-%m-%d").to_string(),
        source: "local".to_string(),
        sha256: manifest.sha256,
        author: manifest.author,
        dependencies: crate::config::settings::Dependencies {
            system: manifest.dependencies.system,
            packages: manifest.dependencies.packages,
        },
    });
    crate::config::settings::write_installed(&installed)?;

    println!("Done.");
    println!("Package installed successfully.");
    Ok(())
}

/// Installs packages from a specific GitHub repository.
fn install_from_github(repo: &str, packages: &[String]) -> Result<usize> {
    let (owner, repo_name) = parse_github_repo(repo)?;
    let mut count = 0;

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(indicatif::ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"));

    for pkg_name in packages {
        pb.set_message(format!("Fetching {}...", pkg_name));
        pb.enable_steady_tick(std::time::Duration::from_millis(80));

        let index = crate::repository::github::fetch_index(&owner, &repo_name)?;
        let entry = index.find_package(pkg_name).ok_or_else(|| {
            BuffyError::PackageNotFound {
                name: pkg_name.clone(),
            }
        })?;

        // Check if already installed
        if crate::package::deps::is_package_installed(pkg_name) {
            println!("  Package \"{}\" is already installed. Skipping.", pkg_name);
            count += 1;
            continue;
        }

        // Download to temp directory
        let temp_dir = tempfile::tempdir()?;
        let pkg_base = &entry.path;

        // Download package.json
        let _ = crate::repository::github::download_file(
            &owner, &repo_name,
            &format!("{}/package.json", pkg_base),
            &temp_dir.path().join("package.json"),
        );

        // Download SHA file
        let _ = crate::repository::github::download_file(
            &owner, &repo_name,
            &format!("{}/{}-SHA.txt", pkg_base, pkg_name),
            &temp_dir.path().join(format!("{}-SHA.txt", pkg_name)),
        );

        // Download .bsl files
        for cmd in &entry.commands {
            let cmd_name = cmd.split_whitespace().last().unwrap_or(cmd);
            let _ = crate::repository::github::download_file(
                &owner, &repo_name,
                &format!("{}/{}.bsl", pkg_base, cmd_name),
                &temp_dir.path().join(format!("{}.bsl", cmd_name)),
            );
        }

        // Also try module.bsl as fallback
        let _ = crate::repository::github::download_file(
            &owner, &repo_name,
            &format!("{}/module.bsl", pkg_base),
            &temp_dir.path().join("module.bsl"),
        );

        pb.set_message(format!("Verifying {}...", pkg_name));

        // Verify package integrity
        if let Err(e) = crate::package::verify::verify_package(temp_dir.path(), pkg_name) {
            eprintln!("Verification failed: {}", e);
            eprintln!("Skipping {}.", pkg_name);
            continue;
        }

        // Read manifest for dependency resolution
        let manifest = crate::package::manifest::validate(temp_dir.path())?;

        // Check for name conflicts with already-installed packages
        if let Some((existing_sha, author, source)) = crate::config::aliases::detect_conflict(pkg_name, &manifest.sha256)? {
            // Check if user has a cached preference
            if let Some((preferred_sha, _preferred_source)) = crate::config::aliases::resolve_conflict(pkg_name)? {
                if preferred_sha == existing_sha {
                    println!("  Package \"{}\" has a conflict with existing installation.", pkg_name);
                    println!("  Using cached preference for existing version ({}...). Skipping.", &existing_sha[..12.min(existing_sha.len())]);
                    continue;
                } else if preferred_sha == manifest.sha256 {
                    println!("  Package \"{}\" has a conflict with existing installation.", pkg_name);
                    println!("  Using cached preference for new version. Overwriting...");
                }
            } else {
                // No cached preference - prompt user
                println!("  Conflicting package \"{}\" found:", pkg_name);
                println!("    1. Existing: {}... (from {}, author: {})", &existing_sha[..12.min(existing_sha.len())], source, author);
                println!("    2. New:      {}... (from github.com/{}/{})", &manifest.sha256[..12.min(manifest.sha256.len())], owner, repo_name);
                println!("  Enter choice (1/2) or 's' to skip:");

                // Read user choice from stdin
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_ok() {
                    match input.trim() {
                        "1" => {
                            println!("  Keeping existing version.");
                            // Cache the preference
                            crate::config::aliases::set_conflict_preference(pkg_name, &existing_sha, &source)?;
                            continue;
                        }
                        "2" => {
                            println!("  Installing new version.");
                            // Cache the preference
                            crate::config::aliases::set_conflict_preference(pkg_name, &manifest.sha256, &format!("github.com/{}/{}", owner, repo_name))?;
                            // Continue to install below
                        }
                        _ => {
                            println!("  Skipping.");
                            continue;
                        }
                    }
                } else {
                    eprintln!("  Could not read input. Skipping.");
                    continue;
                }
            }
        }

        // Resolve and install dependencies
        println!("  Resolving dependencies...");
        let mut visited = HashSet::new();
        match resolve_dependencies(&manifest.dependencies, pkg_name, &mut visited) {
            Ok(report) => {
                print_dependency_report(&report);
            }
            Err(e) => {
                eprintln!("  Dependency resolution failed: {}", e);
                eprintln!("  Package \"{}\" will be installed but dependencies may be missing.", pkg_name);
            }
        }

        pb.set_message(format!("Installing {}...", pkg_name));

        // Install to commands directory
        let dest_dir = crate::config::buffy_home::commands_dir().join(pkg_name);
        std::fs::create_dir_all(&dest_dir)?;

        // Copy all files from temp dir to commands dir
        if let Ok(entries) = std::fs::read_dir(temp_dir.path()) {
            for entry in entries.flatten() {
                let path = entry.path();
                let dest = dest_dir.join(entry.file_name());
                std::fs::copy(&path, &dest)?;
                logger::formatter::info(&format!("  -> {}", entry.file_name().to_string_lossy()));
            }
        }

        // Register in installed.json
        let mut installed = crate::config::settings::read_installed()?;
        installed.push(crate::config::settings::InstalledEntry {
            name: pkg_name.clone(),
            version: manifest.version,
            installed: chrono::Local::now().format("%Y-%m-%d").to_string(),
            source: format!("github.com/{}/{}", owner, repo_name),
            sha256: manifest.sha256,
            author: manifest.author,
            dependencies: crate::config::settings::Dependencies {
                system: manifest.dependencies.system,
                packages: manifest.dependencies.packages,
            },
        });
        crate::config::settings::write_installed(&installed)?;

        pb.suspend(|| {
            logger::formatter::success(&format!("Installed {}", pkg_name));
        });
        count += 1;
    }

    Ok(count)
}

/// Installs all packages from a GitHub repository.
fn install_all_from_github(repo: &str) -> Result<usize> {
    let (owner, repo_name) = parse_github_repo(repo)?;

    println!("Fetching repository index...");
    let index = crate::repository::github::fetch_index(&owner, &repo_name)?;

    let package_names: Vec<String> = index.packages.iter().map(|p| p.name.clone()).collect();
    println!("Found {} packages.", package_names.len());

    let count = install_from_github(repo, &package_names)?;
    Ok(count)
}

/// Installs packages from configured repositories.
fn install_from_repositories(packages: &[String]) -> Result<usize> {
    let repos = crate::config::settings::read_repositories()?;
    let mut count = 0;

    for pkg_name in packages {
        println!("Searching for \"{}\"...", pkg_name);
        let mut found = false;

        // Skip if already installed
        if crate::package::deps::is_package_installed(pkg_name) {
            println!("  Package \"{}\" is already installed.", pkg_name);
            count += 1;
            continue;
        }

        for repo_url in &repos {
            let full_repo = if repo_url.starts_with("https://") {
                repo_url.clone()
            } else {
                format!("https://github.com/{}", repo_url)
            };

            if let Ok(n) = install_from_github(&full_repo, &[pkg_name.clone()]) {
                count += n;
                found = true;
                break;
            }
        }

        if !found {
            eprintln!("Package \"{}\" not found in any repository.", pkg_name);
        }
    }

    Ok(count)
}

/// Parses a GitHub repo string into (owner, name). Uses shared utility.
fn parse_github_repo(repo: &str) -> Result<(String, String)> {
    crate::repository::source::parse_github_url(repo).ok_or_else(|| {
        BuffyError::RepositoryConnection {
            url: repo.to_string(),
            detail: "Invalid GitHub repository format. Expected: github.com/owner/repo".to_string(),
        }
    })
}

/// Prints the dependency resolution report in a user-friendly format.
fn print_dependency_report(report: &DependencyReport) {
    if !report.system_met.is_empty() {
        println!("    System dependencies met: {}", report.system_met.join(", "));
    }
    if !report.system_missing.is_empty() {
        println!("    {} Missing system dependencies (optional): {}", "⚠", report.system_missing.join(", "));
    }
    if !report.bsl_installed.is_empty() {
        println!("    Installed BSL dependencies: {}", report.bsl_installed.join(", "));
    }
    if !report.bsl_already_installed.is_empty() {
        println!("    BSL dependencies already satisfied: {}", report.bsl_already_installed.join(", "));
    }
}
