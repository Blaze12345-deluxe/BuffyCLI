VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Restarts Docker Compose services"

OUTPUT = false

WRITE "Restarting Docker Compose services..."
OUTPUT = true
RUN "docker compose restart"
OUTPUT = false

WRITE ""
WRITE "Services restarted."
