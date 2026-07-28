"""
training_data.py - BSL Training Corpus

Provides a corpus of BSL script examples organized by category.
Each example includes:
  - The BSL source code
  - A natural language description of what it does
  - Tags for categorization
  - Required arguments (if any)
  - The shell commands it depends on

This corpus is used by bsl_generator.py to match user prompts
to the most relevant script patterns and generate new scripts.
"""

import json
from typing import List, Dict, Optional

# ─── Training Examples ───────────────────────────────────────────────────────

TRAINING_EXAMPLES = [
    # ── System Information ──────────────────────────────────────────────
    {
        "name": "system-info",
        "description": "Displays detailed system information including OS, user, memory, and disk usage.",
        "prompt_keywords": ["system", "info", "information", "details", "specs", "specifications"],
        "tags": ["system", "diagnostics", "info"],
        "dependencies": ["uname", "free", "df"],
        "args": [],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Displays detailed system information.\"
OUTPUT = false

CLEAR

WRITE \"=========================================\"
WRITE \"  System Information\"
WRITE \"=========================================\"
WRITE \"\"

WRITE \"  User:       ${USER}\"
WRITE \"  Home:       ${HOME}\"
WRITE \"  Directory:  ${PWD}\"
WRITE \"  Date:       ${DATE}\"
WRITE \"  Time:       ${TIME}\"
WRITE \"  Temp:       ${TEMP}\"
WRITE \"\"

WRITE \"-----------------------------------------\"
WRITE \"  Operating System\"
WRITE \"-----------------------------------------\"
OUTPUT = true
RUN \"uname -a\"
OUTPUT = false

WRITE \"\"
WRITE \"-----------------------------------------\"
WRITE \"  Memory Usage\"
WRITE \"-----------------------------------------\"
OUTPUT = true
RUN \"free -h | head -3\"
OUTPUT = false

WRITE \"\"
WRITE \"-----------------------------------------\"
WRITE \"  Disk Usage\"
WRITE \"-----------------------------------------\"
OUTPUT = true
RUN \"df -h / | tail -1\"
OUTPUT = false

WRITE \"\"
WRITE \"=========================================\"
EXIT"""
    },
    # ── Python Virtual Environment ──────────────────────────────────────
    {
        "name": "pip-env",
        "description": "Creates a Python virtual environment, upgrades pip, and sets up .gitignore.",
        "prompt_keywords": ["python", "venv", "virtual", "environment", "pip", "python env"],
        "tags": ["python", "development", "setup"],
        "dependencies": ["python3"],
        "args": [],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Creates a Python virtual environment in the current directory.\"
OUTPUT = false

WRITE \"Creating Python virtual environment...\"
RUN \"python3 -m venv .venv\"

WRITE \"Virtual environment created.\"
WRITE \"Upgrading pip...\"
RUN \".venv/bin/python -m pip install --upgrade pip\"

WRITE \"Creating requirements.txt (if missing)...\"
RUN \"test -f requirements.txt || touch requirements.txt\"

WRITE \"Adding .venv to .gitignore (if missing)...\"
RUN \"test -f .gitignore || touch .gitignore\"
RUN \"grep -qxF .venv/ .gitignore || echo .venv/ >> .gitignore\"

WRITE \"\"
WRITE \"Setup Complete!\"
WRITE \"\"
WRITE \"To activate: source .venv/bin/activate\"
WRITE \"\"

EXIT"""
    },
    # ── Project Scaffolding ─────────────────────────────────────────────
    {
        "name": "project-setup",
        "description": "Scaffolds a new project directory with src, tests, docs folders, README, .gitignore, Makefile, and LICENSE.",
        "prompt_keywords": ["project", "scaffold", "create project", "new project", "init", "template"],
        "tags": ["development", "scaffolding", "setup"],
        "dependencies": ["mkdir", "echo"],
        "args": ["project_name"],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Scaffolds a new project with directory structure.\"
OUTPUT = false

WRITE \"Creating project: ${1}\"
WRITE \"\"

RUN \"mkdir -p ${1}/src ${1}/tests ${1}/docs\"

WRITE \"Creating README.md...\"
RUN \"echo '# ${1}' > ${1}/README.md\"
RUN \"echo '## Description' >> ${1}/README.md\"
RUN \"echo '## Installation' >> ${1}/README.md\"
RUN \"echo '## Usage' >> ${1}/README.md\"

WRITE \"Creating .gitignore...\"
RUN \"echo '.venv/' > ${1}/.gitignore\"
RUN \"echo '__pycache__/' >> ${1}/.gitignore\"
RUN \"echo '*.pyc' >> ${1}/.gitignore\"
RUN \"echo '.env' >> ${1}/.gitignore\"
RUN \"echo 'target/' >> ${1}/.gitignore\"
RUN \"echo 'dist/' >> ${1}/.gitignore\"

WRITE \"Creating Makefile...\"
RUN \"echo 'all: test' > ${1}/Makefile\"
RUN \"echo '' >> ${1}/Makefile\"
RUN \"echo 'test:' >> ${1}/Makefile\"
RUN \"echo '	@echo Running tests...' >> ${1}/Makefile\"

WRITE \"Creating LICENSE...\"
RUN \"echo 'MIT License' > ${1}/LICENSE\"
RUN \"echo 'Copyright (c) ${DATE}' >> ${1}/LICENSE\"

WRITE \"\"
WRITE \"=========================================\"
WRITE \"  Project '${1}' Created!\"
WRITE \"=========================================\"
WRITE \"  Location: ${PWD}/${1}\"
WRITE \"\"
WRITE \"  cd ${1} && git init\"
WRITE \"\"

EXIT"""
    },
    # ── Git Quick Setup ─────────────────────────────────────────────────
    {
        "name": "git-quick-setup",
        "description": "Initializes a Git repository, creates a .gitignore, and makes the first commit.",
        "prompt_keywords": ["git", "repository", "init", "git init", "version control", "commit"],
        "tags": ["git", "version control", "setup"],
        "dependencies": ["git"],
        "args": ["project_name"],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Initializes a Git repository with standard setup.\"
OUTPUT = false

RUN \"git --version\"
RUN \"test -d .git && echo 'already a repo' || echo 'not a repo'\"

WRITE \"\"
WRITE \"Initializing Git repository...\"
WRITE \"\"
RUN \"git init\"

WRITE \"Creating .gitignore...\"
RUN \"test -f .gitignore && echo 'exists' || echo '.venv/' >> .gitignore\"
RUN \"echo '__pycache__/' >> .gitignore\"
RUN \"echo '*.pyc' >> .gitignore\"
RUN \"echo '.env' >> .gitignore\"
RUN \"echo 'node_modules/' >> .gitignore\"
RUN \"echo 'target/' >> .gitignore\"
RUN \"echo 'dist/' >> .gitignore\"
RUN \"echo '.DS_Store' >> .gitignore\"

WRITE \"\"
WRITE \"Creating initial commit...\"
RUN \"git add .gitignore\"
RUN \"git commit -m 'Initial commit: add .gitignore'\"

WRITE \"\"
WRITE \"=========================================\"
WRITE \"  Git Repository Ready!\"
WRITE \"=========================================\"
WRITE \"  Next steps:\"
WRITE \"    git add .\"
WRITE \"    git commit -m 'Add project files'\"
WRITE \"    git remote add origin <url>\"
WRITE \"    git push -u origin main\"
WRITE \"\"

EXIT"""
    },
    # ── System Update ───────────────────────────────────────────────────
    {
        "name": "system-update",
        "description": "Updates system packages using apt: update, upgrade, autoremove.",
        "prompt_keywords": ["update", "upgrade", "apt", "system update", "package update", "debian", "ubuntu"],
        "tags": ["system", "update", "apt", "maintenance"],
        "dependencies": ["sudo", "apt"],
        "args": [],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Updates system packages using apt.\"
OUTPUT = true

CLEAR

WRITE \"=========================================\"
WRITE \"  System Update\"
WRITE \"=========================================\"
WRITE \"\"

WRITE \"Step 1: Updating package lists...\"
RUN \"sudo apt update\"

WRITE \"\"
WRITE \"Step 2: Upgrading packages...\"
RUN \"sudo apt upgrade -y\"

WRITE \"\"
WRITE \"Step 3: Removing unused packages...\"
RUN \"sudo apt autoremove -y\"

WRITE \"\"
WRITE \"=========================================\"
WRITE \"  Update Complete\"
WRITE \"=========================================\"
WRITE \"System update finished successfully.\"

EXIT"""
    },
    # ── Docker Cleanup ──────────────────────────────────────────────────
    {
        "name": "docker-cleanup",
        "description": "Cleans up unused Docker resources: stopped containers, unused images, volumes, and build cache.",
        "prompt_keywords": ["docker", "clean", "cleanup", "docker clean", "prune", "containers"],
        "tags": ["docker", "containers", "cleanup", "maintenance"],
        "dependencies": ["docker"],
        "args": [],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Cleans up unused Docker resources.\"
OUTPUT = false

WRITE \"=========================================\"
WRITE \"  Docker Cleanup\"
WRITE \"=========================================\"
WRITE \"\"

WRITE \"Step 1: Checking Docker is running...\"
RUN \"docker info > /dev/null 2>&1 && echo 'Docker is running' || echo 'Docker is not running'\"

WRITE \"\"
WRITE \"Step 2: Stopping all running containers...\"
OUTPUT = true
RUN \"docker stop $(docker ps -q) 2>/dev/null || echo 'No running containers'\"
OUTPUT = false

WRITE \"\"
WRITE \"Step 3: Removing unused containers...\"
OUTPUT = true
RUN \"docker container prune -f\"
OUTPUT = false

WRITE \"\"
WRITE \"Step 4: Removing unused images...\"
OUTPUT = true
RUN \"docker image prune -af\"
OUTPUT = false

WRITE \"\"
WRITE \"Step 5: Removing unused volumes...\"
OUTPUT = true
RUN \"docker volume prune -f\"
OUTPUT = false

WRITE \"\"
WRITE \"Step 6: Removing build cache...\"
OUTPUT = true
RUN \"docker builder prune -af\"
OUTPUT = false

WRITE \"\"
WRITE \"=========================================\"
WRITE \"  Docker Cleanup Complete!\"
WRITE \"=========================================\"

EXIT"""
    },
    # ── Network Diagnostic ──────────────────────────────────────────────
    {
        "name": "network-diagnostic",
        "description": "Runs network diagnostics: DNS lookup, ping test, traceroute, and HTTP connection test.",
        "prompt_keywords": ["network", "diagnostic", "ping", "dns", "traceroute", "connectivity", "net"],
        "tags": ["network", "diagnostics", "troubleshooting"],
        "dependencies": ["ping", "nslookup", "traceroute", "curl"],
        "args": ["hostname_or_ip"],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Runs network diagnostics: ping and traceroute.\"
OUTPUT = false

WRITE \"Network Diagnostic Tool\"
WRITE \"\"
WRITE \"Target: ${1}\"
WRITE \"\"

WRITE \"=========================================\"
WRITE \"  Step 1: DNS Resolution\"
WRITE \"=========================================\"
OUTPUT = true
RUN \"nslookup ${1} 2>/dev/null || host ${1} 2>/dev/null || echo 'DNS lookup tools not available'\"
OUTPUT = false

WRITE \"\"
WRITE \"=========================================\"
WRITE \"  Step 2: Ping Test\"
WRITE \"=========================================\"
OUTPUT = true
RUN \"ping -c 4 ${1} 2>/dev/null || echo 'Ping failed'\"
OUTPUT = false

WRITE \"\"
WRITE \"=========================================\"
WRITE \"  Step 3: Traceroute\"
WRITE \"=========================================\"
OUTPUT = true
RUN \"traceroute ${1} 2>/dev/null || echo 'Traceroute not available'\"
OUTPUT = false

WRITE \"\"
WRITE \"=========================================\"
WRITE \"  Step 4: Connection Test\"
WRITE \"=========================================\"
OUTPUT = true
RUN \"curl -sI https://${1} 2>/dev/null | head -5 || echo 'Connection check failed'\"
OUTPUT = false

WRITE \"\"
WRITE \"=========================================\"
WRITE \"  Diagnostic Complete\"
WRITE \"=========================================\"

EXIT"""
    },
    # ── Backup Directory ────────────────────────────────────────────────
    {
        "name": "backup-directory",
        "description": "Backs up a directory to a compressed tar.gz archive.",
        "prompt_keywords": ["backup", "archive", "compress", "tar", "back up", "save"],
        "tags": ["backup", "archive", "files"],
        "dependencies": ["tar"],
        "args": ["source_directory", "destination_file"],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Backs up a directory to a compressed archive.\"
OUTPUT = false

WRITE \"=========================================\"
WRITE \"  Directory Backup\"
WRITE \"=========================================\"
WRITE \"\"

WRITE \"Source:      ${1}\"
WRITE \"Destination: ${2}\"
WRITE \"\"

RUN \"test -d '${1}' && echo 'Source exists' || echo 'WARNING: Source does not exist'\"

OUTPUT = true
RUN \"tar -czf '${2}' '${1}'\"
OUTPUT = false

WRITE \"\"
WRITE \"Backup created!\"
RUN \"ls -lh '${2}'\"
WRITE \"\"
WRITE \"To restore: tar -xzf '${2}'\"

EXIT"""
    },
    # ── Disk Usage ──────────────────────────────────────────────────────
    {
        "name": "disk-usage",
        "description": "Shows disk usage report: largest subdirectories, largest files, and overall summary.",
        "prompt_keywords": ["disk", "usage", "space", "storage", "du", "disk usage", "size"],
        "tags": ["system", "disk", "storage", "report"],
        "dependencies": ["du", "find"],
        "args": [],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Shows disk usage for the current directory.\"
OUTPUT = false

CLEAR

WRITE \"=========================================\"
WRITE \"  Disk Usage Report\"
WRITE \"=========================================\"
WRITE \"\"

WRITE \"Directory: ${PWD}\"
WRITE \"\"
WRITE \"-----------------------------------------\"
WRITE \"  Top 10 Largest Subdirectories\"
WRITE \"-----------------------------------------\"
OUTPUT = true
RUN \"du -sh */ 2>/dev/null | sort -rh | head -10\"

OUTPUT = false
WRITE \"\"
WRITE \"-----------------------------------------\"
WRITE \"  Top 10 Largest Files\"
WRITE \"-----------------------------------------\"
OUTPUT = true
RUN \"find . -maxdepth 2 -type f -exec du -sh '{}' ';' 2>/dev/null | sort -rh | head -10\"

OUTPUT = false
WRITE \"\"
WRITE \"-----------------------------------------\"
WRITE \"  Summary\"
WRITE \"-----------------------------------------\"
OUTPUT = true
RUN \"du -sh .\"
OUTPUT = false

WRITE \"\"
WRITE \"=========================================\"
WRITE \"  Report Complete\"
WRITE \"=========================================\"

EXIT"""
    },
    # ── Download File ───────────────────────────────────────────────────
    {
        "name": "download-file",
        "description": "Downloads a file using curl or wget with progress output.",
        "prompt_keywords": ["download", "curl", "wget", "fetch", "get file", "download file"],
        "tags": ["download", "network", "files"],
        "dependencies": ["curl", "wget"],
        "args": ["url", "output_filename"],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Downloads a file using curl or wget with progress.\"
OUTPUT = true

WRITE \"=========================================\"
WRITE \"  File Downloader\"
WRITE \"=========================================\"
WRITE \"\"

WRITE \"URL: ${1}\"
WRITE \"Output: ${2}\"
WRITE \"\"

RUN \"curl -L -o '${2}' '${1}' 2>/dev/null || wget -O '${2}' '${1}' 2>/dev/null || echo 'Download failed'\"

WRITE \"\"
WRITE \"Download complete.\"

EXIT"""
    },
    # ── Find Large Files ────────────────────────────────────────────────
    {
        "name": "find-large-files",
        "description": "Finds files larger than a specified size in the current directory.",
        "prompt_keywords": ["find", "large", "files", "search", "big files", "large files"],
        "tags": ["files", "search", "disk"],
        "dependencies": ["find", "du"],
        "args": ["min_size_mb"],
        "source": """VERSION = \"2026.07.28\"
AUTHOR = \"Buffy Community\"
DESCRIPTION = \"Finds files larger than a specified size (default: 10MB).\"
OUTPUT = true

WRITE \"=========================================\"
WRITE \"  Finding Large Files\"
WRITE \"=========================================\"
WRITE \"\"

WRITE \"Searching for files larger than ${1:-10}MB in ${PWD}\"
WRITE \"\"

RUN \"find . -type f -size +${1:-10}M -exec ls -lh '{}' ';' 2>/dev/null | sort -rh -k5 | head -20\"

WRITE \"\"
WRITE \"Search complete.\"

EXIT"""
    },
]


