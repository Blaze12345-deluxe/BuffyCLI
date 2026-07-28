VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Upgrades all installed pip packages"

OUTPUT = false

WRITE "Upgrading all pip packages..."

// First upgrade pip itself
OUTPUT = true
RUN "python3 -m pip install --upgrade pip"
OUTPUT = false

// Then list outdated packages and upgrade each
WRITE "Checking for outdated packages..."
OUTPUT = true
RUN "python3 -m pip list --outdated --format=freeze | cut -d= -f1 | while read -r pkg; do python3 -m pip install --upgrade \"$pkg\"; done"
OUTPUT = false

WRITE ""
WRITE "All packages upgraded."
