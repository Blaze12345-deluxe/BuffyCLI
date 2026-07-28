//! GitHub repository integration for package distribution.
//!
//! Handles fetching repository indexes, downloading package files, and
//! searching across multiple configured repositories. All HTTP requests
//! use synchronous `ureq` connections to raw.githubusercontent.com.

pub mod github;
pub mod index;
pub mod source;

pub use github::{
    fetch_index,
    fetch_index_cached,
    refresh_index,
    validate_repository,
    search_across_repositories,
    find_across_repositories,
    download_file,
};
