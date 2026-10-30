#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
git clone -q --depth 1 https://github.com/mbezruchko/Snowhush "$TMP/upstream"
rm -rf "$ROOT/snow-core/src"
cp -r "$TMP/upstream/core/src" "$ROOT/snow-core/src"
cp "$TMP/upstream/LICENSE" "$ROOT/snow-core/LICENSE-UPSTREAM"
echo "Wersja oryginału: $(git -C "$TMP/upstream" log -1 --format='%h %cs %s')"
cd "$ROOT" && cargo test --release -p snow-core
