# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- **BSL example scripts** — 9 real-world `.bsl` examples in `examples/` covering system info, git setup, Docker cleanup, backups, network diagnostics, project scaffolding, and more
- **`CONTRIBUTING.md`** — Comprehensive contribution guidelines covering architecture, coding standards, testing workflow, PR process, and package publishing

### Fixed

- **Lexer** — `=` characters inside string arguments (e.g. `WRITE "====="`) no longer incorrectly parsed as metadata assignment operators
- **Interpreter** — Runtime `OUTPUT = true/false` toggling now works mid-script instead of being read only from metadata
- **Lint** — Missing `Statement::SetOutput` match arm added to exhaustive match
- **Uninstall** — Removed unused variable causing compiler warning
- **Test stability** — Shared `TEST_HOME_LOCK` mutex prevents parallel-test races on `HOME` env var across the `aliases` and `buffy_home` modules

---

## [0.1.0] — 2026-07-27

### Added

#### Phase 0 — Project Initialization
- Rust project scaffolding with `cargo init`
- Module directory structure for all subsystems
- All crate dependencies configured (`clap`, `serde`, `ureq`, `sha2`, `thiserror`, `indicatif`, etc.)
- Release profile with LTO and strip optimization
- Test fixture directories (`tests/fixtures/`, `tests/integration/`)

#### Phase 1 — Configuration Manager
- Unified `BuffyError` enum with `thiserror` derives and user-facing suggestions
- Platform path resolution via `dirs` crate (`src/config/paths.rs`)
- `~/.buffy/` home directory layout with `ensure_directories()` (`src/config/buffy_home.rs`)
- JSON config file management: read/write `config.json`, `installed.json`, `repositories.json` (`src/config/settings.rs`)

#### Phase 2 — Logger
- Colored terminal output with `write()`, `success()` (green ✔), `error()` (red ✘), `warning()` (yellow), `info()` (blue) (`src/logger/formatter.rs`)
- `tracing` subscriber initialization for structured logging

#### Phase 3 — BSL Interpreter Core
- **AST** (`src/bsl/ast.rs`): `BslScript`, `Metadata` enum (`Version`, `Author`, `Description`, `Output`), `Statement` enum (`Write`, `Run`, `Wait`, `Clear`, `Exit`, `SetOutput`), `WaitTarget` enum (`Duration`, `Prompt`)
- **Lexer** (`src/bsl/lexer.rs`): Line-based tokenizer that strips comments (`//`), produces tokens with line numbers
- **Parser** (`src/bsl/parser.rs`): Validates metadata section, builds AST, rejects unknown instructions, enforces metadata-before-statements ordering
- **Variable resolver** (`src/bsl/variable.rs`): Resolves `${HOME}`, `${USER}`, `${PWD}`, `${TEMP}`, `${DATE}`, `${TIME}`, `${1}`–`${N}`
- **Executor** (`src/bsl/executor.rs`): Spawns user shell (`$SHELL` or `/bin/sh`), handles `OUTPUT` mode, captures stderr
- **Interpreter** (`src/bsl/interpreter.rs`): Walks AST, dispatches to handlers, stops on non-zero exit codes
- **BSL error types** (`src/bsl/error.rs`): `Syntax`, `Runtime`, `CommandFailed`, `UnknownInstruction`, `Io`

#### Phase 4 — CLI & Resolver
- **CLI argument parser** (`src/cli/args.rs`): `CliArgs` struct with clap derive covering all flags with thorough help text
- **Command resolver** (`src/resolver/tree.rs`): Walks `~/.buffy/commands/` directory tree matching CLI arguments to `.bsl` files with priority: exact match → `index.bsl` → directory name match → first alphabetically
- **Command dispatch** (`src/cli/dispatch.rs`): Routes flags to subsystem handlers, resolves BSL commands and passes to interpreter
- **Entry points** (`src/lib.rs`, `src/main.rs`): Logger initialization, directory setup, CLI execution

