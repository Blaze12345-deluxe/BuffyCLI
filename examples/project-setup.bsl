VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Scaffolds a new project with directory structure."
OUTPUT = false

// Check for required argument
WRITE "Creating project: ${1}"
WRITE ""

// Create project directory structure
RUN "mkdir -p ${1}/src"
RUN "mkdir -p ${1}/tests"
RUN "mkdir -p ${1}/docs"

// Create common files
WRITE "Creating README.md..."
RUN "echo '# ${1}' > ${1}/README.md"
RUN "echo '## Description' >> ${1}/README.md"
RUN "echo '## Installation' >> ${1}/README.md"
RUN "echo '## Usage' >> ${1}/README.md"

WRITE "Creating .gitignore..."
RUN "echo '.venv/' > ${1}/.gitignore"
RUN "echo '__pycache__/' >> ${1}/.gitignore"
RUN "echo '*.pyc' >> ${1}/.gitignore"
RUN "echo '.env' >> ${1}/.gitignore"
RUN "echo 'target/' >> ${1}/.gitignore"
RUN "echo '.DS_Store' >> ${1}/.gitignore"
RUN "echo 'dist/' >> ${1}/.gitignore"

WRITE "Creating Makefile..."
RUN "echo 'all: test' > ${1}/Makefile"
RUN "echo '' >> ${1}/Makefile"
RUN "echo 'test:' >> ${1}/Makefile"
RUN "echo '\techo Running tests...' >> ${1}/Makefile"
RUN "echo '' >> ${1}/Makefile"
RUN "echo 'clean:' >> ${1}/Makefile"
RUN "echo '\trm -rf dist/' >> ${1}/Makefile"

WRITE "Creating LICENSE..."
RUN "echo 'MIT License' > ${1}/LICENSE"
RUN "echo 'Copyright (c) ${DATE}' >> ${1}/LICENSE"

WRITE ""
WRITE "========================================="
WRITE "  Project '${1}' Created!"
WRITE "========================================="
WRITE ""
WRITE "  Location: ${PWD}/${1}"
WRITE ""
WRITE "  To get started:"
WRITE "    cd ${1}"
WRITE "    git init"
WRITE "    git add ."
WRITE "    git commit -m 'Initial commit'"
WRITE ""

EXIT
