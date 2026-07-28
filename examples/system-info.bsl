VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Displays detailed system information."
OUTPUT = false

CLEAR

WRITE "========================================="
WRITE "  System Information"
WRITE "========================================="
WRITE ""

WRITE "  User:       ${USER}"
WRITE "  Home:       ${HOME}"
WRITE "  Directory:  ${PWD}"
WRITE "  Date:       ${DATE}"
WRITE "  Time:       ${TIME}"
WRITE "  Temp:       ${TEMP}"
WRITE ""

WRITE "-----------------------------------------"
WRITE "  Operating System"
WRITE "-----------------------------------------"
RUN "uname -a"

WRITE ""
WRITE "-----------------------------------------"
WRITE "  Memory Usage"
WRITE "-----------------------------------------"
RUN "free -h | head -3"

WRITE ""
WRITE "-----------------------------------------"
WRITE "  Disk Usage"
WRITE "-----------------------------------------"
RUN "df -h / | tail -1"

WRITE ""
WRITE "========================================="
EXIT
