//! Configuration file management and directory path resolution.
//!
//! Manages all ~/.buffy/ state files (config.json, installed.json,
//! repositories.json, aliases.json) and provides path helpers for the
//! ~/.buffy/ directory structure.

pub mod aliases;
pub mod buffy_home;
pub mod paths;
pub mod settings;
