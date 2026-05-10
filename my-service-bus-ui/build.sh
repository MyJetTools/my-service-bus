#!/usr/bin/env bash
# Build the Dioxus SPA in release mode (artifacts land in cargo `target/`)
# and then copy them into the server's wwwroot. The server
# (`my-service-bus`) serves that folder via StaticFilesMiddleware, so a
# successful run of this script is the only step needed to publish a UI
# change.
#
# Usage:  ./build.sh
# Override target dir:  WWWROOT=/path/to/wwwroot ./build.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WWWROOT="${WWWROOT:-$SCRIPT_DIR/../my-service-bus/wwwroot}"
DX_OUT="$SCRIPT_DIR/target/dx/my-service-bus-ui/release/web/public"

cd "$SCRIPT_DIR"
dx build --release --platform web

if [ ! -d "$DX_OUT" ]; then
    echo "ERROR: expected dx build output at $DX_OUT but it was not produced." >&2
    exit 1
fi

# Wipe previous bundle so files removed from the UI don't linger.
rm -rf "$WWWROOT"
mkdir -p "$WWWROOT/assets"

cp -R "$DX_OUT"/. "$WWWROOT/"

# Dioxus.toml `[web.resource].style` injects raw `<link href="/assets/x.css">`
# into index.html. dx hashes assets referenced via `asset!()` but does not
# emit raw copies. Mirror them ourselves so both the static `<link>` and the
# hashed runtime asset references resolve.
cp "$SCRIPT_DIR/assets/styled.css" "$WWWROOT/assets/styled.css"
cp "$SCRIPT_DIR/assets/app.css" "$WWWROOT/assets/app.css"

echo
echo "UI built  → $DX_OUT"
echo "Copied to → $WWWROOT"
