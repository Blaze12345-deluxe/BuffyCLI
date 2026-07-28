# Contributing to Buffy

Thank you for considering contributing to Buffy! This guide covers everything you need to know to get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Quick Start](#quick-start)
- [Project Architecture](#project-architecture)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Testing Guide](#testing-guide)
- [Pull Request Workflow](#pull-request-workflow)
- [Writing BSL Packages](#writing-bsl-packages)
- [Release Process](#release-process)

---

## Code of Conduct

This project is governed by the **Contributor Covenant v2.1**. By participating, you agree to uphold a harassment-free experience for everyone. Report unacceptable behavior to the project maintainers.

---

## Quick Start

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/buffy.git
cd buffy

# Build
cargo build

# Run tests
cargo test

# Check for warnings
cargo clippy -- -D warnings

# Format code
cargo fmt
```

Make sure the project builds and all tests pass before making changes:

```bash
cargo test  # Ensure all tests pass
```

---

## Project Architecture

Buffy is a Rust CLI application composed of independent subsystems:

```
src/
├── main.rs                   # Entry point — calls buffy::run()
├── lib.rs                    # Subsystem initialization, run() function
├── cli/                      # CLI parsing (clap) and command dispatch
│   ├── args.rs               # CliArgs derive struct with all flags
│   └── dispatch.rs           # Route flags → handlers, resolve BSL commands
├── bsl/                      # BSL interpreter core
│   ├── lexer.rs              # tokenize() — line-based tokenizer
│   ├── ast.rs                # BslScript, Metadata, Statement enums
│   ├── parser.rs             # parse() — validate + build AST
│   ├── interpreter.rs        # interpret() — walk AST, execute
│   ├── executor.rs           # execute() — spawn shell, capture output
│   ├── variable.rs           # resolve() — ${HOME}, ${USER}, ${1}–${N}
│   ├── error.rs              # BslError variants
│   └── mod.rs                # Public API: parse(), interpret()
├── package/                  # Package management
│   ├── install.rs            # install() — all 4 sources
│   ├── uninstall.rs          # uninstall() — remove + deregister
│   ├── update.rs             # update_all(), update_one() — version check
│   ├── verify.rs             # verify_package() — SHA-256 3-way check
│   ├── manifest.rs           # PackageManifest — validate JSON
│   ├── deps.rs               # Dependency resolution
│   ├── discover.rs           # discover() — scan $PATH, suggest packages
│   └── mod.rs
├── repository/               # GitHub repository integration
│   ├── source.rs             # InstallSource enum + parsing
│   ├── index.rs              # RepositoryIndex — find_package()
│   ├── github.rs             # fetch_index(), download_package()
│   └── mod.rs
├── config/                   # Configuration file management
│   ├── settings.rs           # Read/write config.json, installed.json, etc.
│   ├── buffy_home.rs         # ~/.buffy/ directory layout
│   ├── paths.rs              # Platform path resolution
│   └── mod.rs
├── resolver/                 # Command resolution (directory walk)
│   ├── tree.rs               # resolve() — match args → .bsl files
│   └── mod.rs
├── diagnostic/               # System checks
│   ├── doctor.rs             # run_doctor() — full diagnostics
│   ├── lint.rs               # lint_script() — syntax + style
│   ├── validate.rs           # validate_script() — metadata checks
│   └── mod.rs
├── logger/                   # Terminal output formatting
│   ├── formatter.rs          # write(), success(), error(), warning(), info()
│   └── mod.rs
└── error.rs                  # Unified BuffyError enum (thiserror)
```

### Dependency Graph

```
bsl/  ←  resolver/  ←  cli/  ←  lib.rs  ←  main.rs
  ↑                       ↑
config/  ─────────────────┤
logger/  ─────────────────┤
package/ ←  repository/ ──┤
diagnostic/  ─────────────┤
```

**Key rule:** Core subsystems (`bsl/`, `config/`, `logger/`) should not depend on upper layers (`cli/`, `package/`). New modules should follow this layering.

---

## Development Setup

### Prerequisites

- **Rust 1.70+** (install via [rustup](https://rustup.rs/))
- **Linux** (primary target; macOS secondary; Windows tertiary)
- A shell at `/bin/sh` or configured `$SHELL`

### Editor Setup

We recommend:

- **VS Code** with the `rust-analyzer` extension
- **CLion** with the Rust plugin
- **Helix** / **Neovim** with rust-analyzer LSP

Enable `rust-analyzer.checkOnSave` for instant feedback.

### Building

```bash
# Debug build (fast iteration)
cargo build

# Release build (optimized, stripped)
cargo build --release

# Run directly from source
cargo run -- --help
cargo run -- --run examples/system-info.bsl
```

### Running Specific Subsystems

```bash
# Run a BSL script directly
cargo run -- --run examples/disk-usage.bsl

# Check syntax only (no execution)
cargo run -- --check examples/disk-usage.bsl

# Install a local package
cargo run -- --install ./pip-env.bsl

# List installed commands
cargo run -- --list
```

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `SHELL` | Shell for executing RUN commands | `/bin/sh` |
| `HOME` | User home directory (used for `~/.buffy/`) | System default |
| `GITHUB_TOKEN` | GitHub API token (private repos) | None |

---

## Coding Standards

### Style

- **Formatting:** Run `cargo fmt` before every commit. CI enforces `cargo fmt --check`.
- **Clippy:** Run `cargo clippy -- -D warnings`. CI treats all warnings as errors.
- **No unsafe code** unless absolutely necessary and documented with `// SAFETY:`.
- **No unwrap/expect** in production code paths. Use `?` with proper error types.

### Naming

- **Types:** `PascalCase` — `BslScript`, `InstallSource`, `DoctorReport`
- **Functions:** `snake_case` — `tokenize()`, `resolve()`, `verify_package()`
- **Modules:** `snake_case` — `buffy_home.rs`, `formatter.rs`
- **Error variants:** `PascalCase` with description — `PackageVerificationFailed`, `CircularDependency`

### Error Handling

All errors flow through one of two types:

1. **`BuffyError`** (`src/error.rs`) — Top-level errors with helpful suggestions for the user. Used by CLI dispatch and package management.
2. **`BslError`** (`src/bsl/error.rs`) — BSL-specific errors (syntax, runtime, unknown instruction). Used by the interpreter.

Rules:

- Use `thiserror` derives for all error types
- Every error variant should include enough context for a helpful error message
- Prefer specific error variants over generic `Other(String)` or `Io(io::Error)` wrappers
- Use `anyhow::Result` only at the top-level CLI dispatch layer

### Imports

Group imports in this order, separated by blank lines:

```rust
use std::...;        // Standard library
use std::io::...;    // (still std, grouped together)

use crate::...;      // Crate-internal modules
use serde::...;      // Third-party crates
```

### Comments

- **`///`** for public API documentation (rendered in docs)
- **`//`** for internal implementation notes
- **`// SAFETY:`** for unsafe block justifications
- **`// TODO:`** for planned but unimplemented work
- **`// HACK:`** for workarounds that should be revisited
- **`// NOTE:`** for non-obvious design decisions

---

## Testing Guide

### Test Locations

Tests live in two places:

1. **Inline unit tests** — `#[cfg(test)] mod tests { ... }` in each source file
2. **Integration tests** — `tests/integration/*.rs` for end-to-end flows

### Running Tests

```bash
# All tests
cargo test

# Specific module (unit tests)
cargo test bsl::            # BSL interpreter tests
cargo test config::         # Configuration tests
cargo test package::        # Package manager tests
cargo test repository::     # Repository tests
cargo test diagnostic::     # Diagnostic tests
cargo test resolver::       # Resolver tests
cargo test cli::            # CLI tests

# Specific test function
cargo test test_parse_full_script

# Integration tests only
cargo test --test bsl_lexer_test
cargo test --test bsl_parser_test
cargo test --test bsl_pipeline_test
cargo test --test resolver_test
cargo test --test config_test

# Run without parallelization (for tests that modify HOME env var)
cargo test -- --test-threads=1
```

### Test Fixtures

Test fixtures live in `tests/fixtures/`:

```
tests/fixtures/
├── sample-scripts/          # .bsl files for parser/interpreter tests
│   ├── valid_full.bsl
│   ├── valid_minimal.bsl
│   ├── invalid_bad_metadata.bsl
│   ├── invalid_unknown.bsl
│   └── invalid_syntax.bsl
├── mock-repository/         # Mock package index for install tests
│   ├── index.json
│   └── packages/
│       ├── pip-env/
│       │   ├── module.bsl
│       │   ├── package.json
│       │   └── pip-env-SHA.txt
│       └── git-tools/
│           ├── clone.bsl
│           ├── package.json
│           └── git-tools-SHA.txt
└── mock-buffy-home/         # Mock ~/.buffy/ config for config tests
    ├── config.json
    ├── installed.json
    ├── repositories.json
    └── commands/
```

When adding a new fixture:

- Use `tempfile::TempDir` to create isolated test environments
- For `HOME`-sensitive tests, use the shared `TEST_HOME_LOCK` mutex to prevent parallel-test races
- Never write to the real `~/.buffy/` directory in tests

### Writing Tests

**Unit test example (inline):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_script() {
        let tokens = lexer::tokenize(r#"WRITE "hello""#).unwrap();
        let script = parser::parse(tokens).unwrap();
        assert_eq!(script.statements.len(), 1);
    }

    #[test]
    fn test_resolve_home_variable() {
        let ctx = ExecutionContext { args: vec![] };
        let result = variable::resolve("${HOME}/projects", &ctx);
        assert!(result.starts_with('/'));
        assert!(result.ends_with("/projects"));
    }
}
```

**Integration test example:**

```rust
#[test]
fn test_install_from_local_file() {
    let tmp = tempfile::tempdir().unwrap();
    let bsl_path = tmp.path().join("test-script.bsl");
    std::fs::write(&bsl_path, r#"VERSION = "1.0"\nWRITE "hello"\n"#).unwrap();

    // Set up mock home
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());

    let result = buffy::package::install::install_local(&bsl_path.to_string_lossy());
    assert!(result.is_ok());
}
```

### Test Guidelines

- **Every new module** should have unit tests covering at least 80% of its functionality
- **Every bug fix** should include a test that reproduces the bug before the fix
- **Every new feature** should have tests for both success and error paths
- **Test edge cases:** empty input, null values, boundary conditions, invalid data
- **Name tests descriptively:** `test_empty_command_returns_error`, `test_metadata_after_statement_rejected`
- **Avoid test interdependence:** each test should set up its own state
- **Use `assert!` / `assert_eq!` / `assert_matches!`** — never `println!` for verification

---

## Pull Request Workflow

### 1. Before You Start

- **Open an issue** first to discuss your proposed change (feature, bug fix, refactor)
- Wait for maintainer feedback before investing significant time
- Check [existing issues](https://github.com/Blaze12345-deluxe/BuffyCLI/issues) to avoid duplicates

### 2. Branch Naming

```
feat/short-description     # New features
fix/short-description      # Bug fixes
docs/short-description     # Documentation
refactor/short-description # Code restructuring
test/short-description     # Adding/fixing tests
```

### 3. Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
feat(bsl): add support for OUTPUT toggle at runtime

If a BSL script has multiple RUN commands, the user can now toggle
OUTPUT on/off mid-script. Previously OUTPUT was only read from metadata.

Closes #42
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `style`, `ci`

### 4. Before Submitting

```bash
# Ensure everything is clean
cargo build                    # Must compile without errors
cargo test                     # All tests must pass
cargo clippy -- -D warnings    # No warnings allowed
cargo fmt --check              # Code must be formatted
```

### 5. PR Checklist

- [ ] Code follows coding standards (fmt, clippy)
- [ ] All existing tests pass
- [ ] New tests added for new functionality
- [ ] Documentation updated (README, docs/, module-level docs)
- [ ] Changes are scoped to a single concern
- [ ] Commit messages follow Conventional Commits
- [ ] Branch is rebased on latest `main`

### 6. Review Process

1. A maintainer will review within 3 business days
2. Address review feedback with additional commits (no rebasing/squashing during review)
3. Once approved, a maintainer will merge your PR

---

## Writing BSL Packages

### Package Structure

```
my-package/
├── package.json              # Manifest (required)
├── my-package-SHA.txt        # SHA-256 checksum (required)
├── my-package.bsl            # Command(s) — at least one .bsl file required
└── README.md                 # Documentation (recommended)
```

### package.json

```json
{
  "name": "my-package",
  "version": "2026.07.27",
  "description": "Does something useful",
  "author": "Your Name",
  "license": "MIT",
  "sha256": "abcdef123456...",
  "commands": ["my-package"],
  "tags": ["utility", "dev-tools"],
  "system_dependencies": ["git", "curl"],
  "bsl_dependencies": ["pip-env"],
  "min_buffy_version": "0.1.0"
}
```

### SHA.txt Format

```
<64-char-hex-hash>  <package-name>
```

Generate it with:

```bash
# Hash all files EXCEPT package.json and *-SHA.txt
sha256sum my-package.bsl README.md | sed 's/ .*\// /' > my-package-SHA.txt
```

### Testing Your Package

```bash
# Validate locally before publishing
buffy --check my-package.bsl
buffy --validate my-package.bsl
buffy --install ./my-package
buffy my-package
buffy --verify my-package
buffy --uninstall my-package
```

### Publishing

1. Fork the [Blaze12345-deluxe/Buffy-Plugins](https://github.com/Blaze12345-deluxe/Buffy-Plugins) repository
2. Add your package to `packages/<name>/`
3. Update `index.json` with your package entry
4. Submit a pull request to the plugins repository

---

## Release Process

Releases are done by maintainers. The process:

1. **Version bump** — Update `version` in `Cargo.toml`
2. **Full test suite** — `cargo build --release && cargo test && cargo clippy && cargo fmt --check`
3. **Changelog** — Document changes for the release
4. **Tag** — `git tag v0.X.0 && git push origin v0.X.0`
5. **Release** — Create a GitHub Release with:
   - Changelog summary
   - Release binary attached (`./target/release/buffy`)
   - SHA-256 checksum of the binary

---

## Getting Help

- **Issues:** [GitHub Issues](https://github.com/Blaze12345-deluxe/BuffyCLI/issues)
- **BSL Spec:** [`docs/Buffy-Script-Language-Spec.txt`](docs/Buffy-Script-Language-Spec.txt)
- **Build Plan:** [`docs/build.txt`](docs/build.txt)
- **Package Format:** [`docs/package-format-spec.md`](docs/package-format-spec.md)

---

Thank you for contributing to Buffy! Every contribution — code, docs, tests, or feedback — makes the project better.
