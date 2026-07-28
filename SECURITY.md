# Security Policy

## Supported Versions

We release security patches for the following versions:

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Active development |
| < 0.1   | ❌ Not supported |

During the pre-1.0 phase, security issues are addressed on a best-effort basis in the latest release. Once we reach 1.0.0, a formal LTS cadence will be established.

---

## Reporting a Vulnerability

We take all security vulnerabilities seriously. If you discover a security issue in Buffy, please follow the responsible disclosure process below.

### How to Report

**Do not report security vulnerabilities through public GitHub issues, discussions, or pull requests.**

Instead, report via one of the following methods:

1. **GitHub Private Vulnerability Reporting** (preferred)\
   Use the [Security Advisories](https://github.com/Blaze12345-deluxe/BuffyCLI/security/advisories/new) page on our GitHub repository to submit a private report.

2. **Email**\
   Send details to the project maintainers by reaching out through the contact information on the [repository owner's GitHub profile](https://github.com/Blaze12345-deluxe).

### What to Include

When reporting, please include as much of the following as possible:

- **Type of issue** (e.g., arbitrary code execution, privilege escalation, denial of service, data exposure)
- **Affected versions** — the versions of Buffy you've confirmed are affected
- **Affected components** — which subsystem is involved (BSL interpreter, package installer, resolver, repository manager, etc.)
- **Steps to reproduce** — a minimal, reproducible test case or script
- **Impact** — what an attacker could achieve by exploiting the issue
- **Suggested fix** — if you have one, include it (optional but appreciated)

### Response Timeline

| Timeframe | Action |
|-----------|--------|
| Within 48 hours | We acknowledge receipt of your report |
| Within 5 business days | Initial triage and severity assessment |
| Within 14 business days | Patch development, review, and release plan |
| Within 30 calendar days | Public disclosure (coordinated release date) |

Timelines may vary depending on severity and complexity. We will keep you informed of progress throughout.

---

## Scope

### In Scope

The following components are within scope for security reporting:

- **BSL interpreter** (`src/bsl/`) — sandbox violations, command injection via BSL scripts, variable resolution bypass
- **Package installer** (`src/package/install.rs`) — path traversal, malicious package installation, SHA-256 verification bypass
- **Package verifier** (`src/package/verify.rs`) — hash collision, verification skip
- **Repository manager** (`src/repository/`) — repository spoofing, MITM during package download, index injection
- **Config file handling** (`src/config/`) — config injection, privilege escalation via `~/.buffy/` file manipulation
- **HTTP client** (`src/repository/github.rs`) — TLS validation bypass, insecure redirect handling
- **Dependency resolution** (`src/package/deps.rs`) — dependency confusion, malicious dependency injection
- **Shell command execution** (`src/bsl/executor.rs`) — command injection via `${}` variable expansion, argument injection

### Out of Scope

The following are considered out of scope:

- **Denial of service via resource exhaustion** (e.g., filling disk with installed packages, running excessive WAIT durations)
- **Attacks requiring local file system access** to `~/.buffy/` (assume user trusts their own home directory)
- **Social engineering attacks** against repository maintainers
- **Vulnerabilities in third-party dependencies** — report those to the respective package maintainers
- **Theoretical attacks without practical exploitation paths**
- **BSL script-level issues** (script authors can write whatever commands they want — BSL is intentionally not sandboxed from the user's shell)

---

## BSL Security Model

BSL is intentionally **not sandboxed** — any `RUN` instruction executes arbitrary shell commands as the current user. Security is provided at the *package distribution* layer, not the *script execution* layer:

1. **Integrity** — All packages are verified via SHA-256 checksums before installation (3-way comparison: computed hash vs `package.json` vs `{name}-SHA.txt`)
2. **Authenticity** — Packages are distributed through GitHub repositories with TLS-protected downloads
3. **Transparency** — Installed packages are plain text `.bsl` files — users can inspect them before running
4. **Choice** — Users choose which repositories to trust; no packages are installed without explicit user action

If you find a way to bypass SHA-256 verification, inject malicious packages into a repository, or execute commands in a way the user did not intend, **please report it immediately**.

---

## Safe Installation Practices

For users:

- **Only install packages from repositories you trust.** The default plugin repository is `https://github.com/Blaze12345-deluxe/Buffy-Plugins`.
- **Inspect `.bsl` files before running them.** Installed packages live in `~/.buffy/commands/` — they're plain text.
- **Use `buffy --verify <package>`** to check package integrity after installation.
- **Use `buffy --check <file.bsl>`** to validate syntax without executing.
- **Review `package.json`** for `system_dependencies` and `bsl_dependencies` before installing.
- **Keep repositories up to date** with `buffy --repo refresh` to get the latest package metadata.

---

## Thank You

We appreciate the security research community's help in keeping Buffy safe. If you report a valid security vulnerability, we will:

- Credit you in the security advisory and release notes (unless you prefer to remain anonymous)
- List you in our acknowledgments section (with your permission)
- Prioritize your report for timely resolution

Thank you for responsible disclosure.
