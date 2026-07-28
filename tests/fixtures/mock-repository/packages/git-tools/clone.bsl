VERSION = "2026.06.15"
AUTHOR = "Buffy Community"
DESCRIPTION = "Clones a git repository."
OUTPUT = false

WRITE "Cloning git repository..."
RUN "git clone ${1}"
WRITE "Done."
EXIT
