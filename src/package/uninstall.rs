use crate::error::{BuffyError, Result};
use crate::logger;

/// Removes an installed package.
/// Checks for dependent packages before uninstalling.
pub fn uninstall(name: &str) -> Result<()> {
    // Find the package in installed.json
    let installed = crate::config::settings::read_installed()?;
    let pos = installed.iter().position(|e| e.name == name);

    match pos {
        Some(idx) => {
            // Check for dependent packages that rely on this one
            let dependents = crate::package::deps::find_dependents(name)?;
            if !dependents.is_empty() {
                logger::formatter::warning(&format!(
                    "The following installed packages depend on \"{}\":",
                    name
                ));
                for dep in &dependents {
                    logger::formatter::info(&format!("  - {}", dep));
                }
                logger::formatter::warning("Uninstalling may break these packages.");
                // Continue anyway — the user explicitly asked to uninstall
            }

            // Remove from commands directory
            let pkg_dir = crate::config::buffy_home::commands_dir().join(name);
            if pkg_dir.exists() {
                std::fs::remove_dir_all(&pkg_dir)?;
                logger::formatter::success(&format!("Removed {}", name));
            } else {
                logger::formatter::warning(&format!("Package directory for {} not found", name));
            }

            // Remove from installed.json
            let mut updated = installed;
            updated.remove(idx);
            crate::config::settings::write_installed(&updated)?;

            logger::formatter::success("Package uninstalled.");
            Ok(())
        }
        None => Err(BuffyError::PackageNotFound {
            name: name.to_string(),
        }),
    }
}
