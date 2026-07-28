VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Removes all __pycache__ directories recursively"

OUTPUT = false

WRITE "Searching for __pycache__ directories..."
OUTPUT = true
RUN "find . -type d -name '__pycache__' -exec rm -rf {} + 2>/dev/null; echo 'Cleanup complete.'"
OUTPUT = false

WRITE "All __pycache__ directories removed."
