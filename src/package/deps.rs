use crate::error::{BuffyError, Result};
use crate::package::manifest::PackageDependencies;
use std::collections::HashSet;

/// Result of dependency resolution.
pub struct DependencyReport {
    /// System dependencies that are met (found on $PATH).
    pub system_met: Vec<String>,
    /// System dependencies that are missing (not found on $PATH).
    pub system_missing: Vec<String>,
    /// BSL package dependencies that were installed.
    pub bsl_installed: Vec<String>,
    /// BSL package dependencies that were already installed.
    pub bsl_already_installed: Vec<String>,
}

impl DependencyReport {
    fn new() -> Self {
        Self {
            system_met: Vec::new(),
            system_missing: Vec::new(),
            bsl_installed: Vec::new(),
            bsl_already_installed: Vec::new(),
        }
    }

    /// Returns true if all system dependencies are met.
    pub fn all_system_met(&self) -> bool {
        self.system_missing.is_empty()
    }
}

/// Resolves all dependencies for a package before installation.
/// Checks system deps on $PATH and installs BSL package deps.
///
/// `visited` tracks packages already processed in this dependency chain
/// to detect circular dependencies.
pub fn resolve_dependencies(
    deps: &PackageDependencies,
    package_name: &str,
    visited: &mut HashSet<String>,
) -> Result<DependencyReport> {
    let mut report = DependencyReport::new();

    // Check for circular dependency
    if visited.contains(package_name) {
        let chain: Vec<String> = visited.iter().cloned().collect();
        return Err(BuffyError::CircularDependency {
            chain: chain.join(" -> "),
        });
    }

    visited.insert(package_name.to_string());

    // Resolve system dependencies
    for sys_dep in &deps.system {
        if check_system_dependency(sys_dep) {
            report.system_met.push(sys_dep.clone());
        } else {
            report.system_missing.push(sys_dep.clone());
        }
    }

    // Resolve BSL package dependencies
    for pkg_dep in &deps.packages {
        if is_package_installed(pkg_dep) {
            report.bsl_already_installed.push(pkg_dep.clone());
        } else {
            // Install the dependency
            println!("  Installing dependency: {}...", pkg_dep);
            let install_args = vec![pkg_dep.clone()];
            crate::package::install::install(&install_args)?;
            report.bsl_installed.push(pkg_dep.clone());
        }
    }

    Ok(report)
}

/// Checks if a system dependency exists on $PATH.
pub fn check_system_dependency(name: &str) -> bool {
    if let Ok(paths) = std::env::var("PATH") {
        for dir in paths.split(':') {
            let full_path = std::path::Path::new(dir).join(name);
            if full_path.exists() {
                return true;
            }
        }
    }
    false
}

/// Checks if a BSL package is already installed.
pub fn is_package_installed(name: &str) -> bool {
    let commands_dir = crate::config::buffy_home::commands_dir().join(name);
    commands_dir.exists()
}

/// Checks if any installed package depends on the given package.
/// Used before uninstalling to warn about dependents.
pub fn find_dependents(name: &str) -> Result<Vec<String>> {
    let installed = crate::config::settings::read_installed()?;
    let mut dependents = Vec::new();

    for entry in &installed {
        if entry.dependencies.packages.iter().any(|d| d == name) {
            dependents.push(entry.name.clone());
        }
    }

    Ok(dependents)
}

/// Verifies all system dependencies for an installed package.
pub fn verify_system_dependencies(package_dir: &std::path::Path) -> Result<Vec<String>> {
    let manifest = crate::package::manifest::validate(package_dir)?;
    let mut missing = Vec::new();

    for sys_dep in &manifest.dependencies.system {
        if !check_system_dependency(sys_dep) {
            missing.push(sys_dep.clone());
        }
    }

    Ok(missing)
}

/// Verifies all BSL package dependencies for an installed package.
pub fn verify_bsl_dependencies(package_dir: &std::path::Path) -> Result<Vec<String>> {
    let manifest = crate::package::manifest::validate(package_dir)?;
    let mut missing = Vec::new();

    for pkg_dep in &manifest.dependencies.packages {
        if !is_package_installed(pkg_dep) {
            missing.push(pkg_dep.clone());
        }
    }

    Ok(missing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_system_dependency_exists() {
        // "sh" should exist on any Unix system
        let result = check_system_dependency("sh");
        assert!(result);
    }

    #[test]
    fn test_check_system_dependency_not_exists() {
        let result = check_system_dependency("nonexistent_command_xyz_123");
        assert!(!result);
    }

    #[test]
    fn test_is_package_installed_not_installed() {
        let result = is_package_installed("nonexistent-pkg-xyz");
        assert!(!result);
    }

    #[test]
    fn test_resolve_empty_dependencies() {
        let deps = PackageDependencies::default();
        let mut visited = HashSet::new();
        let result = resolve_dependencies(&deps, "test-pkg", &mut visited);
        assert!(result.is_ok());
        if let Ok(report) = result {
            assert!(report.system_met.is_empty());
            assert!(report.system_missing.is_empty());
            assert!(report.bsl_installed.is_empty());
            assert!(report.bsl_already_installed.is_empty());
        }
    }

    #[test]
    fn test_find_dependents_empty() {
        // No packages installed in test environment
        let result = find_dependents("test-pkg");
        assert!(result.is_ok());
        if let Ok(dependents) = result {
            assert!(dependents.is_empty());
        }
    }

    #[test]
    fn test_verify_system_dependencies_no_pkg_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = verify_system_dependencies(dir.path());
        assert!(result.is_err()); // No package.json
    }
}
