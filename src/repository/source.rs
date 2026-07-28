/// Represents the type of installation source.
#[derive(Debug, Clone, PartialEq)]
pub enum InstallSource {
    /// Local filesystem path (e.g., ./pip-env.bsl)
    Local(String),
    /// GitHub repository with owner and repo name
    GitHub { owner: String, repo: String },
    /// Search configured repositories
    Configured,
}

/// Parses an install argument to determine the source.
pub fn parse_source(arg: &str) -> InstallSource {
    if arg.starts_with("./") {
        InstallSource::Local(arg.to_string())
    } else if let Some(url) = arg.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            InstallSource::GitHub {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
            }
        } else {
            InstallSource::Configured
        }
    } else if let Some(path) = arg.strip_prefix("github.com/") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            InstallSource::GitHub {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
            }
        } else {
            InstallSource::Configured
        }
    } else {
        InstallSource::Configured
    }
}

/// Parses a GitHub URL string into (owner, repo) pair.
/// Accepts formats: "github.com/owner/repo", "https://github.com/owner/repo", "owner/repo"
pub fn parse_github_url(url: &str) -> Option<(String, String)> {
    let cleaned = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("github.com/"))
        .unwrap_or(url);
    let parts: Vec<&str> = cleaned.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_source() {
        assert_eq!(parse_source("./pip-env.bsl"), InstallSource::Local("./pip-env.bsl".to_string()));
    }

    #[test]
    fn test_github_url() {
        let source = parse_source("https://github.com/Blaze12345-deluxe/Buffy-Plugins");
        assert_eq!(
            source,
            InstallSource::GitHub {
                owner: "Blaze12345-deluxe".to_string(),
                repo: "Buffy-Plugins".to_string(),
            }
        );
    }

    #[test]
    fn test_github_com_format() {
        let source = parse_source("github.com/user/repo");
        assert_eq!(
            source,
            InstallSource::GitHub {
                owner: "user".to_string(),
                repo: "repo".to_string(),
            }
        );
    }

    #[test]
    fn test_configured_source() {
        assert_eq!(parse_source("pip-env"), InstallSource::Configured);
    }

    #[test]
    fn test_parse_github_url_full() {
        let result = parse_github_url("https://github.com/owner/repo");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_short() {
        let result = parse_github_url("github.com/owner/repo");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_bare() {
        let result = parse_github_url("owner/repo");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_invalid() {
        let result = parse_github_url("just-a-name");
        assert_eq!(result, None);
    }
}