# ── Pattern Templates for Script Generation ─────────────────────────────────

PATTERN_TEMPLATES = {
    "system_info": {
        "structure": [
            "metadata",
            "clear_screen",
            "header",
            "display_variables (USER, HOME, PWD, DATE, TIME)",
            "section (OS info)",
            "run_command (uname -a)",
            "section (Memory)",
            "run_command (free -h)",
            "section (Disk)",
            "run_command (df -h)",
            "footer",
            "exit",
        ],
        "variables_used": ["${USER}", "${HOME}", "${PWD}", "${DATE}", "${TIME}", "${TEMP}"],
    },
    "step_by_step": {
        "structure": [
            "metadata",
            "header",
            "step_narratives",
            "run_commands (with OUTPUT toggles)",
            "footer",
            "exit",
        ],
        "variables_used": ["${1}", "${2}"],
    },
    "file_operation": {
        "structure": [
            "metadata",
            "header",
            "argument_display",
            "validation_checks (test -d, test -f)",
            "run_commands",
            "completion_message",
            "exit",
        ],
        "variables_used": ["${1}", "${2}", "${PWD}"],
    },
    "diagnostic": {
        "structure": [
            "metadata",
            "header",
            "argument_display",
            "multiple_sections (each with header + run_command)",
            "summary_section",
            "exit",
        ],
        "variables_used": ["${1}", "${DATE}", "${TIME}"],
    },
}

