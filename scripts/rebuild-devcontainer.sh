#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_FILE="$WORKSPACE_ROOT/.devcontainer/docker-compose.yml"

NO_CACHE=false
for arg in "$@"; do
  case "$arg" in
    --no-cache) NO_CACHE=true ;;
    *) echo "Usage: $0 [--no-cache]" >&2; exit 1 ;;
  esac
done

echo "==> Tearing down devcontainer stack..."
docker compose -f "$COMPOSE_FILE" down --remove-orphans

echo "==> Pulling latest base images..."
docker pull navilalabs/zeitrak-devbase:latest
docker compose -f "$COMPOSE_FILE" pull --ignore-buildable

if $NO_CACHE; then
  echo "==> Rebuilding app image (no cache)..."
  docker compose -f "$COMPOSE_FILE" build --no-cache app
else
  echo "==> Rebuilding app image..."
  docker compose -f "$COMPOSE_FILE" build app
fi

echo "==> Starting devcontainer stack..."
docker compose -f "$COMPOSE_FILE" up -d

echo ""
echo "Done. Reopen the project in your editor to reconnect."
