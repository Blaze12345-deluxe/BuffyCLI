# Buffy CLI — Architecture & Directory Structure

> **Version:** 1.0 (Design Document)
> **Target Language:** Rust
> **Last Updated:** 2026-07-27

---

## Table of Contents

1. [Project Layout](#1-project-layout)
2. [Crate Dependencies](#2-crate-dependencies)
3. [Module Architecture](#3-module-architecture)
4. [Data Flow](#4-data-flow)
5. [Core Component Design](#5-core-component-design)
6. [BSL Language Processing Pipeline](#6-bsl-language-processing-pipeline)
7. [Error Handling Strategy](#7-error-handling-strategy)
8. [Configuration & State Files](#8-configuration--state-files)
9. [Testing Strategy](#9-testing-strategy)
10. [Implementation Roadmap](#10-implementation-roadmap)

---

## 1. Project Layout

```
buffy/
│
├── Cargo.toml                          # Single crate (binary + library)
├── Cargo.lock
├── README.md
├── LICENSE
│
├── src/
│   ├── main.rs                         # Entry point: parse args → call lib::run()
│   ├── lib.rs                          # Library root: re-exports, run() dispatch
│   │
│   ├── cli/
│   │   ├── mod.rs                      # CLI module root
│   │   ├── args.rs                     # Clap derive structs (ArgMatches wrappers)
│   │   └── dispatch.rs                 # Command → handler routing
│   │
│   ├── resolver/
│   │   ├── mod.rs                      # Resolver module root
│   │   ├── tree.rs                     # Directory tree walker for commands/
│   │   └── pattern.rs                  # Fuzzy/partial command name matching
│   │
│   ├── bsl/
│   │   ├── mod.rs                      # BSL module root (re-exports key types)
│   │   ├── lexer.rs                    # Tokenizer: raw text → tokens
│   │   ├── ast.rs                      # AST node definitions (Statement, Metadata, etc.)
│   │   ├── parser.rs                   # Parser: tokens → AST
│   │   ├── interpreter.rs              # Interpreter: walk AST → execute
│   │   ├── executor.rs                 # Shell executor: RUN + capture stdio
│   │   ├── variable.rs                 # Variable resolver: ${VAR} → values
│   │   └── error.rs                    # BSL-specific errors (parse, runtime)
│   │
│   ├── package/
│   │   ├── mod.rs                      # Package module root
│   │   ├── install.rs                  # Install from repo, local, or GitHub URL
│   │   ├── uninstall.rs                # Uninstall a package
│   │   ├── update.rs                   # Update single or all packages
│   │   ├── verify.rs                   # SHA-256 integrity verification
│   │   ├── manifest.rs                 # package.json parsing & validation
│   │   └── discover.rs                 # System scan → suggest packages
│   │
│   ├── repository/
│   │   ├── mod.rs                      # Repository module root
│   │   ├── github.rs                   # GitHub API: download, list, index fetch
│   │   ├── index.rs                    # index.json parsing & version comparison
│   │   └── source.rs                   # Source enum (GitHub URL, alias, local)
│   │
│   ├── config/
│   │   ├── mod.rs                      # Config module root
│   │   ├── settings.rs                 # Config structs with serde derives
│   │   ├── buffy_home.rs               # ~/.buffy/ directory layout & helpers
│   │   └── paths.rs                    # Platform-appropriate path resolution (XDG)
│   │
│   ├── logger/
│   │   ├── mod.rs                      # Logger module root
│   │   └── formatter.rs                # Terminal output: colored WRITE, error formatting
│   │
│   ├── diagnostic/
│   │   ├── mod.rs                      # Diagnostic module root
│   │   ├── doctor.rs                   # Full system check (config, repos, permissions)
│   │   ├── lint.rs                     # BSL syntax & style linting
│   │   └── validate.rs                 # Metadata validation
│   │
│   └── error.rs                        # Unified error types (thiserror)
│
├── tests/
│   ├── integration/
│   │   ├── bsl_lexer_test.rs
│   │   ├── bsl_parser_test.rs
│   │   ├── bsl_interpreter_test.rs
│   │   ├── resolver_test.rs
│   │   ├── install_test.rs
│   │   └── config_test.rs
│   │
│   └── fixtures/
│       ├── sample-scripts/             # Sample .bsl files for parser tests
│       ├── mock-repository/            # Fake package repo for install tests
│       │   └── packages/
│       │       └── pip-env/
│       │           ├── module.bsl
│       │           └── package.json
│       └── mock-buffy-home/            # Fake ~/.buffy/ for config tests
│           ├── commands/
│           ├── config.json
│           └── installed.json
│
├── docs/                               # Existing project docs
│   ├── install-docs.txt
│   ├── build-in-commands.txt
│   ├── Buffy-Script-Language-Spec.txt
│   ├── plan.txt
│   └── architecture.md                 # This file
│
└── pip-env.bsl                         # Example BSL script
```

---

## 2. Crate Dependencies

```toml
[package]
name = "buffy"
version = "0.1.0"
edition = "2021"
description = "Buffy CLI Automation Framework"

[dependencies]
# CLI argument parsing
clap = { version = "4", features = ["derive"] }

# Serialization for JSON config files
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# HTTP client for GitHub API (synchronous, lightweight CLI choice)
ureq = "2"

# SHA-256 verification
sha2 = "0.10"

# Error handling
thiserror = "1"
anyhow = "1"

# Platform-appropriate directory paths ($HOME, XDG)
dirs = "5"

# Terminal output colors
colored = "2"

# Timestamps for log files
chrono = "0.4"

# File globbing (FOREACH, system discovery)
glob = "0.3"

# Progress bars (install/update operations)
indicatif = "0.17"

# Temporary files (package downloads)
tempfile = "3"

# Logging framework
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tempfile = "3"

[profile.release]
opt-level = 2
strip = true
lto = true
```

### Why These Crates?

| Crate | Purpose | Why Not Alternatives |
|-------|---------|---------------------|
| `clap` | CLI argument parsing | Industry standard, derive macros are clean |
| `serde` | JSON config | Standard Rust serialization |
| `ureq` | HTTP downloading | Simpler than `reqwest` for CLIs; no async runtime needed |
| `sha2` | Package verification | Direct SHA-256; no need for higher-level crypto |
| `thiserror` | Error types | Idiomatic Rust error derives |
| `anyhow` | Main error handling | Simplifies error propagation in `main()` |
| `dirs` | Path resolution | Cross-platform home/config directories |
| `colored` | Terminal colors | Simple API, widely used |
| `indicatif` | Progress bars | Standard for Rust CLI tools |
| `tracing` | Logging | Structured, composable; better than `log` crate |

Note: `ureq` is synchronous, so we **do not need `tokio`** for this project. This keeps the binary small and simple. If async is needed later (e.g., parallel repository checking), it can be introduced incrementally.

---

## 3. Module Architecture

### Dependency Graph

```
main.rs
  │
  └── lib.rs (run dispatch)
        │
        ├── cli::args          # Pure: parse → Command enum
        ├── cli::dispatch      # Pure: Command → handler call
        │
        ├── resolver           # Reads ~/.buffy/commands/ filesystem
        │
        ├──── bsl              # Core BSL pipeline
        │     ├── lexer
        │     ├── parser
        │     ├── interpreter
        │     ├── executor     # Spawns child processes
        │     └── variable     # Pure: string substitution
        │
        ├──── package          # Package management
        │     ├── install      # Depends on repository + config
        │     ├── uninstall
        │     ├── update
        │     ├── verify       # Pure: hash checking
        │     ├── manifest     # Pure: JSON parsing
        │     └── discover     # System scan
        │
        ├──── repository        # Network access via ureq
        │     ├── github
        │     ├── index
        │     └── source
        │
        ├──── config            # Reads/writes ~/.buffy/
        │     ├── settings
        │     ├── buffy_home
        │     └── paths
        │
        ├──── logger            # Terminal output
        │     └── formatter
        │
        └──── error             # Used everywhere
```

### Internal Dependencies (Which modules import what)

```
cli          → config, resolver, bsl, package, repository, logger, diagnostic
resolver     → config, bsl::error
bsl::lexer   → bsl::ast
bsl::parser  → bsl::lexer, bsl::ast, bsl::error
bsl::interp  → bsl::parser, bsl::executor, bsl::variable, bsl::error
bsl::exec    → (std::process::Command)
bsl::var     → ()
package::*   → config, repository, bsl::parser, bsl::error, logger
repository   → config, logger
config       → error
diagnostic   → config, package, bsl::parser, repository, logger
logger       → (colored, tracing)
error        → (thiserror)
```

---

## 4. Data Flow

### 4.1 General Execution Flow

```
┌──────────────┐
│  User types  │
│  `buffy ...` │
└──────┬───────┘
       │
       ▼
┌──────────────────────────────────────┐
│  main.rs                             │
│  • Initialize logger (tracing)       │
│  • Call lib::run()                   │
└──────────┬───────────────────────────┘
           │
           ▼
┌──────────────────────────────────────┐
│  cli::args::parse()                  │
│  • Parse raw args using clap         │
│  • Return CliArgs struct             │
└──────────┬───────────────────────────┘
           │
           ▼
┌──────────────────────────────────────┐
│  cli::dispatch::execute(args)        │
│  • Match args.command → handler      │
└──────────┬───────────────────────────┘
           │
     ┌─────┴───────────────────────────────────────────┐
     │                                                  │
     ▼                                                  ▼
┌─────────────────┐                      ┌──────────────────────┐
│ Built-in Command │                      │ BSL Command          │
│ (--install,      │                      │ (e.g. "pip-env")     │
│  --version, etc) │                      │                      │
└────────┬─────────┘                      └──────────┬───────────┘
         │                                            │
         ▼                                            ▼
   Handler function                           resolver::resolve(args)
   (e.g., package::install())                      │
                                                   ▼
                                          bsl::interpreter::run()
                                                   │
                                                   ▼
                                          bsl::executor::execute()
                                                   │
                                                   ▼
                                          (shell commands run)
```

### 4.2 Package Install Flow

```
buffy --install pip-env
       │
       ▼
cli::dispatch
       │
       ▼
package::install::install("pip-env")
       │
       ├──→ config::settings::read_repositories()
       │       │
       │       ▼
       │   ["https://github.com/BuffyCLI/packages", ...]
       │
       ├──→ repository::github::fetch_index(repo_url)
       │       │
       │       ▼
       │   index.json content
       │
       ├──→ repository::index::find_package("pip-env")
       │       │
       │       ▼
       │   { name: "pip-env", version: "1.2.0", path: "packages/pip-env" }
       │
       ├──→ repository::github::download_package(url, temp_dir)
       │       │
       │       ▼
       │   ZIP/tarball extracted to temp dir
       │
       ├──→ package::verify::verify_integrity(temp_dir)
       │       │
       │       ▼
       │   SHA-256 match ✓
       │
       ├──→ package::manifest::validate(temp_dir)
       │       │
       │       ▼
       │   package.json valid ✓, module.bsl present ✓
       │
       ├──→ config::buffy_home::install_to_commands(pkg_name, temp_dir)
       │       │
       │       ▼
       │   ~/.buffy/commands/pip-env/module.bsl
       │
       └──→ config::settings::register_installed(pkg_name, version)
               │
               ▼
           installed.json updated ✓
```

### 4.3 BSL Script Execution Flow

```
buffy pip-env
       │
       ▼
resolver::tree::resolve(["pip-env"])
       │
       ├──→ Walk ~/.buffy/commands/pip-env/
       ├──→ Found → commands/pip-env/module.bsl
       │
       ▼
       Read file → String
       │
       ▼
bsl::lexer::tokenize(source)
       │
       ▼
   Vec<Token>
       │
       ▼
bsl::parser::parse(tokens)
       │
       ▼
   BslScript { metadata: [...], statements: [...] }
       │
       ▼
bsl::interpreter::run(script, args)
       │
       ├──→ Validate metadata section
       ├──→ Set OUTPUT mode
       │
       ├──→ For each statement:
       │       │
       │       ├──→ statement: WRITE("Hello")
       │       │       ├──→ bsl::variable::resolve("Hello") → "Hello"
       │       │       └──→ logger::formatter::write("Hello")
       │       │
       │       ├──→ statement: RUN("python3 -m venv .venv")
       │       │       ├──→ bsl::variable::resolve("python3 -m venv .venv")
       │       │       ├──→ bsl::executor::run("python3 -m venv .venv", output_mode)
       │       │       │       ├──→ std::process::Command::new("python3")
       │       │       │       ├──→ .args(["-m", "venv", ".venv"])
       │       │       │       ├──→ .stdout(configurable)
       │       │       │       ├──→ .stderr(captured)
       │       │       │       └──→ Return Result<Output, ExecError>
       │       │       │
       │       │       └──→ If exit_code != 0 → halt with error
       │       │
       │       ├──→ statement: WAIT(5)
       │       │       └──→ std::thread::sleep(Duration::from_secs(5))
       │       │
       │       └──→ statement: EXIT
       │               └──→ break out of execution loop
       │
       └──→ Return Ok(())
```

---

## 5. Core Component Design

### 5.1 CLI Module (`src/cli/`)

```rust
// ── src/cli/args.rs ──

/// Top-level CLI argument structure parsed by clap.
/// Buffy accepts two modes:
///   1. Flag-style commands: buffy --install pip-env
///   2. Subcommand-style:   buffy install pip-env
///   3. BSL commands:       buffy docker compose up
#[derive(clap::Parser)]
#[command(name = "buffy", about = "Buffy CLI Automation Framework")]
pub struct CliArgs {
    /// BSL command to run (the "rest" after flags)
    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,

    // ── Built-in Flags ──
    #[arg(long = "list", short = 'l')]
    pub list: bool,

    #[arg(long = "version")]
    pub version: bool,

    #[arg(long = "about")]
    pub about: bool,

    #[arg(long = "doctor")]
    pub doctor: bool,

    #[arg(long = "update")]
    pub self_update: bool,

    #[arg(long = "check-update")]
    pub check_update: bool,

    // ── Package Management ──
    #[arg(long = "install")]
    pub install: Option<Vec<String>>,

    #[arg(long = "uninstall")]
    pub uninstall: Option<String>,

    #[arg(long = "update-packages")]
    pub update_packages: bool,

    #[arg(long = "update-package")]
    pub update_package: Option<String>,

    #[arg(long = "info")]
    pub info: Option<String>,

    #[arg(long = "outdated")]
    pub outdated: bool,

    #[arg(long = "verify")]
    pub verify: Option<String>,

    #[arg(long = "clean")]
    pub clean: bool,

    // ── Repository Management ──
    #[arg(long = "repo")]
    pub repo: Option<Vec<String>>,

    // ── Logs ──
    #[arg(long = "logs")]
    pub logs: Option<Option<String>>,

    // ── Script Utilities ──
    #[arg(long = "check")]
    pub check: Option<String>,

    #[arg(long = "validate")]
    pub validate: Option<String>,

    #[arg(long = "run")]
    pub run: Option<String>,

    #[arg(long = "benchmark")]
    pub benchmark: Option<String>,

    // ── System ──
    #[arg(long = "repair")]
    pub repair: bool,

    #[arg(long = "reset")]
    pub reset: bool,

    #[arg(long = "discover")]
    pub discover: bool,
}
```

```rust
// ── src/cli/dispatch.rs ──

/// The dispatch function routes parsed CLI arguments to the correct handler.
///
/// # Logic
/// 1. If a flag like `--install` is set → call the package installer.
/// 2. If no flags match → treat remaining args as a BSL command path.
/// 3. Execute the BSL script via resolver → interpreter pipeline.
pub fn execute(args: CliArgs) -> Result<()> {
    // Priority: flags first, then BSL commands
    if args.list { return list_commands(); }
    if args.version { return print_version(); }
    if args.doctor { return run_doctor(); }
    // ... match other flags ...

    // If no flags matched, treat remaining args as BSL command
    if !args.command.is_empty() {
        let path = resolver::resolve(&args.command)?;
        let source = std::fs::read_to_string(&path)?;
        let script = bsl::parse(&source)?;
        bsl::interpret(&script, &args.command[1..])?;
    }

    // If no args at all → show welcome screen
    show_welcome()
}
```

### 5.2 Resolver Module (`src/resolver/`)

```rust
// ── src/resolver/tree.rs ──

/// Resolves a BSL command from remaining CLI arguments by walking
/// the `~/.buffy/commands/` directory tree.
///
/// # Resolution Logic
/// - `buffy pip-env` → walk `commands/pip-env/module.bsl`
/// - `buffy restart docker` → walk `commands/restart/docker.bsl`
/// - `buffy docker compose up` → walk `commands/docker/compose/up.bsl`
///
/// At each step, first try to match the arg as a subdirectory.
/// For the final arg, try:
///   1. `<arg>.bsl` (exact file)
///   2. `module.bsl` inside `<arg>/` directory
pub fn resolve(args: &[String]) -> Result<PathBuf, ResolverError> {
    let commands_dir = config::buffy_home::commands_dir();

    let mut current_dir = commands_dir;

    for (i, arg) in args.iter().enumerate() {
        let is_last = i == args.len() - 1;

        if is_last {
            // Last arg: try as a .bsl file
            let file_path = current_dir.join(format!("{}.bsl", arg));
            if file_path.exists() {
                return Ok(file_path);
            }

            // Try as a directory with module.bsl inside
            let module_path = current_dir.join(arg).join("module.bsl");
            if module_path.exists() {
                return Ok(module_path);
            }

            return Err(ResolverError::NotFound {
                command: args.join(" "),
                searched: current_dir.clone(),
            });
        } else {
            // Middle arg: walk into subdirectory
            current_dir = current_dir.join(arg);
            if !current_dir.is_dir() {
                return Err(ResolverError::NotFound {
                    command: args.join(" "),
                    searched: current_dir.clone(),
                });
            }
        }
    }

    unreachable!("args is non-empty, so we should have returned")
}
```

### 5.3 BSL Module (`src/bsl/`)

#### AST Types (`ast.rs`)

```rust
/// Represents a complete parsed BSL script.
pub struct BslScript {
    pub metadata: Vec<Metadata>,
    pub statements: Vec<Statement>,
}

/// Metadata lines that appear before any executable instructions.
pub enum Metadata {
    Version(String),
    Author(String),
    Description(String),
    Output(bool),
}

/// Executable instructions in a BSL script.
pub enum Statement {
    Write(String),          // WRITE "text"
    Run(String),            // RUN "command"
    Wait(WaitTarget),       // WAIT <seconds|"prompt">
    Clear,                  // CLEAR
    Exit,                   // EXIT
}

pub enum WaitTarget {
    Duration(u64),
    Prompt(String),
}
```

#### Lexer (`lexer.rs`)

```rust
/// Tokenizes raw BSL source text into a stream of tokens.
/// BSL is line-oriented: each line is parsed independently.
///
/// # Tokens
/// - Identifiers: VERSION, AUTHOR, DESCRIPTION, OUTPUT,
///                WRITE, RUN, WAIT, CLEAR, EXIT
/// - Strings: "quoted text"
/// - Numbers: 5, 10, 30
/// - Equals sign: =
/// - Comment: // (rest of line ignored)
/// - Newline
/// - EOF
pub fn tokenize(source: &str) -> Result<Vec<Token>, BslError> {
    // Remove BOM if present
    // Split into lines
    // Strip comments
    // Tokenize each line independently
    // Return Vec<Token> or error with line number
}
```

#### Parser (`parser.rs`)

```rust
/// Parses a token stream into a BslScript AST.
///
/// # Validation performed
/// 1. Metadata section must appear before statements
/// 2. No metadata duplicates (e.g., two VERSION lines)
/// 3. All metadata values are valid types (bool for OUTPUT)
/// 4. Unknown identifiers are rejected
/// 5. Every statement has the correct number of arguments
/// 6. Strings are properly quoted
///
/// Returns detailed errors with line numbers.
pub fn parse(tokens: Vec<Token>) -> Result<BslScript, BslError> {
    // Phase 1: Scan metadata until first statement
    // Phase 2: Parse remaining tokens as statements
    // Phase 3: Validate (no unknown instructions, etc.)
}
```

#### Interpreter (`interpreter.rs`)

```rust
/// Executes a parsed BSL script.
///
/// The interpreter:
/// 1. Applies metadata settings (OUTPUT mode, etc.)
/// 2. Iterates through statements sequentially
/// 3. Resolves variables in each statement
/// 4. Dispatches each statement to the appropriate handler
/// 5. Halts on error (displays command, exit code, stderr)
/// 6. Returns control to the caller
pub fn interpret(script: &BslScript, args: &[String]) -> Result<()> {
    let ctx = ExecutionContext {
        output_enabled: script.get_output_mode(),
        args: args.to_vec(),
    };

    for statement in &script.statements {
        match statement {
            Statement::Write(text) => {
                let resolved = resolve_variables(text, &ctx);
                logger::write(&resolved);
            }
            Statement::Run(cmd) => {
                let resolved = resolve_variables(cmd, &ctx);
                let result = executor::execute(&resolved, ctx.output_enabled)?;
                if !result.success {
                    return Err(execution_error(&resolved, result));
                }
            }
            Statement::Wait(target) => handle_wait(target),
            Statement::Clear => executor::clear_screen(),
            Statement::Exit => break,
        }
    }
    Ok(())
}
```

#### Executor (`executor.rs`)

```rust
/// Result of executing a shell command.
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

/// Executes a shell command string using the system shell.
///
/// # Behavior
/// - OUTPUT = true:  stdout is shown in real-time (inherited)
/// - OUTPUT = false: stdout is suppressed entirely
/// - stderr is always captured, shown only on failure
/// - Shell used: /bin/sh on Unix, cmd.exe on Windows
///
/// # Security
/// Commands are passed to the system shell as-is.
/// BSL scripts are assumed to be trusted (user-installed).
pub fn execute(command: &str, output_enabled: bool) -> Result<ExecResult> {
    let shell = if cfg!(unix) { "/bin/sh" } else { "cmd.exe" };
    let flag = if cfg!(unix) { "-c" } else { "/C" };

    let mut cmd = Command::new(shell);
    cmd.arg(flag).arg(command);

    if output_enabled {
        // Inherit stdout/stderr so user sees output in real-time
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::piped());
    } else {
        // Suppress stdout, still capture stderr
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());
    }

    let output = cmd.output()?;

    Ok(ExecResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        success: output.status.success(),
    })
}
```

#### Variable Resolver (`variable.rs`)

```rust
/// Resolves `${VAR}` placeholders in a string.
///
/// # Built-in Variables
/// - `${HOME}`     → User's home directory
/// - `${USER}`     → Current username
/// - `${PWD}`      → Current working directory
/// - `${TEMP}`     → System temp directory
/// - `${DATE}`     → Current date (YYYY-MM-DD)
/// - `${TIME}`     → Current time (HH:MM:SS)
/// - `${1}-${N}`   → Command-line arguments
///
/// # Resolution Order
/// 1. Replace built-in variables
/// 2. Replace numbered argument variables
/// 3. Leave unknown variables as-is (for env vars)
pub fn resolve(text: &str, ctx: &ExecutionContext) -> String {
    // Regex-based replacement: \$\{(\w+)\}
    // Match each variable name and look it up
    let mut result = text.to_string();

    // Built-in variables
    for (name, value) in builtin_variables(ctx) {
        result = result.replace(&format!("${{{}}}", name), &value);
    }

    // Numbered arguments
    for (i, arg) in ctx.args.iter().enumerate() {
        let var_name = format!("${{{}}}", i + 1);
        result = result.replace(&var_name, arg);
    }

    result
}
```

### 5.4 Package Manager (`src/package/`)

```rust
/// The install function handles three installation sources:
/// 1. Local file:  ./pip-env.bsl  (prefix "./")
/// 2. GitHub URL:  github.com/user/repo pip-env
/// 3. Repository:  pip-env  (search configured repos)
pub fn install(args: &[String]) -> Result<()> {
    let (source, packages) = parse_install_args(args)?;

    match source {
        InstallSource::Local(path) => install_local(path)?,
        InstallSource::GitHub { owner, repo } => {
            let index = repository::fetch_index(&owner, &repo)?;
            for pkg in packages {
                let info = index.find(&pkg)?;
                download_and_install(&owner, &repo, info)?;
            }
        }
        InstallSource::Configured => {
            let repos = config::settings::read_repositories()?;
            for pkg in packages {
                let (repo_url, info) = repository::find_across_repos(&repos, &pkg)?;
                download_and_install_from_url(&repo_url, info)?;
            }
        }
    }

    Ok(())
}

/// Downloads a package, verifies integrity, validates structure,
/// installs to ~/.buffy/commands/, and registers in installed.json.
fn download_and_install(url: &str, info: &PackageInfo) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    // Download and extract
    repository::github::download_package(url, info, temp_dir.path())?;

    // Verify SHA-256
    package::verify::verify_package(temp_dir.path(), info)?;

    // Validate package structure
    let manifest = package::manifest::validate(temp_dir.path())?;

    // Install to commands directory
    let dest = config::buffy_home::commands_dir().join(&info.name);
    fs::create_dir_all(&dest)?;
    fs::copy(temp_dir.path().join("module.bsl"), dest.join("module.bsl"))?;

    // Register
    config::settings::register_installed(&info.name, &info.version)?;

    Ok(())
}
```

### 5.5 Configuration Module (`src/config/`)

```rust
// ── src/config/buffy_home.rs ──

/// Returns the path to `~/.buffy/`.
/// Creates it if it doesn't exist.
pub fn buffy_home() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".buffy")
}

pub fn commands_dir() -> PathBuf {
    buffy_home().join("commands")
}

pub fn packages_dir() -> PathBuf {
    buffy_home().join("packages")
}

pub fn cache_dir() -> PathBuf {
    buffy_home().join("cache")
}

pub fn logs_dir() -> PathBuf {
    buffy_home().join("logs")
}

pub fn ensure_directories() -> Result<()> {
    fs::create_dir_all(commands_dir())?;
    fs::create_dir_all(packages_dir())?;
    fs::create_dir_all(cache_dir())?;
    fs::create_dir_all(logs_dir())?;
    Ok(())
}
```

```rust
// ── src/config/settings.rs ──

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_repositories: Vec<String>,
    pub output_preferences: OutputPrefs,
    pub update_settings: UpdateSettings,
    pub package_verification: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub installed: String,   // ISO date
    pub source: String,      // "official" | "local" | "github.com/..."
}

pub fn read_config() -> Result<Config> { /* read config.json */ }
pub fn write_config(config: &Config) -> Result<()> { /* write config.json */ }
pub fn read_installed() -> Result<Vec<InstalledPackage>> { /* read installed.json */ }
pub fn register_installed(name: &str, version: &str) -> Result<()> { /* append to installed.json */ }
pub fn read_repositories() -> Result<Vec<String>> { /* read repositories.json */ }
```

### 5.6 Repository Module (`src/repository/`)

```rust
/// Downloads the index.json from a GitHub repository.
/// Uses the GitHub raw content URL.
pub fn fetch_index(owner: &str, repo: &str) -> Result<RepositoryIndex> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/main/index.json",
        owner, repo
    );
    let response = ureq::get(&url).call()?;
    let json: RepositoryIndex = serde_json::from_reader(response.into_reader())?;
    Ok(json)
}

/// Downloads a specific package from a GitHub repository.
/// Downloads as ZIP from GitHub, extracts to target directory.
pub fn download_package(
    owner: &str,
    repo: &str,
    package_path: &str,
    target_dir: &Path,
) -> Result<()> {
    // Download module.bsl and package.json from raw.githubusercontent.com
    let base_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main/{}",
        owner, repo, package_path
    );

    // Download module.bsl
    let module_url = format!("{}/module.bsl", base_url);
    let response = ureq::get(&module_url).call()?;
    let mut file = fs::File::create(target_dir.join("module.bsl"))?;
    std::io::copy(&mut response.into_reader(), &mut file)?;

    // Download package.json
    let pkg_url = format!("{}/package.json", base_url);
    let response = ureq::get(&pkg_url).call()?;
    let mut file = fs::File::create(target_dir.join("package.json"))?;
    std::io::copy(&mut response.into_reader(), &mut file)?;

    Ok(())
}
```

### 5.7 Logger Module (`src/logger/`)

```rust
/// Displays a WRITE message to the terminal.
/// Uses colored output for formatting.
pub fn write(message: &str) {
    println!("{}", message);
}

/// Displays a success message (green checkmark).
pub fn success(message: &str) {
    println!("{} {}", "✔".green(), message);
}

/// Displays an error message (red X).
pub fn error(message: &str) {
    eprintln!("{} {}", "✘".red().bold(), message);
}

/// Displays a formatted command error with exit code.
pub fn command_error(command: &str, exit_code: i32, stderr: &str) {
    eprintln!("{}", "Error".red().bold());
    eprintln!();
    eprintln!("{}", "Command:".yellow());
    eprintln!("{}", command);
    eprintln!();
    eprintln!("{}", "Exit Code:".yellow());
    eprintln!("{}", exit_code);
    if !stderr.is_empty() {
        eprintln!();
        eprintln!("{}", "stderr:".yellow());
        eprintln!("{}", stderr);
    }
}

/// Displays a progress bar (used during install/update).
pub fn progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}
```

### 5.8 Diagnostics Module (`src/diagnostic/`)

```rust
// ── src/diagnostic/doctor.rs ──

/// Runs the full system diagnostic (`buffy --doctor`).
///
/// Checks performed:
/// 1. ~/.buffy/ directory structure exists (commands, packages, cache, logs)
/// 2. config.json is valid JSON
/// 3. installed.json is valid JSON (all referenced packages exist on disk)
/// 4. repositories.json is valid (URLs are reachable)
/// 5. Every installed .bsl file parses correctly (BSL syntax check)
/// 6. File permissions are correct on installed scripts
/// 7. Disk space and temp directory writability
pub fn run_doctor() -> Result<DoctorReport> {
    let mut report = DoctorReport::new();

    report.check("Home directory", check_home_directory());
    report.check("Config file", check_config_file());
    report.check("Installed packages", check_installed_packages());
    report.check("Repository connectivity", check_repositories());
    report.check("BSL syntax", check_all_scripts());
    report.check("Permissions", check_permissions());

    Ok(report)
}
```

---

## 6. BSL Language Processing Pipeline

```
Source Text (.bsl file)
        │
        ▼
   ┌──────────────────────┐
   │ 1. Lexer             │  Line-by-line tokenization
   │    tokenize()        │  • Strip comments (//)
   │                      │  • Split into tokens (ID, STRING, NUMBER, =)
   │                      │  • Track line numbers
   └──────────┬───────────┘
              │
              ▼
   ┌──────────────────────┐
   │ 2. Parser            │  Token → AST conversion
   │    parse()           │  • Validate metadata section placement
   │                      │  • Group metadata, reject duplicates
   │                      │  • Parse statements with correct arity
   │                      │  • Return BslScript { metadata, statements }
   └──────────┬───────────┘
              │
              ▼
   ┌──────────────────────┐
   │ 3. Variable Resolver │  String interpolation
   │    resolve()         │  • Replace ${HOME}, ${USER}, etc.
   │                      │  • Replace ${1}, ${2}, etc. (args)
   │                      │  • Leave env vars intact
   └──────────┬───────────┘
              │
              ▼
   ┌──────────────────────┐
   │ 4. Interpreter       │  AST execution
   │    interpret()       │  • Apply metadata (OUTPUT mode)
   │                      │  • Walk statements sequentially
   │                      │  • Dispatch to handler functions
   │                      │  • Handle errors with context
   └──────────┬───────────┘
              │
              ▼
   ┌──────────────────────┐
   │ 5. Executor          │  Shell interaction
   │    execute()         │  • Spawn child process
   │                      │  • Handle output mode (show/hide)
   │                      │  • Capture stderr for error display
   │                      │  • Return exit code + output
   └──────────────────────┘
```

---

## 7. Error Handling Strategy

### Unified Error Types

```rust
// ── src/error.rs ──

use thiserror::Error;

/// Top-level error type for all Buffy operations.
/// Each variant carries context about what went wrong and where.
#[derive(Error, Debug)]
pub enum BuffyError {
    // ── BSL Language Errors ──
    #[error("BSL syntax error in {path}:{line}: {message}")]
    BslSyntax {
        path: String,
        line: usize,
        message: String,
    },

    #[error("BSL runtime error: {message}")]
    BslRuntime {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("Unknown instruction at line {line}: `{instruction}`")]
    UnknownInstruction { line: usize, instruction: String },

    // ── Resolution Errors ──
    #[error("Command not found: `{command}`")]
    CommandNotFound { command: String },

    // ── Package Errors ──
    #[error("Package `{name}` not found in any repository")]
    PackageNotFound { name: String },

    #[error("Package `{name}` verification failed: {detail}")]
    PackageVerificationFailed { name: String, detail: String },

    #[error("Invalid package manifest in `{path}`: {detail}")]
    InvalidManifest { path: String, detail: String },

    // ── Repository Errors ──
    #[error("Failed to connect to repository `{url}`: {detail}")]
    RepositoryConnection { url: String, detail: String },

    // ── Configuration Errors ──
    #[error("Invalid configuration in `{path}`: {detail}")]
    ConfigError { path: String, detail: String },

    // ── IO Errors ──
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // ── JSON Errors ──
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    // ── HTTP Errors ──
    #[error("HTTP error: {0}")]
    Http(#[from] ureq::Error),
}
```

### Error Propagation Pattern

```
Library functions  → Return Result<T, BuffyError>
                   → Use thiserror's #[from] for automatic conversions

Binary (main.rs)   → Use anyhow::Result for top-level error handling
                   → Display errors with logger::error()
                   → Exit with non-zero status code

BSL runtime errors → Display command, exit code, stderr
                   → Stop execution immediately
                   → Return BuffyError::BslRuntime
```

---

## 8. Configuration & State Files

### File Location: `~/.buffy/`

```
~/.buffy/
│
├── config.json                    # User configuration
├── installed.json                 # Installed package registry
├── repositories.json              # Package repository list
├── aliases.json                   # Command aliases
│
├── commands/                      # Installed .bsl commands
│   ├── pip-env/
│   │   └── module.bsl
│   ├── restart/
│   │   └── docker.bsl
│   └── docker/
│       └── compose/
│           └── up.bsl
│
├── packages/                      # Cached package metadata
│
├── cache/                         # Temporary download files
│
└── logs/                          # Execution logs
    └── 2026-07-27.log
```

### File Formats

**config.json:**
```json
{
    "default_repositories": ["https://github.com/BuffyCLI/packages"],
    "output_preferences": {
        "color": true,
        "show_progress": true
    },
    "update_settings": {
        "check_on_startup": true,
        "auto_update": false
    },
    "package_verification": true
}
```

**installed.json:**
```json
{
    "pip-env": {
        "version": "1.2.0",
        "installed": "2026-07-27",
        "source": "official"
    },
    "docker-tools": {
        "version": "2.4.1",
        "installed": "2026-07-25",
        "source": "github.com/ExampleUser/docker-packages"
    }
}
```

**repositories.json:**
```json
[
    "https://github.com/BuffyCLI/packages",
    "https://github.com/ExampleUser/linux-tools"
]
```

**aliases.json:**
```json
{
    "ve": "pip-env",
    "dcu": "docker compose up",
    "dcd": "docker compose down"
}
```

---

## 9. Testing Strategy

### Unit Tests (within each module)

| Module | What to Test |
|--------|-------------|
| `bsl::lexer` | Tokenization of valid/invalid BSL, comments, edge cases |
| `bsl::parser` | AST construction, error reporting, metadata validation |
| `bsl::interpreter` | Statement dispatch, variable resolution, error handling |
| `bsl::executor` | Command execution, output modes, exit code handling |
| `bsl::variable` | All built-in variables, args, unknown vars, edge cases |
| `resolver::tree` | Directory walking, file resolution, error cases |
| `package::manifest` | Valid/invalid package.json parsing |
| `package::verify` | SHA-256 matching, tamper detection |
| `config::settings` | Read/write round-trips, missing files, corrupt data |
| `logger::formatter` | Output formatting, color stripping |

### Integration Tests (`tests/integration/`)

| Test | What it Tests |
|------|-------------|
| `bsl_parser_test.rs` | Full scripts from fixtures → parsed AST matches expected |
| `bsl_interpreter_test.rs` | Execute BSL scripts, verify side effects |
| `resolver_test.rs` | Mock command tree, verify resolution paths |
| `install_test.rs` | Mock repository, verify install flow |
| `config_test.rs` | Mock buffy home, verify read/write operations |

### Test Fixtures (`tests/fixtures/`)

```
fixtures/
├── sample-scripts/
│   ├── valid_full.bsl          # Complete valid script with all features
│   ├── valid_minimal.bsl       # Minimal valid script
│   ├── invalid_bad_metadata.bsl  # Metadata after statements
│   ├── invalid_unknown.bsl     # Unknown instruction
│   └── invalid_syntax.bsl      # Malformed syntax
│
├── mock-repository/
│   ├── index.json
│   └── packages/
│       ├── pip-env/
│       │   ├── module.bsl
│       │   └── package.json
│       └── docker-restart/
│           ├── module.bsl
│           └── package.json
│
└── mock-buffy-home/
    ├── config.json
    ├── installed.json
    ├── repositories.json
    └── commands/
```

---

## 10. Implementation Roadmap

The implementation should proceed in incremental, testable stages:

### Phase 1: Project Scaffolding
- [ ] Initialize Rust project with Cargo
- [ ] Configure Cargo.toml with all dependencies
- [ ] Create module skeleton (all `mod.rs` files)
- [ ] Implement `config::buffy_home` and `config::paths`
- [ ] Set up tracing/logging
- [ ] Verify build succeeds

### Phase 2: BSL Language Core
- [ ] Implement AST types (`bsl::ast`)
- [ ] Implement lexer (`bsl::lexer`)
- [ ] Implement parser (`bsl::parser`)
- [ ] Write unit tests for lexer + parser
- [ ] Implement variable resolver (`bsl::variable`)
- [ ] Implement executor (`bsl::executor`)
- [ ] Implement interpreter (`bsl::interpreter`)
- [ ] Wire up: parse + interpret a .bsl file
- [ ] Test with `pip-env.bsl`

### Phase 3: CLI Argument Parsing
- [ ] Define `CliArgs` with clap derive
- [ ] Implement dispatch logic
- [ ] Implement `--version`, `--about`, welcome screen
- [ ] Implement command resolver (`resolver::tree`)
- [ ] Wire up: `buffy pip-env` → resolve → interpret
- [ ] Write integration tests

### Phase 4: Package Manager
- [ ] Implement `package::manifest` (package.json parsing)
- [ ] Implement `repository::index` (index.json parsing)
- [ ] Implement `repository::github` (download from GitHub raw)
- [ ] Implement `package::verify` (SHA-256)
- [ ] Implement `package::install` (local files)
- [ ] Implement `package::install` (from GitHub repos)
- [ ] Implement `package::uninstall`
- [ ] Implement `--install`, `--uninstall` CLI flags

### Phase 5: Repository System
- [ ] Implement `repositories.json` management
- [ ] Implement multi-repo search with version comparison
- [ ] Implement `--repo add/remove/list`
- [ ] Implement `@` wildcard install
- [ ] Implement `package::update` (single + all)

### Phase 6: Diagnostics & Utilities
- [ ] Implement `--doctor` system check
- [ ] Implement `--check` (syntax check)
- [ ] Implement `--validate` (metadata validation)
- [ ] Implement `--discover` (system scan)
- [ ] Implement `--benchmark`
- [ ] Implement `--logs`
- [ ] Implement `--repair`, `--reset`
- [ ] Implement `--clean`

### Phase 7: Polish & Documentation
- [ ] Colored output formatting everywhere
- [ ] Progress bars for install/update
- [ ] Error message polish (helpful suggestions)
- [ ] Shell auto-completion scripts
- [ ] Full README.md with examples
- [ ] CI pipeline setup (GitHub Actions)
- [ ] Cross-platform testing (Linux, macOS, Windows)

---

> **Design Philosophy**  
> The architecture keeps each subsystem independent. The BSL interpreter doesn't know about package management. The CLI parser doesn't know about the interpreter internals. This makes each component testable in isolation and allows future features (async, plugins, WASM embedding) to be added without rewrites.
