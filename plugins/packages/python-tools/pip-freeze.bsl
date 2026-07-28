VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Generates requirements.txt from installed pip packages"

OUTPUT = false

WRITE "Generating requirements.txt..."
OUTPUT = true
RUN "python3 -m pip freeze > requirements.txt"
OUTPUT = false

WRITE "requirements.txt created with $(wc -l < requirements.txt) packages."
