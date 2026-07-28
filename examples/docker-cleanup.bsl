VERSION = "2026.07.27"
AUTHOR = "Buffy Community"
DESCRIPTION = "Cleans up unused Docker resources."
OUTPUT = false

WRITE "========================================="
WRITE "  Docker Cleanup"
WRITE "========================================="
WRITE ""

WRITE "Step 1: Checking Docker is running..."
RUN "docker info > /dev/null 2>&1 && echo 'Docker is running' || echo 'Docker is not running'"

WRITE ""
WRITE "Step 2: Stopping all running containers..."

OUTPUT = true
RUN "docker stop $(docker ps -q) 2>/dev/null || echo 'No running containers'"

OUTPUT = false
WRITE ""
WRITE "Step 3: Removing unused containers..."

OUTPUT = true
RUN "docker container prune -f"

OUTPUT = false
WRITE ""
WRITE "Step 4: Removing unused images..."

OUTPUT = true
RUN "docker image prune -af"

OUTPUT = false
WRITE ""
WRITE "Step 5: Removing unused volumes..."

OUTPUT = true
RUN "docker volume prune -f"

OUTPUT = false
WRITE ""
WRITE "Step 6: Removing build cache..."

OUTPUT = true
RUN "docker builder prune -af"

OUTPUT = false
WRITE ""
WRITE "========================================="
WRITE "  Docker Cleanup Complete!"
WRITE "========================================="

EXIT
