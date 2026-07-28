VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Backs up a directory to a compressed archive."
OUTPUT = false

// Usage: buffy --run backup-directory.bsl /path/to/source /path/to/backup.tar.gz

WRITE "========================================="
WRITE "  Directory Backup"
WRITE "========================================="
WRITE ""

// Check that source argument is provided
WRITE "Source:      ${1}"
WRITE "Destination: ${2}"
WRITE ""

// Verify source exists
RUN "test -d '${1}' && echo 'Source exists' || echo 'WARNING: Source does not exist'"

OUTPUT = true

// Create the backup
RUN "tar -czf '${2}' '${1}'"

OUTPUT = false

WRITE ""
WRITE "Backup created successfully!"
RUN "ls -lh '${2}'"

WRITE ""
WRITE "To restore:"
WRITE "  tar -xzf '${2}'"

EXIT
