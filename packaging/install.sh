#!/usr/bin/env bash
set -euo pipefail

UUID="snowhush-gnome@wojtipek"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC="$HERE/$UUID"
DEST="$HOME/.local/share/gnome-shell/extensions/$UUID"

if [ ! -f "$SRC/metadata.json" ] || [ ! -f "$SRC/bin/snowhush-engine" ]; then
    echo "Brak plików rozszerzenia w $SRC – uruchom skrypt z rozpakowanej paczki. / Extension files missing in $SRC – run the script from the unzipped package." >&2
    exit 1
fi

OLD="snowhush-gnome@wojciech"
if [ -d "$HOME/.local/share/gnome-shell/extensions/$OLD" ]; then
    gnome-extensions disable "$OLD" 2>/dev/null || true
    pkill -x snowhush-engine 2>/dev/null || true
    rm -rf "$HOME/.local/share/gnome-shell/extensions/$OLD"
    python3 - "$OLD" <<'PY'
import ast, subprocess, sys
uuid = sys.argv[1]
for key in ("enabled-extensions", "disabled-extensions"):
    raw = subprocess.run(["gsettings", "get", "org.gnome.shell", key], capture_output=True, text=True).stdout.strip()
    items = ast.literal_eval(raw.removeprefix("@as ")) if raw else []
    if uuid in items:
        items.remove(uuid)
        subprocess.run(["gsettings", "set", "org.gnome.shell", key, str(items)], check=True)
PY
    echo "Usunięto poprzednią wersję ($OLD). / Removed the previous version ($OLD)."
fi

was_enabled=no
if gnome-extensions info "$UUID" >/dev/null 2>&1; then
    gnome-extensions disable "$UUID" 2>/dev/null || true
    was_enabled=yes
fi

rm -rf "$DEST"
mkdir -p "$(dirname "$DEST")"
cp -r "$SRC" "$DEST"
chmod +x "$DEST/bin/snowhush-engine"
glib-compile-schemas "$DEST/schemas"

if [ "$was_enabled" = yes ] && gnome-extensions enable "$UUID" 2>/dev/null; then
    echo "Zaktualizowano i włączono. Zmiany w samym rozszerzeniu zadziałają po wylogowaniu i ponownym zalogowaniu."
    echo "Updated and enabled. Changes to the extension itself take effect after you log out and back in."
else
    python3 - "$UUID" <<'PY'
import ast, subprocess, sys
uuid = sys.argv[1]
raw = subprocess.run(["gsettings", "get", "org.gnome.shell", "enabled-extensions"], capture_output=True, text=True, check=True).stdout.strip()
items = ast.literal_eval(raw.removeprefix("@as ")) if raw else []
if uuid not in items:
    items.append(uuid)
    subprocess.run(["gsettings", "set", "org.gnome.shell", "enabled-extensions", str(items)], check=True)
PY
    echo "Zainstalowano. Wyloguj się i zaloguj ponownie – śnieg włączy się sam (przełącznik „Śnieg” w Szybkich ustawieniach)."
    echo "Installed. Log out and back in – the snow starts by itself (the Snow tile in Quick Settings)."
fi
