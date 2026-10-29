#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
git clone -q --depth 1 https://github.com/mbezruchko/Snowhush "$TMP/upstream"
for f in simulation.rs layer.rs geometry.rs tests.rs; do
    cp "$TMP/upstream/windows/src/$f" "$ROOT/snow-core/src/$f"
done
cp "$TMP/upstream/LICENSE" "$ROOT/snow-core/LICENSE-UPSTREAM"
echo "Wersja oryginału: $(git -C "$TMP/upstream" log -1 --format='%h %cs %s')"
cd "$ROOT" && cargo test --release -p snow-core
