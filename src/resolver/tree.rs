use std::path::PathBuf;
use crate::error::{BuffyError, Result};

/// Resolves a BSL command by walking ~/.buffy/commands/.
/// Aliases are resolved first so "ve" becomes "pip-env" before directory lookup.
pub fn resolve(args: &[String]) -> Result<PathBuf> {
    let commands_dir = crate::config::buffy_home::commands_dir();
    let mut current_dir = commands_dir.clone();

    // Resolve aliases for the first argument
    let resolved_args: Vec<String> = if let Some(first) = args.first() {
        let resolved = crate::config::aliases::resolve(first).unwrap_or_else(|_| first.to_string());
        if resolved != *first {
            // Alias expanded: replace the first arg
            let mut new_args = vec![resolved];
            new_args.extend_from_slice(&args[1..]);
            new_args
        } else {
            args.to_vec()
        }
    } else {
        return Err(BuffyError::CommandNotFound {
            command: String::new(),
        });
    };

    for (i, arg) in resolved_args.iter().enumerate() {
        let is_last = i == resolved_args.len() - 1;

        if is_last {
            // Last arg: try as a .bsl file
            let file_path = current_dir.join(format!("{}.bsl", arg));
            if file_path.exists() {
                return Ok(file_path);
            }

            // Try index.bsl as default
            let index_path = current_dir.join(arg).join("index.bsl");
            if index_path.exists() {
                return Ok(index_path);
            }

            // Try as a directory with matching name
            let dir_name_path = current_dir.join(arg).join(format!("{}.bsl", arg));
            if dir_name_path.exists() {
                return Ok(dir_name_path);
            }

            // Try first .bsl alphabetically
            if let Ok(entries) = std::fs::read_dir(current_dir.join(arg)) {
                let mut bsl_files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "bsl"))
                    .collect();
                bsl_files.sort_by_key(|e| e.file_name());
                if let Some(first) = bsl_files.into_iter().next() {
                    return Ok(first.path());
                }
            }

            // Flat lookup fallback: scan all package subdirectories for a .bsl file
            // matching the command name (e.g., "cd-down" -> commands/*/cd-down.bsl)
            if current_dir == commands_dir {
                if let Ok(entries) = std::fs::read_dir(&commands_dir) {
                    let mut matches: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .filter_map(|dir| {
                            let candidate = dir.path().join(format!("{}.bsl", arg));
                            if candidate.exists() {
                                Some(candidate)
                            } else {
                                None
                            }
                        })
                        .collect();

                    if matches.len() == 1 {
                        return Ok(matches.swap_remove(0));
                    } else if matches.len() > 1 {
                        // Multiple matches: return the first one
                        return Ok(matches.swap_remove(0));
                    }
                }
            }

            return Err(BuffyError::CommandNotFound {
                command: args.join(" "),
            });
        } else {
            // Middle arg: walk into subdirectory
            current_dir = current_dir.join(arg);
            if !current_dir.is_dir() {
                return Err(BuffyError::CommandNotFound {
                    command: args.join(" "),
                });
            }
        }
    }

    Err(BuffyError::CommandNotFound {
        command: args.join(" "),
    })
}
