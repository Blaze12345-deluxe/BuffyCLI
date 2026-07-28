# Buffy Package Format Specification

> **Version:** 1.1
> **Last Updated:** 2026-07-27
> **Status:** Draft

---

## Table of Contents

1. [Overview](#1-overview)
2. [Package Structure](#2-package-structure)
3. [Manifest File (package.json)](#3-manifest-file-packagejson)
4. [Command Resolution](#4-command-resolution)
5. [Package Installation](#5-package-installation)
6. [Package Verification](#6-package-verification)
7. [Versioning](#7-versioning)
8. [Dependencies](#8-dependencies)
9. [Tags & Discoverability](#9-tags--discoverability)
10. [Repository Index](#10-repository-index)
11. [Name Conflict Resolution](#11-name-conflict-resolution)
12. [Repository System](#12-repository-system)
13. [Update Mechanism](#13-update-mechanism)
14. [Diagnostics & Repair](#14-diagnostics--repair)
15. [Installed Packages Database](#15-installed-packages-database)
16. [Package Info Display](#16-package-info-display)
17. [Benchmarking](#17-benchmarking)
18. [Aliases & Overrides](#18-aliases--overrides)
19. [Appendix: Example Package](#19-appendix-example-package)

---

## 1. Overview

Buffy packages are the distribution unit for BSL commands. Each package contains one or more BSL scripts plus metadata describing the package, its dependencies, and its integrity.

### Design Principles

- **Linux-first** — BSL scripts target Linux as the primary platform. macOS is secondary. Windows support is tertiary.
- **Package = collection of commands** — a single package can contain multiple related BSL commands (e.g., `pip-env` could include `create`, `activate`, `destroy`).
- **Safety through verification** — every package is verified via SHA-256 before installation, with a 3-way comparison between the generated hash, the hash in `package.json`, and the hash in a separate checksum file.
- **No central registry** — packages are distributed through GitHub repositories (official, community, or private). There is no single package registry.

---

## 2. Package Structure

### 2.1 Repository Layout

Every package repository follows this layout:

```
<repository-root>/
│
├── index.json                    # Repository index (all packages)
│
└── packages/
    ├── pip-env/
    │   ├── package.json          # Package manifest (required)
    │   ├── pip-env-SHA.txt       # SHA-256 checksum (required)
    │   ├── create.bsl            # BSL command (at least one required)
    │   ├── activate.bsl          # BSL command (optional)
    │   ├── destroy.bsl           # BSL command (optional)
    │   ├── README.md             # Documentation (optional)
    │   └── LICENSE               # License file (optional)
    │
    └── docker-tools/
        ├── package.json
        ├── docker-tools-SHA.txt
        ├── up.bsl
        ├── down.bsl
        └── README.md
```

### 2.2 Local Package (Standalone File)

A single `.bsl` file can be installed from the local filesystem without a full package directory:

```
./pip-env.bsl    # Standalone BSL script
```

When installed, Buffy auto-generates a minimal `package.json`.

### 2.3 Directory Naming Convention

Packages are stored in `~/.buffy/commands/` using a **subdirectory-per-segment** convention. Each CLI argument maps to a subdirectory level:

```
~/.buffy/commands/
│
├── pip-env/
│   ├── create.bsl          # buffy pip-env create
│   └── activate.bsl        # buffy pip-env activate
│
├── restart/
│   └── docker.bsl          # buffy restart docker
│
└── docker/
    └── compose/
        └── up.bsl           # buffy docker compose up
```

When installing from a repository, the package directory name becomes the root command. Any additional `.bsl` files inside it become subcommands.

### 2.4 File Naming

- BSL scripts use the `.bsl` extension.
- Filenames are **arbitrary** — any valid filename can be used (not limited to `module.bsl`).
- The filename (minus extension) becomes the command name for invocation.
- Subcommands can be nested as deeply as needed.

### 2.5 Asset Files

Non-BSL files (templates, config stubs, helper scripts) declared in `package.json`'s `assets` field are installed **alongside** the `.bsl` files in the same command directory. Their relative paths from the package root are preserved.

Example: an asset at `templates/requirements.txt` is installed to `~/.buffy/commands/pip-env/templates/requirements.txt`.

---

## 3. Manifest File (package.json)

### 3.1 Required Fields

```json
{
    "name": "pip-env",
    "version": "2026.07.27",
    "description": "Creates and manages Python virtual environments.",
    "author": "Buffy Community",
    "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | ✅ | Package name (used for installation, update, uninstall) |
| `version` | string | ✅ | Date-based version (YYYY.MM.DD) |
| `description` | string | ✅ | Short description shown by `buffy --info` |
| `author` | string | ✅ | Creator name or organization |
| `sha256` | string | ✅ | SHA-256 hash of the package contents (excluding `package.json` and `SHA.txt`) |

### 3.2 Optional Fields

```json
{
    "tags": ["python", "venv", "setup"],
    "dependencies": {
        "system": ["python3", "python3-venv"],
        "packages": ["git-tools"]
    },
    "assets": [
        "templates/requirements.txt",
        "configs/.gitignore.stub"
    ],
    "license": "MIT",
    "homepage": "https://github.com/BuffyCLI/packages/tree/main/packages/pip-env"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `tags` | string[] | Keywords for discoverability and `--discover` matching |
| `dependencies.system` | string[] | System tools that must be installed (e.g., `python3`, `docker`) |
| `dependencies.packages` | string[] | Other BSL packages required (installed automatically) |
| `assets` | string[] | Non-BSL files included in the package (templates, configs, etc.) |
| `license` | string | SPDX license identifier |
| `homepage` | string | URL to the package's home page |

### 3.3 Auto-Generated Package.json

When installing from a local `.bsl` file (no `package.json`), Buffy auto-generates:

```json
{
    "name": "pip-env",
    "version": "2026.07.27",
    "description": "Local BSL script installed from ./pip-env.bsl",
    "author": "unknown",
    "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
}
```

- `name` — derived from the `.bsl` filename
- `version` — current date at install time
- `description` — indicates it was a local installation
- `author` — set to `unknown`
- `sha256` — generated from the `.bsl` file contents

---

## 4. Command Resolution

### 4.1 Resolution Algorithm

When the user runs a BSL command (no flags):

```
buffy pip-env create
```

Buffy resolves the command by walking `~/.buffy/commands/`:

1. Split the remaining arguments into segments: `["pip-env", "create"]`
2. Walk the directory tree:
   - `commands/pip-env/` — exists ✓
   - `commands/pip-env/create.bsl` — exists ✓ → execute
3. If the final segment is a directory with no matching file, look for a default entry.

### 4.2 Resolution Priority

```
1. Exact file match       commands/pip-env/create.bsl
2. index.bsl default      commands/pip-env/index.bsl        (if directory invoked alone)
3. Directory name match   commands/pip-env/pip-env.bsl      (matches the dir name)
4. First alphabetically   commands/pip-env/activate.bsl     (fallback)
5. Error                  "Command not found: pip-env create"
```

### 4.3 Default Subcommand via `index.bsl`

If `index.bsl` exists in a command directory, it serves as the **default subcommand** when no specific subcommand is given:

```
buffy pip-env
```

This executes `commands/pip-env/index.bsl` (if it exists).

This is purely a filename convention — `index.bsl` is treated like any other BSL file. It does not receive special argument handling.

### 4.4 Displaying Available Subcommands

If a directory is resolved but no specific subcommand is given and no `index.bsl` exists:

```
buffy pip-env
```

Buffy lists the available subcommands:

```
pip-env commands:

  create     Creates a Python virtual environment
  activate   Activates an existing virtual environment
  destroy    Removes a virtual environment
```

---

## 5. Package Installation

### 5.1 Installation Sources

Packages can be installed from four sources, resolved in this order:

| Syntax | Source | Example |
|--------|--------|---------|
| `./` prefix | Local filesystem | `buffy --install ./pip-env.bsl` |
| `github.com/...` or `https://github.com/...` | Specific GitHub repo | `buffy --install github.com/user/repo pip-env` |
| `@` | All packages from a repo | `buffy --install github.com/user/repo @` |
| Plain name | Configured repositories | `buffy --install pip-env` |

### 5.2 Multiple Package Installation

When installing multiple packages in one command, Buffy installs all that can be resolved and skips any that are not found:

```
buffy --install pip-env docker-tools nonexistent-pkg
```

Output:

```
Installing 3 packages...

✔ pip-env           installed
✔ docker-tools      installed
✘ nonexistent-pkg   not found in any repository

2 of 3 packages installed successfully.
```

The command exits successfully if at least one package was installed. Not-found packages are reported but do not block valid installations.

### 5.3 Install Flow

```
buffy --install pip-env
```

1. **Parse source** — determine whether local file, GitHub URL, or configured repo
2. **Locate package** — download index.json from repos, find matching package
3. **Download** — fetch package.json + .bsl files + SHA.txt + assets from the source
4. **Verify** — generate SHA-256 hash, compare against both package.json and SHA.txt
5. **Validate** — check package.json is valid, at least one .bsl file exists
6. **Install** — copy files to `~/.buffy/commands/<name>/`
7. **Register** — add entry to `installed.json`
8. **Report** — show success message

### 5.4 Install Output

```
Installing "pip-env"...

Downloading package...
Verifying package...
  ✔ SHA-256 verified (3-way match)
Installing...
  → create.bsl
  → activate.bsl
  → destroy.bsl

Done.

Package installed successfully.
  Version: 2026.07.27
  Commands: pip-env create, pip-env activate, pip-env destroy
```

### 5.5 Local Install Flow

```
buffy --install ./pip-env.bsl
```

1. Verify file exists and has `.bsl` extension
2. Parse and validate syntax
3. Auto-generate `package.json`
4. Install to `~/.buffy/commands/pip-env/pip-env.bsl`
5. Register in `installed.json` with `source: "local"`
6. Original file is never modified

### 5.6 GitHub Install Flow

```
buffy --install github.com/Blaze1234-deluxe/custom-buffy-plugins pip-env
```

1. Connect to the specified GitHub repository
2. Download `packages/pip-env/package.json` and `packages/pip-env/pip-env-SHA.txt`
3. Verify SHA-256 (3-way comparison)
4. Download all `.bsl` files and assets declared in `package.json`
5. Install to `~/.buffy/commands/pip-env/`
6. Register with `source: "github.com/Blaze1234-deluxe/custom-buffy-plugins"`
7. The repository is used only for this install unless added to `repositories.json`

### 5.7 Wildcard Install

```
buffy --install github.com/Blaze1234-deluxe/custom-buffy-plugins @
```

Installs every available package from the specified repository. The `@` symbol is reserved exclusively for this purpose.

Output:

```
Found 84 packages.

Installing...

✔ apt-update
✔ docker-compose
✔ docker-restart
✔ git-clone
✔ pip-env
...

84 packages installed successfully.
```

### 5.8 Dependency Installation

If a package declares `dependencies.packages`, Buffy automatically installs those dependencies first (from the same source or configured repos):

```
Installing "docker-compose"...
  Dependency: "docker-core" not installed.
  Installing "docker-core"...
  ✔ docker-core installed.
✔ docker-compose installed.
```

If a dependency installation fails, Buffy **retries once** automatically. If the retry also fails, the installation is aborted and the error is reported.

---

## 6. Package Verification

### 6.1 Verification Files

Every package must include its SHA-256 hash in **two locations**:

1. **Inside `package.json`** — the `sha256` field
2. **Separate `{package-name}-SHA.txt` file** — in the package directory

Example `pip-env-SHA.txt`:

```
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  pip-env
```

### 6.2 Verification Flow

When verifying (on install, update, or `--verify`):

1. **Generate hash** — compute SHA-256 of the package's `.bsl` files and declared assets (sorted alphabetically, concatenated)
2. **Compare against `package.json`** — does the generated hash match the `sha256` field?
3. **Compare against `{package-name}-SHA.txt`** — does the generated hash match the contents of the checksum file?
4. **All 3 must match** — if any comparison fails, verification fails and installation is rejected

```
Verifying pip-env...

  Generated:    e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
  package.json: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ✔
  SHA.txt:      e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ✔

  ✔ Verified (3-way match)
```

### 6.3 What Gets Hashed

The SHA-256 hash is computed over:

- All `.bsl` files in the package directory (sorted alphabetically by relative path)
- All asset files declared in `package.json` (sorted alphabetically by relative path)

The following files are **excluded** from the hash:

- `package.json` itself (preventing circular verification)
- `{package-name}-SHA.txt` itself (self-verifying via the other two)

Files are hashed in sorted order, concatenated, then the combined content is hashed once.

---

## 7. Versioning

### 7.1 Scheme

Packages use a **date-based versioning** scheme:

```
YYYY.MM.DD
```

Examples:

| Version | Date |
|---------|------|
| `2026.07.27` | July 27, 2026 |
| `2026.01.01` | January 1, 2026 |
| `2025.12.25` | December 25, 2025 |

### 7.2 Comparison

Version comparison is numeric by component:

1. Compare year (higher = newer)
2. If equal, compare month (higher = newer)
3. If equal, compare day (higher = newer)

```
2026.07.27 > 2026.07.26 > 2026.06.30 > 2025.12.25
```

### 7.3 Multiple Releases Same Day

If a package is updated multiple times on the same day, a build number can be appended:

```
2026.07.27.1
2026.07.27.2
```

The build number is compared after the date components.

---

## 8. Dependencies

### 8.1 System Dependencies

`dependencies.system` lists external tools that must be available on the system:

```json
{
    "dependencies": {
        "system": ["python3", "python3-venv"]
    }
}
```

When a package with system dependencies is installed, Buffy checks if each tool exists in `$PATH`. Missing tools are reported as warnings (not errors), since the user may install them later.

```
Warning: Package "pip-env" requires:
  • python3        → not found in $PATH
  • python3-venv   → not found in $PATH

Install system dependencies manually to use this package.
```

### 8.2 Package Dependencies

`dependencies.packages` lists other BSL packages that must be installed:

```json
{
    "dependencies": {
        "packages": ["git-tools"]
    }
}
```

When installing a package with BSL dependencies:

1. Check if each dependency is already installed
2. If not, automatically install it from the same source (or configured repos)
3. Install dependencies before the requested package
4. If a dependency cannot be resolved, retry once; if still failing, abort with an error message

---

## 9. Tags & Discoverability

### 9.1 Tags Field

Packages declare tags in `package.json` for discoverability:

```json
{
    "tags": ["python", "venv", "setup", "virtual-environment"]
}
```

Tags serve two purposes:
1. **Repository search** — users can search by tag
2. **System discovery** — the `--discover` feature matches tags against detected software

### 9.2 Discover Flow (`buffy --discover`)

The discover feature scans the user's system for installed software, then suggests packages with matching tags.

**Algorithm:**
1. Scan `$PATH` for common tools (git, docker, python3, node, java, ffmpeg, etc.)
2. For each detected tool, look up packages with matching tags across configured repositories
3. Show a grouped summary:

```
Scanning system...

  ✔ Python 3.13 found
    Suggested: pip-env (create/manage virtual environments)

  ✔ Docker found
    Suggested: docker-compose-up, docker-compose-down, docker-restart

  ✔ Git found
    Suggested: git-clone, git-clean, git-branch

Install all suggested packages? (Y/n)
```

If the user selects `Y`, all suggested packages are installed. If `n`, nothing is installed.

### 9.3 Tag Naming Conventions

Tags should match tool names where applicable:

| Software | Tag |
|----------|-----|
| Python | `python` |
| Docker | `docker` |
| Git | `git` |
| Node.js | `node` |
| Java | `java` |
| FFmpeg | `ffmpeg` |

Use lowercase, single words when possible.

---

## 10. Repository Index

### 10.1 Index Location

Every repository should include an `index.json` at its root:

```
https://github.com/BuffyCLI/packages/
├── index.json
└── packages/
```

### 10.2 Index Format

```json
{
    "packages": [
        {
            "name": "pip-env",
            "version": "2026.07.27",
            "description": "Creates and manages Python virtual environments.",
            "author": "Buffy Community",
            "path": "packages/pip-env",
            "dependencies": {
                "system": ["python3", "python3-venv"],
                "packages": []
            },
            "tags": ["python", "venv", "setup"],
            "commands": ["pip-env create", "pip-env activate", "pip-env destroy"]
        }
    ],
    "meta": {
        "updated": "2026-07-27T14:30:00Z",
        "package_count": 42
    }
}
```

### 10.3 Index Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | ✅ | Package name |
| `version` | string | ✅ | Latest version in YYYY.MM.DD format |
| `description` | string | ✅ | Short description |
| `author` | string | ✅ | Package author |
| `path` | string | ✅ | Relative path within the repository |
| `dependencies.system` | string[] | No | System tool requirements |
| `dependencies.packages` | string[] | No | BSL package dependencies |
| `tags` | string[] | No | Discoverability tags |
| `commands` | string[] | No | List of commands the package provides |

### 10.4 Index Usage

Buffy downloads the `index.json` first instead of scanning the repository. This allows:

- Quick version comparison across multiple repositories
- Client-side search/filtering without API calls
- Efficient wildcard install (`@`) without per-package API requests

---

## 11. Name Conflict Resolution

### 11.1 Conflict Detection

A name conflict occurs when:

- Two packages with the **same name** are installed
- They have **different SHA-256 hashes** (i.e., they are genuinely different packages from different authors)
- They were installed from **different sources/repositories**

### 11.2 Conflict Resolution Flow

When a name conflict exists, `buffy <command>` displays:

```
Multiple packages found for "pip-env":

  1. pip-env  (by: Buffy Community)
     SHA: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
     Source: official repository

  2. pip-env  (by: John Smith)
     SHA: a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a
     Source: github.com/JohnSmith/buffy-packages

Which one would you like to execute? (1/2/q):
```

### 11.3 Choice Caching

After the user selects a version, Buffy caches the choice:

```json
// ~/.buffy/aliases.json
{
    "pip-env": {
        "preferred": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "source": "official"
    }
}
```

The cached choice is used for future executions.

### 11.4 Changing the Default

Users can change their preferred version at any time:

```
buffy --alias pip-env
```

This re-prompts the conflict resolution dialog.

Or by directly editing `aliases.json`.

### 11.5 Conflicts Across Repositories

Conflicts can also occur during installation (before the package is installed):

```
buffy --install pip-env
```

If `pip-env` exists in multiple configured repositories, Buffy shows the versions and asks:

```
Package "pip-env" found in 2 repositories:

  1. official  — version 2026.07.27
  2. community — version 2026.06.15

Which version would you like to install? (1/2/q):
```

---

## 12. Repository System

### 12.1 Default Repository

Every Buffy installation ships with one default package repository:

```
https://github.com/BuffyCLI/packages
```

### 12.2 Multiple Repositories

Users can configure multiple repositories in `~/.buffy/repositories.json`:

```json
[
    "https://github.com/BuffyCLI/packages",
    "https://github.com/ExampleUser/linux-tools",
    "https://github.com/MyName/private-packages"
]
```

### 12.3 Repository Aliases

Repositories can be aliased for convenience:

```json
{
    "official": "https://github.com/BuffyCLI/packages",
    "community": "https://github.com/BuffyCommunity/packages",
    "myrepo": "https://github.com/MyName/private-packages"
}
```

Usage:

```
buffy --install myrepo pip-env
```

### 12.4 Private Repositories

Buffy supports installing from private GitHub repositories. Authentication is handled via:

1. The `GITHUB_TOKEN` environment variable
2. Or a token configured in `config.json`:

```json
{
    "github_token": "ghp_..."
}
```

The token must have `contents: read` permission for the target repository.

### 12.5 Repository Verification

When adding a repository, Buffy verifies:

1. The URL is a valid GitHub repository URL
2. The repository is reachable
3. The repository has a valid `index.json`

If any check fails, the repository is rejected with an error message.

---

## 13. Update Mechanism

### 13.1 Package Updates

```
buffy --update-packages          # Update all installed packages
buffy --update-package pip-env   # Update one specific package
```

Update flow:

1. For each package installed from a repository:
   - Check the source repository's `index.json` for a newer version
   - If found, download and install the new version
   - Verify SHA-256 on the new version
   - **Remove stale files** — delete any `.bsl` files that existed in the old version but not in the new one
   - Update `installed.json`
2. Packages installed from local `.bsl` files are **not** automatically updated (no remote source to check)

### 13.2 Buffy CLI Self-Update

```
buffy --update        # Update Buffy CLI itself
buffy --check-update  # Check for updates without applying
```

Self-update is separate from package updates. The `--update` flag checks the Buffy CLI GitHub repository for a new release, downloads it, and replaces the current binary.

### 13.3 Outdated Packages

```
buffy --outdated
```

Shows packages with updates available:

```
Outdated packages:

  pip-env   2026.06.15  →  2026.07.27  (official)
  git-tools  2025.12.01 →  2026.03.10  (community)

Run "buffy --update-packages" to update all.
```

---

## 14. Diagnostics & Repair

### 14.1 Doctor (`buffy --doctor`)

Full system check, checking:

1. `~/.buffy/` directory structure exists (commands, packages, cache, logs)
2. `config.json` is valid JSON
3. `installed.json` is valid — all referenced packages exist on disk
4. `repositories.json` is valid — URLs are reachable
5. Every installed `.bsl` file has valid BSL syntax
6. File permissions on installed scripts
7. Disk space and temp directory writability

### 14.2 Repair (`buffy --repair`)

Verifies each installed package and fixes issues:

1. Check each package's integrity (regenerate SHA-256, compare against both stored hashes)
2. Check that all referenced `.bsl` files still exist on disk
3. Check that `installed.json` entries match actual filesystem state
4. Remove orphaned command directories (not in `installed.json`)
5. Fix broken `config.json` (regenerate with defaults if corrupted)
6. Recreate missing `~/.buffy/` subdirectories
7. Report all actions taken

### 14.3 Clean (`buffy --clean`)

Clears the package cache (`~/.buffy/cache/`):

```
Cleaning cache...

  Deleted 12 cached files.
  Freed 4.2 MB of disk space.
```

---

## 15. Installed Packages Database

### 15.1 Format

`~/.buffy/installed.json` uses a hash-suffixed key scheme to support multiple packages with the same name from different authors:

```json
{
    "pip-env-e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855": {
        "name": "pip-env",
        "version": "2026.07.27",
        "installed": "2026-07-27",
        "source": "official",
        "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "author": "Buffy Community",
        "dependencies": {
            "system": ["python3"],
            "packages": []
        }
    },
    "docker-tools-a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a": {
        "name": "docker-tools",
        "version": "2026.06.15",
        "installed": "2026-07-25",
        "source": "github.com/ExampleUser/docker-packages",
        "sha256": "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
        "author": "Example User",
        "dependencies": {
            "system": ["docker"],
            "packages": []
        }
    }
}
```

### 15.2 Key Format

Each entry's key is `{package-name}-{sha256-first-12-chars}`. The truncated hash provides disambiguation while keeping keys readable:

```
pip-env-e3b0c44298fc
docker-tools-a7ffc6f8bf1e
```

### 15.3 Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | ✅ | Package name |
| `version` | string | ✅ | Installed version |
| `installed` | string | ✅ | ISO date of installation |
| `source` | string | ✅ | Repository source: `"official"`, `"local"`, or `"github.com/..."` |
| `sha256` | string | ✅ | Full SHA-256 hash at time of install |
| `author` | string | ✅ | Package author |
| `dependencies.system` | string[] | No | System tool requirements |
| `dependencies.packages` | string[] | No | BSL package dependencies |

---

## 16. Package Info Display

### 16.1 Info Command

```
buffy --info pip-env
```

Displays metadata for an installed package:

```
Package: pip-env
  Version:     2026.07.27
  Author:      Buffy Community
  Source:      official repository
  Installed:   2026-07-27

  Description:
  Creates and manages Python virtual environments.

  Commands:
    pip-env create     Creates a Python virtual environment
    pip-env activate   Prints activation instructions
    pip-env destroy    Removes a virtual environment

  Dependencies:
    System:  python3, python3-venv
    BSL:     (none)

  Tags: python, venv, setup
  SHA-256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

### 16.2 Fields Displayed

| Field | Source | Always shown? |
|-------|--------|---------------|
| Name | `package.json` | ✅ Yes |
| Version | `package.json` / `installed.json` | ✅ Yes |
| Author | `package.json` | ✅ Yes |
| Source | `installed.json` | ✅ Yes |
| Install date | `installed.json` | ✅ Yes |
| Description | `package.json` | ✅ Yes |
| Commands | Scanned from `.bsl` files | ✅ Yes |
| System deps | `package.json` | Only if non-empty |
| BSL deps | `package.json` | Only if non-empty |
| Tags | `package.json` | Only if non-empty |
| SHA-256 | `package.json` | ✅ Yes |
| License | `package.json` | Only if set |
| Homepage | `package.json` | Only if set |

---

## 17. Benchmarking

### 17.1 Benchmark Command

```
buffy --benchmark pip-env/create.bsl
```

The `--benchmark` flag measures execution time for each line of a BSL script, plus total time:

```
Benchmark: create.bsl

  Line   Instruction            Time
  ─────────────────────────────────────
     1   WRITE                  0.2ms
     2   RUN "python3 -m venv"  1.42s
     3   WRITE                  0.1ms
     4   EXIT                   0.0ms
  ─────────────────────────────────────
  Total                         1.42s
```

Benchmarking can be performed on:
- A specific `.bsl` file path
- An installed command by name

---

## 18. Aliases & Overrides

### 18.1 Command Aliases

`~/.buffy/aliases.json`:

```json
{
    "ve": "pip-env",
    "dcu": "docker compose up",
    "dcd": "docker compose down"
}
```

Usage:

```
buffy ve create    # Equivalent to "buffy pip-env create"
```

### 18.2 Conflict Preference Storage

When a name conflict is resolved (see [Section 11](#11-name-conflict-resolution)), the preference is cached here:

```json
{
    "pip-env": {
        "preferred": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "source": "official"
    },
    "ve": "pip-env"
}
```

### 18.3 Modifying Aliases

Users can set, list, and remove aliases:

```
buffy --alias ve pip-env        # Set alias
buffy --alias                   # List all aliases
buffy --alias ve                # Show what "ve" resolves to
```

### 18.4 No Override Field

There is **no** `replaces` or `priority` field in `package.json`. Name conflicts are resolved through the user-prompt-and-cache mechanism only.

---

## 19. Appendix: Example Package

### 19.1 Repository Structure

```
packages/pip-env/
├── package.json
├── pip-env-SHA.txt
├── index.bsl
├── create.bsl
├── activate.bsl
├── destroy.bsl
├── README.md
└── LICENSE
```

### 19.2 package.json

```json
{
    "name": "pip-env",
    "version": "2026.07.27",
    "description": "Creates and manages Python virtual environments.",
    "author": "Buffy Community",
    "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "tags": ["python", "venv", "setup", "virtual-environment"],
    "dependencies": {
        "system": ["python3", "python3-venv"],
        "packages": []
    },
    "license": "MIT",
    "homepage": "https://github.com/BuffyCLI/packages/tree/main/packages/pip-env"
}
```

### 19.3 pip-env-SHA.txt

```
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  pip-env
```

### 19.4 index.bsl (Default Entry Point)

```bsl
VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "pip-env: Python virtual environment manager"
OUTPUT = false

WRITE "pip-env — Python Virtual Environment Manager"
WRITE ""
WRITE "Available commands:"
WRITE "  buffy pip-env create     Create a virtual environment"
WRITE "  buffy pip-env activate   Print activation instructions"
WRITE "  buffy pip-env destroy    Remove a virtual environment"

EXIT
```

### 19.5 create.bsl

```bsl
VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Creates a Python virtual environment in the current directory."
OUTPUT = false

WRITE "Creating Python virtual environment..."

RUN "python3 -m venv .venv"

WRITE "Virtual environment created."
WRITE ""
WRITE "To activate:"
WRITE "  source .venv/bin/activate"
WRITE ""
WRITE "To destroy:"
WRITE "  buffy pip-env destroy"

EXIT
```

### 19.6 activate.bsl

```bsl
VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Prints activation instructions for the virtual environment."
OUTPUT = false

WRITE "To activate the virtual environment, run:"
WRITE ""
WRITE "  source .venv/bin/activate"
WRITE ""
WRITE "To verify it's active:"
WRITE "  which python"

EXIT
```

### 19.7 Command Resolution

```
buffy pip-env             →  commands/pip-env/index.bsl       (default, via index.bsl)
buffy pip-env create      →  commands/pip-env/create.bsl
buffy pip-env activate    →  commands/pip-env/activate.bsl
buffy pip-env destroy     →  commands/pip-env/destroy.bsl
```

---

> **Document Version:** 1.1  
> **Last Updated:** 2026-07-27  
> **Status:** Draft for review  
> **Design Principle:** Linux-first, macOS secondary, Windows tertiary
