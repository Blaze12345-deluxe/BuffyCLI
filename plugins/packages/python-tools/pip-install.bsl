VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Installs packages from requirements.txt"

OUTPUT = false

// Check if requirements.txt exists
RUN "test -f requirements.txt"

WRITE "Installing packages from requirements.txt..."
OUTPUT = true
RUN "python3 -m pip install -r requirements.txt"
OUTPUT = false

WRITE ""
WRITE "Installation complete."