# ── Instruction Patterns ────────────────────────────────────────────────────

INSTRUCTION_PATTERNS = {
    "WRITE": {
        "description": "Display a text message to the user",
        "pattern": 'WRITE "message"',
        "usage_notes": [
            "Use for headers, progress messages, and results",
            "Empty WRITE \"\" prints a blank line",
            "Variables like ${HOME} are expanded automatically",
        ],
    },
    "RUN": {
        "description": "Execute a shell command",
        "pattern": 'RUN "shell command"',
        "usage_notes": [
            "Use for all system commands",
            "Wrap arguments in quotes: RUN \"mkdir '${1}'\"",
            "OUTPUT = true shows output; OUTPUT = false hides it",
            "Non-zero exit stops execution immediately",
        ],
    },
    "WAIT": {
        "description": "Pause execution for N seconds or until user presses Enter",
        "pattern": 'WAIT 5  or  WAIT "Press Enter to continue..."',
        "usage_notes": [
            "WAIT with a number pauses for that many seconds",
            "WAIT with a string shows the message and waits for Enter",
        ],
    },
    "CLEAR": {
        "description": "Clear the terminal screen",
        "pattern": "CLEAR",
        "usage_notes": [
            "Use at the start of scripts that show formatted output",
        ],
    },
    "EXIT": {
        "description": "Stop script execution immediately",
        "pattern": "EXIT",
        "usage_notes": [
            "Use at the end of every script",
            "Use for early exit after displaying results",
        ],
    },
    "OUTPUT": {
        "description": "Toggle whether RUN command output is shown",
        "pattern": "OUTPUT = true  or  OUTPUT = false",
        "usage_notes": [
            "Set OUTPUT = false at the start for automation scripts",
            "Toggle to true for specific commands the user should see",
            "Can be set multiple times throughout the script",
        ],
    },
}


