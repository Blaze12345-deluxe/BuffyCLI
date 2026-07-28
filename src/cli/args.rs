#[derive(clap::Parser)]
#[command(
    name = "buffy",
    about = "Buffy CLI Automation Framework",
    long_about = "A lightweight, cross-platform CLI automation framework.\n\
                  \n\
                  Commands are written in BSL (Buffy Script Language) — a simple text format.\n\
                  Install packages from GitHub repositories, write your own .bsl files,\n\
                  and run them from the terminal.\n\
                  \n\
                  Examples:\n\
                    buffy pip-env              Run an installed BSL command\n\
                    buffy --install pip-env    Install a package\n\
                    buffy --run script.bsl     Run a .bsl file on the fly\n\
                    buffy --list              List installed commands\n\
                    buffy --repo search git   Search packages across repositories",
    disable_version_flag = true,
    arg_required_else_help = false,
)]
pub struct CliArgs {
    /// BSL command and its arguments to run (e.g. `buffy pip-env create myproject`)
    ///
    /// If no flags are given, the first argument is resolved as an installed BSL
    /// command. Any remaining arguments are passed to the script as ${1}, ${2}, etc.
    pub command: Vec<String>,

    // ── Built-in Flags ──

    /// List all installed BSL commands
    #[arg(long = "list", short = 'l', help = "List installed commands")]
    pub list: bool,

    /// Show buffy version
    #[arg(long = "version", help = "Print version information")]
    pub version: bool,

    /// Show project information
    #[arg(long = "about", help = "Display detailed project information")]
    pub about: bool,

    /// Run system diagnostics
    #[arg(long = "doctor", help = "Check system health and configuration")]
    pub doctor: bool,

    /// Self-update buffy to the latest version
    #[arg(long = "update", help = "Download and install the latest version")]
    pub self_update: bool,

    /// Check for buffy updates without downloading
    #[arg(long = "check-update", help = "Check if a newer version is available")]
    pub check_update: bool,

    // ── Package Management ──

    /// Install one or more packages (local file, GitHub repo, or by name)
    ///
    /// Examples:
    ///   buffy --install ./pip-env.bsl              Local .bsl file
    ///   buffy --install github.com/user/repo pkg   From specific repository
    ///   buffy --install pip-env                    Search configured repos
    ///   buffy --install github.com/user/repo @     Install all packages from a repo
    #[arg(long = "install", help = "Install package(s) from file, repo, or by name")]
    pub install: Option<Vec<String>>,

    /// Uninstall a package by name
    #[arg(long = "uninstall", help = "Remove an installed package")]
    pub uninstall: Option<String>,

    /// Update all installed packages to latest versions
    #[arg(long = "update-packages", help = "Update all installed packages")]
    pub update_packages: bool,

    /// Update a specific package to the latest version
    #[arg(long = "update-package", help = "Update a single installed package")]
    pub update_package: Option<String>,

    /// Show metadata for an installed package
    #[arg(long = "info", help = "Display package information")]
    pub info: Option<String>,

    /// List packages that have newer versions available
    #[arg(long = "outdated", help = "Show packages with available updates")]
    pub outdated: bool,

    /// Verify a package's SHA-256 integrity
    #[arg(long = "verify", help = "Check package integrity via SHA-256")]
    pub verify: Option<String>,

    /// Clear the download cache
    #[arg(long = "clean", help = "Remove cached files to free disk space")]
    pub clean: bool,

    // ── Repository Management ──

    /// Manage package repositories (list, add, remove, refresh, search)
    ///
    /// Examples:
    ///   buffy --repo list               List configured repositories    ///   buffy --repo add <url>        Add and validate a repository
    ///   buffy --repo <url>              Shorthand for --repo add
    ///   buffy --repo remove <url>       Remove a repository
    ///   buffy --repo refresh            Force-refresh all repository indexes
    ///   buffy --repo search <query>     Search packages across all repositories
    #[arg(long = "repo", help = "Manage package repositories")]
    pub repo: Option<Vec<String>>,

    // ── Logs ──

    /// View or clear execution logs
    ///
    /// Examples:
    ///   buffy --logs          Show the latest log file
    ///   buffy --logs clear    Delete all log files
    #[arg(long = "logs", help = "View or clear execution logs")]
    pub logs: Option<Option<String>>,

    // ── Script Utilities ──

    /// Check a .bsl file for syntax errors
    #[arg(long = "check", help = "Validate BSL syntax in a file")]
    pub check: Option<String>,

    /// Validate a .bsl file's metadata
    #[arg(long = "validate", help = "Check that required metadata is present")]
    pub validate: Option<String>,

    /// Run a .bsl file without installing it
    ///
    /// Useful for testing scripts before packaging them.
    /// Arguments after the filename are passed to the script.
    #[arg(long = "run", help = "Execute a .bsl file directly (no install)")]
    pub run: Option<String>,

    /// Benchmark a .bsl script's execution time
    #[arg(long = "benchmark", help = "Measure script performance (3 runs + average)")]
    pub benchmark: Option<String>,

    // ── Alias Management ──

    /// Manage command aliases and conflict preferences
    ///
    /// Examples:
    ///   buffy --alias list                    List all aliases and conflict preferences
    ///   buffy --alias set ve pip-env          Create shortcut alias
    ///   buffy --alias remove ve               Remove an alias
    ///   buffy --alias resolve pip-env         Show cached conflict preference
    #[arg(long = "alias", help = "Manage aliases and conflict preferences")]
    pub alias: Option<Vec<String>>,

    // ── System ──

    /// Fix broken configuration and orphaned packages
    #[arg(long = "repair", help = "Detect and fix configuration issues")]
    pub repair: bool,

    /// Reset configuration to defaults (preserves packages)
    #[arg(long = "reset", help = "Restore default settings")]
    pub reset: bool,

    /// Scan system and suggest packages from detected tools
    #[arg(long = "discover", help = "Auto-detect tools and suggest matching packages")]
    pub discover: bool,

    // ── Shell Completion ──

    /// Generate shell completion script
    ///
    /// Examples:
    ///   eval "$(buffy --completion bash)"        Enable bash completions
    ///   eval "$(buffy --completion zsh)"         Enable zsh completions
    ///   eval "$(buffy --completion fish)"        Enable fish completions
    #[arg(long = "completion", help = "Generate shell completion script")]
    pub completion: Option<String>,
}
