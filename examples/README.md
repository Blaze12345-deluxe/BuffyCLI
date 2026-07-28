# Buffy BSL Example Scripts

This directory contains real-world BSL (Buffy Script Language) examples demonstrating common automation tasks. Each script is a complete, ready-to-run `.bsl` file.

## How to Run

```bash
# Run directly without installing:
buffy --run examples/system-info.bsl

# Or install locally and run by name:
buffy --install ./examples/project-setup.bsl
buffy project-setup
```

## Examples

| File | Description | Features Demonstrated |
|------|-------------|----------------------|
| `pip-env.bsl` | Python virtual environment setup | `RUN`, `WRITE`, variables |
| `system-info.bsl` | Display system information | `${HOME}`, `${USER}`, `${DATE}`, `${TIME}`, `CLEAR` |
| `system-update.bsl` | Update system packages | `RUN` with `WAIT`, `WRITE` |
| `git-quick-setup.bsl` | Initialize a git repository | `RUN`, `${1}`, `WRITE` |
| `project-setup.bsl` | Full project scaffold | `RUN`, `WAIT`, args, `EXIT` |
| `docker-cleanup.bsl` | Clean Docker resources | `RUN`, `OUTPUT=true`, `WAIT` |
| `backup-directory.bsl` | Backup a directory to archive | `${1}`, `${2}`, `RUN` |
| `disk-usage.bsl` | Check disk usage | `RUN`, `WRITE`, `WAIT` |
| `network-diagnostic.bsl` | Ping and traceroute | `RUN`, `WRITE`, `${1}`, `WAIT` |
| `download-with-progress.bsl` | Download with progress bar | `RUN`, `OUTPUT=true` |

## Writing Your Own

Each script starts with metadata:

```bsl
VERSION = "2026.07.27"
AUTHOR = "Your Name"
DESCRIPTION = "What this script does"
OUTPUT = false
```

Then instructions:

```bsl
WRITE "Doing something..."
RUN "some-command"
WAIT 2
WRITE "Done."
EXIT
```

See the [BSL Language Spec](../docs/Buffy-Script-Language-Spec.txt) for full documentation.
