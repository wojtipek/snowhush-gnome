# Snowhush for GNOME

**English** · [Polski](README.pl.md) · [Русский](README.ru.md)

Soft snow on your GNOME desktop. It settles on the top edges of windows and piles up at the bottom of the screen. The cursor pushes drifts aside, dragged or shaken windows throw their snow off, and closed windows leave lumps that burst into powder.

A port of [Snowhush](https://github.com/mbezruchko/Snowhush) by Mykhailo Bezruchko (macOS and Windows) to GNOME on Wayland. The snow physics is his code, copied unchanged.

> Built with the help of AI. Tested on Ubuntu 26.04 with GNOME 50.

## Requirements

- GNOME Shell 46–51 on **Wayland**
- A GPU with OpenGL ES 3 (any Intel, AMD or NVIDIA driver from the last ten years)

Other Wayland desktops have their own ports: [KDE Plasma](https://github.com/TryCoffee/Snowhush/tree/kde-port) and [Hyprland](https://github.com/Nevers1/snow-hyprland).

## Install

Paste this into a terminal. It downloads the latest release, installs it and cleans up after itself:

```bash
T=$(mktemp -d) && curl -fsSL -o "$T/s.zip" https://github.com/wojtipek/snowhush-gnome/releases/latest/download/snowhush-gnome.zip && unzip -q "$T/s.zip" -d "$T" && bash "$T/snowhush-gnome/install.sh"; rm -rf "$T"
```

Then log out and back in. On Wayland, GNOME only loads new extensions at login.

The same command updates an existing installation to the latest version. You can also download `snowhush-gnome.zip` from [Releases](https://github.com/wojtipek/snowhush-gnome/releases) yourself, unzip it and run `install.sh`.

Everything goes into one folder, `~/.local/share/gnome-shell/extensions/snowhush-gnome@wojtipek`.

## Use

Open **Quick Settings** (top-right corner, where Wi-Fi and Bluetooth are) and find the **Snow** tile. Clicking the tile turns the snow on and off, and the arrow next to it opens the menu:

- **Snowfall**: no snowfall, light, moderate, heavy or variable
- **Blow the snow away**: a gust clears all the snow, including snow on windows on other workspaces
- **Options**: opens the settings window

The settings window (also under the extension in the *Extensions* app) has:

- **Accumulation**, **Melting**
- **Snow on windows**, **Flakes over windows** (turn off to keep the flakes behind windows while the snow on their edges stays)
- **Behind windows**: snow only on the wallpaper
- **Sharper image**: draws at the full screen resolution instead of the GNOME scale (uses more GPU)
- **Halloween**: from 30 October until dawn on 1 November, ghosts made of snow rise out of the drifts, jack-o'-lanterns glow in the snow and the cursor carries a candle (on by default)
- **Language**: automatic (follows the system) or any of the 14 languages below

The snow stops for fullscreen apps, the overview and the lock screen, and gets lighter in power-saver mode.

## Uninstall

```bash
bash ~/.local/share/gnome-shell/extensions/snowhush-gnome@wojtipek/uninstall.sh
```

This removes the extension, the engine and the settings. Nothing is left behind.

## Languages

The menu follows your system language: Polish (the base language), English (British), Spanish (Mexican), Portuguese (Brazilian), Catalan, German, Czech, Slovak, Ukrainian, Lithuanian, Hungarian, Croatian, Esperanto and Latin. Any other language gets Latin. You can also pick the language yourself under **Options → Language**.

Most of the translations were not written by native speakers. Corrections and new languages are welcome: they all live in one table in [`i18n.js`](extension/snowhush-gnome@wojtipek/i18n.js).

## How it works

Wayland doesn't let ordinary apps see other windows or the cursor, so the port has two parts:

```
GNOME Shell extension                           separate process
  scout.js      windows, cursor, monitors  ──►   snowhush-engine
  engine.js     starts the engine,                 snow-core   physics from Snowhush
                places its overlay windows         render.rs   OpenGL ES 3
  indicator.js  the Snow tile and menu             wl.rs       Wayland client
```

- **The engine** (Rust, no external crates, with its own small PNG decoder) draws the snow into one transparent, click-through window per monitor. It loads the system's libwayland, EGL and OpenGL ES libraries at run time.
- **The extension** starts the engine as a trusted Wayland client (`Meta.WaylandClient`), so it can recognise the engine's windows. It keeps them above normal windows, out of the window list and off the focus chain, and places them over each monitor's work area.
- They talk over the engine's standard input, one line of text per command (see [`protocol.rs`](snow-engine/src/protocol.rs)).

`snow-core/src` is Snowhush's shared physics (`core/src`), copied unchanged. The Halloween pictures in `snow-engine/assets` and the shaders in `snow-engine/src/shaders` come from Snowhush too; the shaders are adapted to hide the snow behind windows. `scripts/sync-upstream.sh` pulls the latest version and runs its tests.

## Build from source

You need Rust (cargo), Node.js, `zip` and `glib-compile-schemas`.

```bash
./scripts/build.sh
```

This builds the engine, runs all the tests (the original physics tests, the protocol and the extension logic against mocks of GNOME) and puts the packages in `dist/`.

## Licence

MIT, see [LICENSE](LICENSE). Snowhush © Mykhailo Bezruchko.
