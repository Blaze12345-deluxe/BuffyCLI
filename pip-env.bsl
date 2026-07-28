// =======================================================
// Buffy Script Language (BSL)
// Package: pip-env
// Version: 2026.07.27
// Description: Creates a Python virtual environment.
// =======================================================

VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Creates a Python virtual environment in the current directory."

OUTPUT = false

WRITE "Creating Python virtual environment..."

RUN "python3 -m venv .venv"

WRITE "Virtual environment created."

WRITE "Upgrading pip..."

RUN ".venv/bin/python -m pip install --upgrade pip"

WRITE "Creating requirements.txt (if missing)..."

RUN "[ -f requirements.txt ] || touch requirements.txt"

WRITE "Adding .venv to .gitignore (if missing)..."

RUN "[ -f .gitignore ] || touch .gitignore"

RUN "grep -qxF .venv/ .gitignore || echo .venv/ >> .gitignore"

WRITE ""
WRITE "✔ Setup Complete!"
WRITE ""
WRITE "To activate your virtual environment run:"
WRITE ""
WRITE "source .venv/bin/activate"
WRITE ""
WRITE "You should then see (.venv) at the beginning of your terminal prompt."

EXIT
