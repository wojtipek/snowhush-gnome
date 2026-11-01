import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';

const TITLE_PATTERN = /^snowhush-overlay-(\d+)$/;
const APP_ID = 'org.snowhush.Overlay';
const LATEST_ONLY = new Set(['monitors', 'windows', 'elsewhere', 'cursor', 'covered', 'power']);
const MAX_RESTARTS = 5;
const MAX_CORRECTIONS = 5;
const CORRECTION_WINDOW_US = 2 * 1000 * 1000;

function log(message) {
    console.log(`snowhush-gnome: ${message}`);
}

export class Engine {
    constructor(binaryPath, onStarted) {
        this._binaryPath = binaryPath;
        this._onStarted = onStarted;
        this._queue = [];
        this._writing = false;
        this._overlays = new Map();
        this._windowSignals = new Map();
        this._behindWindows = false;
        this._restarts = [];
        this._stopping = false;
        this._encoder = new TextEncoder();

        global.display.connectObject(
            'window-created', (_display, window) => this._windowCreated(window),
            'workareas-changed', () => this._positionAll(),
            this);
        Main.layoutManager.connectObject('monitors-changed', () => this._positionAll(), this);
    }

    start() {
        this._stopping = false;
        this._spawn();
    }

    stop() {
        this._stopping = true;
        this._kill();
    }

    destroy() {
        this.stop();
        global.display.disconnectObject(this);
        Main.layoutManager.disconnectObject(this);
        for (const window of [...this._windowSignals.keys()])
            this._forgetWindow(window);
    }

    get running() {
        return this._process !== null && this._process !== undefined;
    }

    isOverlay(window) {
        return this._overlays.has(window) || this._looksLikeOverlay(window);
    }

    setBehindWindows(behind) {
        if (behind === this._behindWindows)
            return;
        this._behindWindows = behind;
        for (const window of this._overlays.keys())
            this._applyType(window);
    }

    send(line) {
        if (!this._stdin)
            return;
        const kind = line.split(' ', 1)[0];
        if (LATEST_ONLY.has(kind))
            this._queue = this._queue.filter(entry => entry.kind !== kind);
        this._queue.push({kind, line});
        this._flush();
    }

    _spawn() {
        if (this._process)
            return;
        const argv = [this._binaryPath];
        const launcher = new Gio.SubprocessLauncher({
            flags: Gio.SubprocessFlags.STDIN_PIPE | Gio.SubprocessFlags.STDOUT_PIPE,
        });
        launcher.setenv('RUST_BACKTRACE', '1', true);
        try {
            if (Meta.WaylandClient?.new_subprocess) {
                this._client = Meta.WaylandClient.new_subprocess(global.context, launcher, argv);
                this._process = this._client.get_subprocess();
            } else if (Meta.WaylandClient?.new) {
                this._client = Meta.WaylandClient.new(global.context, launcher);
                this._process = this._client.spawnv(global.display, argv);
            } else {
                this._client = null;
                this._process = launcher.spawnv(argv);
            }
        } catch (error) {
            log(`nie można uruchomić silnika (${this._binaryPath}): ${error.message}`);
            this._process = null;
            this._client = null;
            return;
        }
        log(`silnik uruchomiony (pid ${this._process.get_identifier()})`);
        this._stdin = this._process.get_stdin_pipe();
        this._queue = [];
        this._writing = false;
        this._readOutput(this._process);
        const process = this._process;
        process.wait_async(null, (_proc, result) => {
            try {
                process.wait_finish(result);
            } catch {}
            if (process !== this._process)
                return;
            const status = process.get_if_exited() ? `kod ${process.get_exit_status()}` : 'przerwany sygnałem';
            log(`silnik zakończył pracę (${status})`);
            this._process = null;
            this._client = null;
            this._stdin = null;
            this._overlays.clear();
            if (!this._stopping)
                this._scheduleRestart();
        });
        for (const actor of global.get_window_actors())
            this._windowCreated(actor.meta_window);
        this._onStarted();
    }

    _readOutput(process) {
        const stream = new Gio.DataInputStream({base_stream: process.get_stdout_pipe(), close_base_stream: true});
        const readLine = () => {
            stream.read_line_async(GLib.PRIORITY_LOW, null, (_stream, result) => {
                let line = null;
                try {
                    [line] = stream.read_line_finish_utf8(result);
                } catch {
                    return;
                }
                if (line === null)
                    return;
                if (line === 'ready')
                    log('silnik gotowy');
                readLine();
            });
        };
        readLine();
    }

