#!/usr/bin/env bash
# Builds the Dioxus UI (in ui/) and copies it into wwwroot/, which the node
# serves via StaticFilesMiddleware and the Dockerfile bakes into the image.
# Run this after changing anything under ui/, then commit the updated wwwroot/.
# Just a convenience wrapper so you don't have to cd into ui/ first.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

exec "${SCRIPT_DIR}/ui/build.sh"
