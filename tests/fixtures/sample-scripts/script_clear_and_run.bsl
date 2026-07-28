VERSION = "2026.07.27"
AUTHOR = "Test"
DESCRIPTION = "Tests CLEAR and RUN instructions."
OUTPUT = true

CLEAR
WRITE "Running system info..."
RUN "uname -a"
WRITE "Done."
EXIT
