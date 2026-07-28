VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Starts Docker Compose services in detached mode"

OUTPUT = false

WRITE "Starting Docker Compose services..."
OUTPUT = true
RUN "docker compose up -d"
OUTPUT = false

WRITE ""
WRITE "Services are running."
