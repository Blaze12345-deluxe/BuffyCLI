VERSION = "2026.07.27"
AUTHOR = "Test"
DESCRIPTION = "Tests variable resolution in BSL."
OUTPUT = false

WRITE "Home: ${HOME}"
WRITE "User: ${USER}"
WRITE "Current dir: ${PWD}"
WRITE "Date: ${DATE}"
WRITE "Time: ${TIME}"
WRITE "Arg 1: ${1}"
WRITE "Arg 2: ${2}"
EXIT