    _scheduleRestart() {
        const now = GLib.get_monotonic_time();
        this._restarts = this._restarts.filter(time => now - time < 60 * 1e6);
        if (this._restarts.length >= MAX_RESTARTS) {
            log('silnik pada zbyt często – nie uruchamiam go ponownie (szczegóły: journalctl --user -b | grep snowhush)');
            return;
        }
        this._restarts.push(now);
        this._restartSource = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 2, () => {
            this._restartSource = 0;
            if (!this._stopping)
                this._spawn();
            return GLib.SOURCE_REMOVE;
        });
    }

    _kill() {
        if (this._restartSource) {
            GLib.source_remove(this._restartSource);
            this._restartSource = 0;
        }
        const process = this._process;
        if (!process)
            return;
        this._process = null;
        this._client = null;
        this._overlays.clear();
        const stdin = this._stdin;
        this._stdin = null;
        this._queue = [];
        stdin?.close_async(GLib.PRIORITY_DEFAULT, null, (_stream, result) => {
            try {
                stdin.close_finish(result);
            } catch {}
        });
        GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 2, () => {
            try {
                process.force_exit();
            } catch {}
            return GLib.SOURCE_REMOVE;
        });
    }

    _flush() {
        if (this._writing || !this._stdin || this._queue.length === 0)
            return;
        const text = `${this._queue.map(entry => entry.line).join('\n')}\n`;
        this._queue = [];
        this._writing = true;
        this._write(this._stdin, this._encoder.encode(text));
    }

    _write(stdin, data) {
        stdin.write_bytes_async(new GLib.Bytes(data), GLib.PRIORITY_DEFAULT, null, (_stream, result) => {
            let written = 0;
            try {
                written = stdin.write_bytes_finish(result);
            } catch (error) {
                if (stdin === this._stdin)
                    log(`błąd zapisu do silnika: ${error.message}`);
                this._writing = false;
                return;
            }
            if (stdin !== this._stdin) {
                this._writing = false;
                return;
            }
            if (written < data.length) {
                this._write(stdin, data.subarray(written));
                return;
            }
            this._writing = false;
            this._flush();
        });
    }

    _looksLikeOverlay(window) {
        if (!this._process || !window)
            return false;
        try {
            if (this._client?.owns_window)
                return this._client.owns_window(window);
        } catch {}
        const pid = Number(this._process.get_identifier());
        return window.get_pid() === pid && window.get_wm_class() === APP_ID;
    }

    _windowCreated(window) {
        if (!window || this._overlays.has(window) || !this._looksLikeOverlay(window))
            return;
        this._overlays.set(window, -1);
        this._applyType(window);
        try {
            window.stick();
        } catch {}
        const signals = [
            window.connect('notify::title', () => this._readIndex(window)),
            window.connect('shown', () => this._position(window)),
            window.connect('size-changed', () => this._correct(window)),
            window.connect('position-changed', () => this._correct(window)),
            window.connect('unmanaged', () => this._forgetWindow(window)),
        ];
        this._windowSignals.set(window, signals);
        this._readIndex(window);
    }

    _forgetWindow(window) {
        const signals = this._windowSignals.get(window) ?? [];
        for (const id of signals) {
            try {
                window.disconnect(id);
            } catch {}
        }
        this._windowSignals.delete(window);
        this._overlays.delete(window);
        this._corrections?.delete(window);
    }

    _readIndex(window) {
        const match = TITLE_PATTERN.exec(window.get_title() ?? '');
        if (!match)
            return;
        this._overlays.set(window, Number(match[1]));
        this._position(window);
        log(this.describeOverlays());
    }

    _applyType(window) {
        const type = this._behindWindows ? Meta.WindowType.DESKTOP : Meta.WindowType.DOCK;
        try {
            if (window.set_type) {
                window.set_type(type);
                window.hide_from_window_list?.();
            } else if (this._client) {
                if (this._behindWindows)
                    this._client.make_desktop?.(window);
                else
                    this._client.make_dock?.(window);
                this._client.hide_from_window_list?.(window);
            } else if (!this._behindWindows) {
                window.make_above();
            }
        } catch (error) {
            log(`nie można ustawić typu okna nakładki: ${error.message}`);
        }
    }

    describeOverlays() {
        const parts = [];
        for (const [window, index] of this._overlays) {
            const r = window.get_frame_rect();
            parts.push(`nakładka ${index}: typ ${window.get_window_type()} warstwa ${window.get_layer?.() ?? '?'} ` +
                `${r.x},${r.y} ${r.width}×${r.height}`);
        }
        return parts.join('; ') || 'brak nakładek';
    }

    _position(window) {
        const index = this._overlays.get(window);
        if (index === undefined || index < 0 || index >= Main.layoutManager.monitors.length)
            return false;
        const area = Main.layoutManager.getWorkAreaForMonitor(index);
        const frame = window.get_frame_rect();
        if (frame.x === area.x && frame.y === area.y && frame.width === area.width && frame.height === area.height)
            return false;
        window.move_resize_frame(false, area.x, area.y, area.width, area.height);
        return true;
    }

    _correct(window) {
        const now = GLib.get_monotonic_time();
        const corrections = this._corrections ??= new Map();
        let tries = corrections.get(window);
        if (!tries || now - tries.since >= CORRECTION_WINDOW_US)
            tries = {since: now, count: 0};
        if (tries.count >= MAX_CORRECTIONS)
            return;
        if (this._position(window)) {
            tries.count++;
            corrections.set(window, tries);
        }
    }

    _positionAll() {
        for (const window of this._overlays.keys())
            this._position(window);
    }
}
