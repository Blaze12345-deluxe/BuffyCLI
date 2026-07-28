//! BSL command resolution via directory tree walking.
//!
//! Resolves CLI command arguments to .bsl file paths by walking the
//! ~/.buffy/commands/ directory structure. Supports alias resolution,
//! multi-level subcommands, and a flat-lookup fallback for commands
//! that match a .bsl file by name across all packages.

pub mod tree;
