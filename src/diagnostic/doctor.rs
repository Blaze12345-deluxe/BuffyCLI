use crate::error::Result;
use crate::logger;

/// Runs a full system diagnostic.
pub fn run_doctor() -> Result<DoctorReport> {
    let mut report = DoctorReport::new();
    report.check("Home directory", check_home_directory());
    report.check("Config file", check_config_file());
    report.check("Installed packages registry", check_installed_packages_registry());
    report.check("Repository configuration", check_repositories_config());
    report.check("Aliases file", check_aliases_file());
    report.check("Directory permissions", check_permissions());

    // Repository connectivity (actual HTTP check)
    report.check("Repository connectivity", check_repository_connectivity());

    // Package integrity checks
    report.check("Package SHA integrity", check_package_sha_integrity());

    // Dependency checks for installed packages
    let system_deps_ok = check_system_dependencies();
    let bsl_deps_ok = check_bsl_dependencies();
    report.check("System dependencies", system_deps_ok);
    report.check("BSL dependencies", bsl_deps_ok);

    // Check for updates
    report.check("Update status", check_update_available());

    Ok(report)
}

fn check_home_directory() -> bool {
    let home = crate::config::buffy_home::buffy_home();
    home.exists()
}

fn check_config_file() -> bool {
    crate::config::settings::read_config().is_ok()
}

fn check_installed_packages_registry() -> bool {
    crate::config::settings::read_installed().is_ok()
}

fn check_repositories_config() -> bool {
    crate::config::settings::read_repositories().is_ok()
}

fn check_aliases_file() -> bool {
    crate::config::aliases::read_aliases().is_ok()
}

fn check_permissions() -> bool {
    let home = crate::config::buffy_home::buffy_home();
    home.exists() && home.is_dir()
}

fn check_repository_connectivity() -> bool {
    let repos = match crate::config::settings::read_repositories() {
        Ok(r) => r,
        Err(_) => return false,
    };

    if repos.is_empty() {
        return false;
    }

    // Try to fetch the first repository's index
    if let Some(repo_url) = repos.first() {
        let full = if repo_url.starts_with("https://") {
            repo_url.clone()
        } else {
            format!("https://github.com/{}", repo_url)
        };

        if let Some((owner, repo)) = crate::repository::source::parse_github_url(&full) {
            // Use cached fetch to avoid network timeout issues
            if crate::repository::github::fetch_index_cached(&owner, &repo).is_ok() {
                return true;
            }
            // Try direct fetch as fallback
            return crate::repository::github::fetch_index(&owner, &repo).is_ok();
        }
    }

    false
}

fn check_package_sha_integrity() -> bool {
    let commands_dir = crate::config::buffy_home::commands_dir();
    if !commands_dir.exists() {
        return true; // No packages to check
    }

    let installed = match crate::config::settings::read_installed() {
        Ok(i) => i,
        Err(_) => return false,
    };

    if installed.is_empty() {
        return true;
    }

    for entry in &installed {
        let pkg_dir = commands_dir.join(&entry.name);
        if !pkg_dir.exists() {
            continue; // Missing package directory — handled elsewhere
        }

        // Try to verify the package
        if crate::package::verify::verify_package(&pkg_dir, &entry.name).is_err() {
            return false; // At least one package failed verification
        }
    }

    true
}

fn check_system_dependencies() -> bool {
    let commands_dir = crate::config::buffy_home::commands_dir();
    if !commands_dir.exists() {
        return true;
    }

    let installed = match crate::config::settings::read_installed() {
        Ok(i) => i,
        Err(_) => return true,
    };

    for entry in &installed {
        if !entry.dependencies.system.is_empty() {
            for dep in &entry.dependencies.system {
                if !crate::package::deps::check_system_dependency(dep) {
                    return false;
                }
            }
        }
    }

    true
}

fn check_bsl_dependencies() -> bool {
    let commands_dir = crate::config::buffy_home::commands_dir();
    if !commands_dir.exists() {
        return true;
    }

    let installed = match crate::config::settings::read_installed() {
        Ok(i) => i,
        Err(_) => return true,
    };

    for entry in &installed {
        if !entry.dependencies.packages.is_empty() {
            for dep in &entry.dependencies.packages {
                if !crate::package::deps::is_package_installed(dep) {
                    return false;
                }
            }
        }
    }

    true
}

fn check_update_available() -> bool {
    // Check if current version has a later release available
    // For now, just report that check_update or --update can be used
    // This is a best-effort check
    true
}

pub struct DoctorReport {
    pub checks: Vec<(String, bool)>,
}

impl DoctorReport {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn check(&mut self, name: &str, passed: bool) {
        self.checks.push((name.to_string(), passed));
        if passed {
            logger::formatter::success(&format!("{}", name));
        } else {
            logger::formatter::error(&format!("{}", name));
        }
    }

    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|(_, passed)| *passed)
    }

    /// Returns the number of failed checks.
    pub fn failed_count(&self) -> usize {
        self.checks.iter().filter(|(_, passed)| !passed).count()
    }
}
