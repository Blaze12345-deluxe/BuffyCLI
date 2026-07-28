VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Shows disk usage for the current directory."
OUTPUT = false

CLEAR

WRITE "========================================="
WRITE "  Disk Usage Report"
WRITE "========================================="
WRITE ""

WRITE "Current directory: ${PWD}"
WRITE ""
WRITE "-----------------------------------------"
WRITE "  Top 10 Largest Subdirectories"
WRITE "-----------------------------------------"

OUTPUT = true
RUN "du -sh */ 2>/dev/null | sort -rh | head -10"

OUTPUT = false
WRITE ""
WRITE "-----------------------------------------"
WRITE "  Top 10 Largest Files"
WRITE "-----------------------------------------"

OUTPUT = true
RUN "find . -maxdepth 2 -type f -exec du -sh '{}' ';' 2>/dev/null | sort -rh | head -10"

OUTPUT = false
WRITE ""
WRITE "-----------------------------------------"
WRITE "  Overall Summary"
WRITE "-----------------------------------------"

OUTPUT = true
RUN "du -sh ."

OUTPUT = false
WRITE ""
WRITE "========================================="
WRITE "  Report Complete"
WRITE "========================================="

EXIT
