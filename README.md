# Buffy CLI Automation Framework

**Buffy** is a lightweight, cross-platform CLI automation framework. Create and share terminal commands by writing simple text files — no Python, Bash, or programming experience required.

```bash
# Run an installed command
buffy pip-env

# Install packages
buffy --install pip-env

# Run a .bsl file without installing
buffy --run ./my-script.bsl

# Discover packages for tools on your system
buffy --discover
```

---

## Quick Start

### Install

#### Option 1: Install from Source (Recommended)

```bash
git clone https://github.com/Blaze12345-deluxe/BuffyCLI.git
cd BuffyCLI
cargo build --release
sudo cp target/release/buffy /usr/local/bin/
```

#### Option 2: Cargo Install

```bash
cargo install --git https://github.com/Blaze12345-deluxe/BuffyCLI.git
```

#### Option 3: Pre-built Binary

Download a pre-built binary from the [releases page](https://github.com/Blaze12345-deluxe/BuffyCLI/releases).

### Your First Command

1. **Install a package:**

   ```bash
   buffy --install pip-env
   ```

   This installs the `pip-env` package from the [Blaze12345-deluxe/Buffy-Plugins](https://github.com/Blaze12345-deluxe/Buffy-Plugins) repository.

2. **Run it:**

   ```bash
   buffy pip-env
   ```

   Buffy resolves `pip-env` to the installed `.bsl` file, interprets it, and executes each instruction.

### Install from a Local File

```bash
buffy --install ./my-script.bsl
buffy my-script
```

---

## What is BSL?

Buffy Script Language (BSL) is a simple, interpreted scripting language. It's not general-purpose — it's purpose-built for automation.

### Example Script

```bsl
VERSION = "2026.07.27"
AUTHOR = "You"
DESCRIPTION = "Greets the user"

WRITE "Hello, ${USER}!"
RUN "echo 'Current directory: ${PWD}'"
WAIT 2
WRITE "Done!"
```

### Syntax

| Instruction | Description |
|-------------|-------------|
| `VERSION` | Script version (metadata, must be at top) |
| `AUTHOR` | Script author (metadata) |
| `DESCRIPTION` | Short description (metadata) |
| `OUTPUT` | Toggle command output (`true` or `false`) |
| `WRITE "text"` | Print text to terminal |
| `RUN "command"` | Execute a shell command |
| `WAIT <seconds>` | Pause execution |
| `WAIT "prompt>"` | Wait for user input |
| `CLEAR` | Clear the terminal screen |
| `EXIT` | Stop execution early |
| `// comment` | Comments |

### Built-in Variables

| Variable | Description |
|----------|-------------|
| `${HOME}` | User's home directory |
| `${USER}` | Current username |
| `${PWD}` | Current working directory |
| `${TEMP}` | System temp directory |
| `${DATE}` | Current date (YYYY-MM-DD) |
| `${TIME}` | Current time (HH:MM:SS) |
| `${1}`–`${N}` | Script arguments |

---

## Commands

| Flag | Description |
|------|-------------|
| `buffy <command>` | Run an installed BSL command |
| `--list`, `-l` | List installed commands |
| `--install <pkg>` | Install a package |
| `--uninstall <name>` | Remove a package |
| `--run <file.bsl>` | Execute a .bsl file directly |
| `--update-packages` | Update all installed packages |
| `--outdated` | Show outdated packages |
| `--verify <name>` | Verify package SHA-256 integrity |
| `--info <name>` | Show package metadata |
| `--repo [list\|add\|remove\|refresh\|search]` | Manage repositories |
| `--alias [list\|set\|remove\|resolve]` | Manage aliases |
| `--doctor` | Run system diagnostics |
| `--check <file.bsl>` | Validate syntax |
| `--validate <file.bsl>` | Check metadata |
| `--benchmark <file.bsl>` | Measure execution time |
| `--discover` | Auto-detect tools, suggest packages |
| `--logs [clear]` | View or clear logs |
| `--clean` | Clear download cache |
| `--repair` | Fix configuration issues |
| `--reset` | Reset configuration to defaults |
| `--update` | Self-update buffy |
| `--completion <shell>` | Generate shell completion script (bash, zsh, fish) |
| `--version` | Print version |

---

## Package Management

### How it Works

Packages are distributed through GitHub repositories. There is no central registry — you configure which repositories to use.

1. **Default plugin repository:** `https://github.com/Blaze12345-deluxe/Buffy-Plugins`
2. **Add more:** `buffy --repo add https://github.com/your-org/packages`
3. **Search packages:** `buffy --repo search <query>`
4. **Install:** `buffy --install <package-name>`

### Package Structure

A package in a repository looks like:

```
repository/
├── index.json                    # All packages index
└── packages/<name>/
    ├── package.json              # Manifest
    ├── <name>-SHA.txt            # SHA-256 checksum
    ├── module.bsl                # Command file(s)
    └── README.md                 # Documentation (optional)
```

### Dependency Resolution

When installing a package, Buffy:

1. Downloads and verifies the package (SHA-256 integrity check)
2. Checks for name conflicts with existing installations
3. Resolves system dependencies (checks `$PATH`)
4. Automatically installs BSL package dependencies
5. Registers the package in `installed.json`

---

## Configuration

Buffy stores its configuration in `~/.buffy/`:

```
~/.buffy/
├── commands/           # Installed .bsl files
├── packages/           # Cached package metadata
├── cache/              # Temporary downloads
├── logs/               # Execution logs
├── config.json         # User configuration
├── installed.json      # Installed package registry
├── repositories.json   # Repository list
└── aliases.json        # Command aliases + conflict preferences
```

---

## Development

### Prerequisites

- **Rust 1.70+** ([install via rustup](https://rustup.rs/))
- **Linux** (primary target; macOS secondary; Windows tertiary)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/Blaze12345-deluxe/BuffyCLI.git
cd BuffyCLI

# Build
cargo build

# Run tests
cargo test

# Build release binary
cargo build --release

# Install globally (optional)
sudo cp target/release/buffy /usr/local/bin/
```

### Running Tests

```bash
cargo test                    # All tests
cargo test bsl::              # BSL language tests
cargo test package::          # Package manager tests
cargo test repository::       # Repository tests
cargo test config::           # Configuration tests
```

### Project Structure

```
src/
├── main.rs                   # Entry point
├── lib.rs                    # Subsystem initialization
├── cli/                      # CLI parsing and dispatch
├── bsl/                      # BSL interpreter (lexer, parser, executor)
├── package/                  # Package management (install, verify, update)
├── repository/               # GitHub repository integration
├── config/                   # Configuration file management
├── resolver/                 # Command resolution (directory walk)
├── diagnostic/               # Doctor, lint, validate
├── logger/                   # Terminal output formatting
└── error.rs                  # Error types
```

### CI Pipeline

The project uses GitHub Actions for CI:

- Build on push and pull request
- Full test suite execution
- Clippy linting
- Format check

---

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, development setup, testing workflow, and the pull request process.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

## Security

See [SECURITY.md](SECURITY.md) for our security policy and vulnerability reporting process.

---

## License

MIT
