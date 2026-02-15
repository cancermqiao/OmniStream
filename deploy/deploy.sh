#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

ENV_FILE="deploy/.env.prod"
STATE_DIR="deploy/.state"
STATE_FILE="$STATE_DIR/current_tag"
BACKUP_DIR="deploy/backups"
COMPOSE_FILE="docker-compose.prod.yml"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "missing $ENV_FILE (edit this file with your production values)"
  exit 1
fi

mkdir -p "$STATE_DIR" "$BACKUP_DIR" data
set -a
source "$ENV_FILE"
set +a

if [[ -z "${IMAGE_TAG:-}" ]]; then
  echo "IMAGE_TAG is required in $ENV_FILE"
  exit 1
fi

PREV_TAG=""
if [[ -f "$STATE_FILE" ]]; then
  PREV_TAG="$(cat "$STATE_FILE")"
fi

timestamp="$(date +%Y%m%d_%H%M%S)"
if [[ -f data/omnistream.db ]]; then
  cp data/omnistream.db "$BACKUP_DIR/omnistream.db.$timestamp.bak"
  echo "backup created: $BACKUP_DIR/omnistream.db.$timestamp.bak"
fi

export IMAGE_TAG REGISTRY API_PORT WEB_PORT RUST_LOG

echo "deploying tag: $IMAGE_TAG"
docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" pull || true
docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d --remove-orphans --no-build

HEALTH_URL="http://127.0.0.1:${API_PORT:-3000}/api/tasks"
ok=0
for i in {1..30}; do
  if curl -fsS "$HEALTH_URL" >/dev/null 2>&1; then
    ok=1
    break
  fi
  sleep 2
done

if [[ "$ok" -eq 1 ]]; then
  echo "$IMAGE_TAG" > "$STATE_FILE"
  echo "deploy success: $IMAGE_TAG"
  echo "web:  http://127.0.0.1:${WEB_PORT:-8080}"
  echo "api:  http://127.0.0.1:${API_PORT:-3000}"
  exit 0
fi

echo "health check failed for $IMAGE_TAG"
if [[ -n "$PREV_TAG" ]]; then
  echo "rolling back to $PREV_TAG"
  IMAGE_TAG="$PREV_TAG" docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d --remove-orphans --no-build
  exit 1
fi

echo "no previous tag found, rollback skipped"
exit 1
