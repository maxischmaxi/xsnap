#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "$0")"

cleanup() {
  docker compose down --remove-orphans --timeout 5 2>/dev/null
}
trap cleanup EXIT

docker compose build storybook
docker compose run --rm xsnap
exit $?
