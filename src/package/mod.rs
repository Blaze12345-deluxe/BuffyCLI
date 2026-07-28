//! Package management: install, uninstall, update, verify, and discover.
//!
//! Handles all package lifecycle operations. Packages are installed from
//! GitHub repositories or local files, verified via SHA-256 checksums, and
//! tracked in installed.json. The module also handles system dependency
//! resolution and automatic tool discovery.

pub mod deps;
pub mod discover;
pub mod install;
pub mod manifest;
pub mod uninstall;
pub mod update;
pub mod verify;
