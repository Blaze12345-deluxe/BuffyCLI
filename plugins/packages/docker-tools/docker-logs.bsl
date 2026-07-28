VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Shows recent Docker Compose logs"

OUTPUT = false

WRITE "Recent Docker Compose logs:"
OUTPUT = true
RUN "docker compose logs --tail=50"
OUTPUT = false