def get_examples_by_tag(tag: str) -> List[Dict]:
    """Return all training examples that match a given tag."""
    return [ex for ex in TRAINING_EXAMPLES if tag in ex["tags"]]


def get_examples_by_keyword(keyword: str) -> List[Dict]:
    """Return all training examples that match a keyword (case-insensitive)."""
    keyword_lower = keyword.lower()
    results = []
    for ex in TRAINING_EXAMPLES:
        if any(keyword_lower in kw.lower() for kw in ex["prompt_keywords"]):
            results.append(ex)
        elif keyword_lower in ex["description"].lower():
            results.append(ex)
        elif keyword_lower in ex["name"].lower():
            results.append(ex)
    return results


def get_all_tags() -> List[str]:
    """Return a sorted list of all unique tags in the corpus."""
    tags = set()
    for ex in TRAINING_EXAMPLES:
        tags.update(ex["tags"])
    return sorted(tags)


def get_all_dependencies() -> List[str]:
    """Return a sorted list of all unique shell dependencies."""
    deps = set()
    for ex in TRAINING_EXAMPLES:
        deps.update(ex["dependencies"])
    return sorted(deps)


def summary() -> Dict:
    """Return a summary of the training corpus."""
    return {
        "total_examples": len(TRAINING_EXAMPLES),
        "total_tags": len(get_all_tags()),
        "total_dependencies": len(get_all_dependencies()),
        "tags": get_all_tags(),
        "dependencies": get_all_dependencies(),
        "script_names": [ex["name"] for ex in TRAINING_EXAMPLES],
        "template_count": len(PATTERN_TEMPLATES),
    }


if __name__ == "__main__":
    print(json.dumps(summary(), indent=2))
