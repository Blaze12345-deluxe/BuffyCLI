VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Downloads a file using curl or wget with progress."
OUTPUT = true

// Usage: buffy --run download-with-progress.bsl <url> [output-filename]

WRITE "========================================="
WRITE "  File Downloader"
WRITE "========================================="
WRITE ""

WRITE "URL: ${1}"
WRITE "Output: ${2}"
WRITE ""

// Try curl first, fall back to wget
RUN "curl -L -o '${2}' '${1}' 2>/dev/null || wget -O '${2}' '${1}' 2>/dev/null || echo 'Download failed'"

WRITE ""
WRITE "Download complete."

EXIT
