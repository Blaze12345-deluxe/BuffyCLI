VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Stops Docker Compose services"

OUTPUT = false

WRITE "Stopping Docker Compose services..."
OUTPUT = true
RUN "docker compose down"
OUTPUT = false

WRITE ""
WRITE "Services stopped."
