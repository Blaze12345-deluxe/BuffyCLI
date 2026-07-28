VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Removes unused Docker resources (containers, images, volumes, networks)"

OUTPUT = false

WRITE "=== Docker Cleanup ==="
WRITE ""

WRITE "Pruning stopped containers..."
OUTPUT = true
RUN "docker container prune -f"
OUTPUT = false

WRITE ""
WRITE "Pruning unused images..."
OUTPUT = true
RUN "docker image prune -a -f"
OUTPUT = false

WRITE ""
WRITE "Pruning unused volumes..."
OUTPUT = true
RUN "docker volume prune -f"
OUTPUT = false

WRITE ""
WRITE "Pruning unused networks..."
OUTPUT = true
RUN "docker network prune -f"
OUTPUT = false

WRITE ""
WRITE "=== Docker Cleanup Complete ==="
