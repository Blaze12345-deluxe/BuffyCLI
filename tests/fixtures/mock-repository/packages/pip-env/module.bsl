VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Creates a Python virtual environment."
OUTPUT = false

WRITE "Creating virtual environment..."
RUN "python3 -m venv .venv"
WRITE "Done."
EXIT
