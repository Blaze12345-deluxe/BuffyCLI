VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Pulls latest images and recreates containers"

OUTPUT = false

WRITE "Pulling latest images..."
OUTPUT = true
RUN "docker compose pull"
OUTPUT = false

WRITE ""
WRITE "Recreating containers..."
OUTPUT = true
RUN "docker compose up -d --remove-orphans"
OUTPUT = false

WRITE ""
WRITE "Containers updated and running."
