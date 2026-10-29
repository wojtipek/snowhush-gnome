#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(python3 -c "import json;print(json.load(open('$ROOT/extension/snowhush-gnome@wojtipek/metadata.json'))['version-name'])")"
UUID="snowhush-gnome@wojtipek"
cd "$ROOT"

cargo build --release
cargo test --release --quiet
node --import ./tools/extension-harness/register.mjs tools/extension-harness/test.mjs
glib-compile-schemas --strict --dry-run "extension/$UUID/schemas"

DIST="$ROOT/dist"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
rm -rf "$DIST" && mkdir -p "$DIST"

PKG="$STAGE/snowhush-gnome"
mkdir -p "$PKG"
cp -r "extension/$UUID" "$PKG/"
mkdir -p "$PKG/$UUID/bin"
cp target/release/snowhush-engine "$PKG/$UUID/bin/"
cp LICENSE packaging/uninstall.sh "$PKG/$UUID/"
glib-compile-schemas "$PKG/$UUID/schemas"
cp packaging/install.sh packaging/uninstall.sh README.md README.pl.md LICENSE "$PKG/"

mkdir -p "$STAGE/snowhush-zrodla"
tar -C "$ROOT" --exclude=./target --exclude=./dist --exclude=./.git -cf - . | tar -C "$STAGE/snowhush-zrodla" -xf -

cd "$STAGE"
for part in gnome zrodla; do
    (cd "$STAGE" && zip -qr -X "$DIST/snowhush-$part-$VERSION.zip" "snowhush-$part")
done
ls -la "$DIST"
