#!/usr/bin/env bash
set -euo pipefail

: "${SONAR_HOST_URL:?Set SONAR_HOST_URL to the local SonarQube server URL.}"
: "${SONAR_TOKEN:?Set SONAR_TOKEN in the environment.}"
root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
scanner_image="${SONAR_SCANNER_IMAGE:-sonarsource/sonar-scanner-cli:latest}"

for service in api worker frontend; do
  docker run --rm --network host \
    --volume "$root_dir:/usr/src" \
    --env SONAR_HOST_URL \
    --env SONAR_TOKEN \
    --workdir /usr/src \
    "$scanner_image" \
    "-Dproject.settings=/usr/src/.sonar/${service}.properties" \
    "-Dsonar.working.directory=/usr/src/.sonar/working/${service}"
done
