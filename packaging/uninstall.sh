#!/usr/bin/env bash
set -uo pipefail

UUID="snowhush-gnome@wojtipek"
DEST="$HOME/.local/share/gnome-shell/extensions/$UUID"

gnome-extensions disable "$UUID" 2>/dev/null || true
python3 - "$UUID" <<'PY'
import ast, subprocess, sys
uuid = sys.argv[1]
raw = subprocess.run(["gsettings", "get", "org.gnome.shell", "enabled-extensions"], capture_output=True, text=True).stdout.strip()
items = ast.literal_eval(raw.removeprefix("@as ")) if raw else []
if uuid in items:
    items.remove(uuid)
    subprocess.run(["gsettings", "set", "org.gnome.shell", "enabled-extensions", str(items)], check=True)
PY
pkill -x snowhush-engine 2>/dev/null || true
rm -rf "$DEST"
dconf reset -f /org/gnome/shell/extensions/snowhush-gnome/
echo "Usunięto rozszerzenie, silnik i ustawienia. / Removed the extension, the engine and the settings."
