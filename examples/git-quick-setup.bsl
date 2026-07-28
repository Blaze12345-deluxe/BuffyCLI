VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Initializes a Git repository with standard setup."
OUTPUT = false

// Check if Git is available
RUN "git --version"

// Check if already a git repository
RUN "test -d .git && echo 'already a repo' || echo 'not a repo'"

WRITE ""
WRITE "Initializing Git repository..."
WRITE ""

// Initialize the repository
RUN "git init"

// Create a .gitignore if it doesn't exist
WRITE "Creating .gitignore..."
RUN "test -f .gitignore && echo 'exists' || echo '.venv/' >> .gitignore"
RUN "echo '__pycache__/' >> .gitignore"
RUN "echo '*.pyc' >> .gitignore"
RUN "echo '.env' >> .gitignore"
RUN "echo 'node_modules/' >> .gitignore"
RUN "echo 'target/' >> .gitignore"
RUN "echo 'dist/' >> .gitignore"
RUN "echo '.DS_Store' >> .gitignore"

WRITE ""
WRITE "Creating initial commit..."
RUN "git add .gitignore"
RUN "git commit -m 'Initial commit: add .gitignore'"

WRITE ""
WRITE "========================================="
WRITE "  Git Repository Ready!"
WRITE "========================================="
WRITE ""
WRITE "  Next steps:"
WRITE "    git add ."
WRITE "    git commit -m 'Add project files'"
WRITE "    git remote add origin <url>"
WRITE "    git push -u origin main"
WRITE ""

EXIT
