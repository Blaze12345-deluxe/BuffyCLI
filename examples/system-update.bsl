VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Updates system packages using apt."
OUTPUT = true

CLEAR

WRITE "========================================="
WRITE "  System Update"
WRITE "========================================="
WRITE ""

WRITE "Step 1: Updating package lists..."
RUN "sudo apt update"

WRITE ""
WRITE "Step 2: Upgrading packages..."
RUN "sudo apt upgrade -y"

WRITE ""
WRITE "Step 3: Removing unused packages..."
RUN "sudo apt autoremove -y"

WRITE ""
WRITE "========================================="
WRITE "  Update Complete"
WRITE "========================================="
WRITE ""
WRITE "System update finished successfully."

EXIT