#### Phase 5 — Repository Manager
- **Install source parsing** (`src/repository/source.rs`): `InstallSource` enum (`Local`, `GitHub`, `Configured`) with resolution order
- **Repository index** (`src/repository/index.rs`): `RepositoryIndex` struct, `find_package()`, date-based version comparison
- **GitHub integration** (`src/repository/github.rs`): `fetch_index()` with local caching, `download_package()`, `download_sha_file()`, `search_across_repositories()`
- **Cross-repository search** (`src/repository/mod.rs`)

#### Phase 6 — Package Manager
- **Package manifest** (`src/package/manifest.rs`): `PackageManifest` struct with validation of required fields (`name`, `version`, `sha256`), auto-generation from `.bsl` files
- **SHA-256 verification** (`src/package/verify.rs`): 3-way comparison (generated hash vs `package.json` vs `{name}-SHA.txt`)
- **Package install** (`src/package/install.rs`): Supports all 4 sources (local, GitHub URL, configured repos, bulk `@`), download with progress bars, SHA verification, dependency resolution, registry update
- **Package uninstall** (`src/package/uninstall.rs`): Removes files, deregisters from `installed.json`, checks for dependent packages
- **Package update** (`src/package/update.rs`): `update_all()` and `update_one()` with version checks, stale file cleanup
- **System discovery** (`src/package/discover.rs`): Scans `$PATH` for common tools, matches tags against repository index, suggests and installs matching packages
- **Dependency resolution** (`src/package/deps.rs`): Resolves system dependencies (checks `$PATH`), BSL package dependencies (auto-installs)

#### Phase 7 — Diagnostics
- **System doctor** (`src/diagnostic/doctor.rs`): `run_doctor()` — checks directories, config integrity, package SHA integrity, repository connectivity, system dependencies, BSL dependencies
- **BSL linter** (`src/diagnostic/lint.rs`): `lint_script()` — recommends metadata, checks empty `WRITE`/`RUN`, detects destructive commands (`rm -rf /`), flags excessive `RUN` statements
- **BSL validator** (`src/diagnostic/validate.rs`): `validate_script()` — metadata completeness checks
- **Benchmark**: `benchmark_script()` — warm-up run + 3 timed runs with average/total reporting
- **Self-update**: `self_update()` — checks GitHub releases API, compares versions, prints download instructions
- **Repair**: `repair()` — fixes corrupt config files, regenerates repositories/aliases, removes orphaned packages, warns on SHA mismatches
- **Reset**: `reset()` — restores configuration to defaults

#### Phase 8 — Name Conflicts & Aliases
- Name conflict detection (same command name, different SHA)
- Conflict resolution prompt with user choice caching
- `--alias` flag management: `list`, `set`, `remove`, `resolve`
- Repository conflict handling (same package available from multiple repos)
- `aliases.json` file management for persistent conflict preferences

#### Phase 9 — Polish & Documentation
- **Colored output**: Welcome screen uses `logger::formatter` with consistent `write`, `info`, `success` calls
- **Progress bars**: `indicatif` spinner during package install showing fetch/verify/install operations
- **Error message polish**: All `BuffyError` variants include helpful suggestions (`--list`, `--repo search`, `--repair`, `--reset`), stderr preview with `OUTPUT=true` tip
- **Shell completion**: `--completion <shell>` flag supporting bash, zsh, fish, elvish, powershell via `clap_complete`
- **`README.md`**: Quick start, BSL language reference, command reference table, package management docs, configuration docs, development guide
- **CI pipeline** (`.github/workflows/ci.yml`): GitHub Actions — build + test + clippy + fmt-check on push/PR to `main`
- **Test fixtures**: 8 sample BSL scripts, 2 mock packages (`pip-env`, `git-tools`) with SHA-256 checksums, mock buffy home with config files and commands directory

### Tests

- 74 unit and integration tests across all subsystems
- Inline `#[cfg(test)]` tests in every module
- Integration tests in `tests/integration/` for end-to-end flows
- Test fixtures in `tests/fixtures/` for isolated, reproducible testing

---

[Unreleased]: https://github.com/Blaze12345-deluxe/BuffyCLI/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Blaze12345-deluxe/BuffyCLI/releases/tag/v0.1.0
