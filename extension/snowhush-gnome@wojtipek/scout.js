import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PointerWatcher from 'resource:///org/gnome/shell/ui/pointerWatcher.js';

const FLAG_MAXIMIZED = 1;
const FLAG_FULLSCREEN = 2;
const FLAG_COVER_ONLY = 4;
const FLAG_ROUNDED = 8;

const SOLID_TYPES = new Set([
    Meta.WindowType.NORMAL,
    Meta.WindowType.DIALOG,
    Meta.WindowType.MODAL_DIALOG,
    Meta.WindowType.UTILITY,
    Meta.WindowType.TOOLBAR,
    Meta.WindowType.SPLASHSCREEN,
]);
const COVER_TYPES = new Set([
    Meta.WindowType.MENU,
    Meta.WindowType.DROPDOWN_MENU,
    Meta.WindowType.POPUP_MENU,
    Meta.WindowType.TOOLTIP,
    Meta.WindowType.NOTIFICATION,
    Meta.WindowType.COMBO,
]);

const POINTER_INTERVAL_MS = 16;

const SLIDE_SPEED = 6000;
const IDLE_MS = 5 * 60 * 1000;

export class Scout {
    constructor(engine) {
        this._engine = engine;
        this._windowSignals = new Map();
        this._laterId = 0;
        this._moving = false;
        this._snowOnWindows = true;
        this._lastCursor = '';
        this._lastWindows = '';
        this._lastElsewhere = '';
        this._lastCovered = '';
        this._lastMonitors = '';
        this._powerSaver = false;
        this._switching = false;
    }

    setSwitching(switching) {
        if (switching === this._switching)
            return;
        this._switching = switching;
        this._sendWindows(true);
    }

    start() {
        const display = global.display;
        display.connectObject(
            'window-created', (_d, window) => {
                this._watchWindow(window);
                this._schedule();
            },
            'restacked', () => this._schedule(),
            'grab-op-begin', () => {
                this._moving = true;
                this._sendWindows(true);
            },
            'grab-op-end', () => {
                this._moving = false;
                this._sendWindows(true);
            },
            'in-fullscreen-changed', () => this._sendCovered(),
            'workareas-changed', () => this._sendMonitors(),
            this);
        this._workspace = global.workspace_manager.get_active_workspace_index();
        global.workspace_manager.connectObject('active-workspace-changed', () => {
            this._engine.send('workspace');
            const index = global.workspace_manager.get_active_workspace_index();
            if (index !== this._workspace)
                this._engine.send(`slide ${index > this._workspace ? -SLIDE_SPEED : SLIDE_SPEED}`);
            this._workspace = index;
            this._sendWindows(true);
        }, this);
        Main.layoutManager.connectObject('monitors-changed', () => this._sendMonitors(), this);
        Main.overview.connectObject(
            'showing', () => this._sendCovered(),
            'hidden', () => this._sendCovered(),
            this);
        Main.sessionMode.connectObject('updated', () => this._sendCovered(), this);

        for (const actor of global.get_window_actors())
            this._watchWindow(actor.meta_window);

        this._pointerWatch = PointerWatcher.getPointerWatcher().addWatch(POINTER_INTERVAL_MS, (x, y) => this._sendCursor(x, y));
        this._watchPowerProfile();
        this._watchIdle();
        this.sendAll();
    }

    _watchIdle() {
        const monitor = global.backend?.get_core_idle_monitor?.();
        if (!monitor)
            return;
        this._idleMonitor = monitor;
        this._idle = monitor.get_idletime() >= IDLE_MS;
        this._idleWatch = monitor.add_idle_watch(IDLE_MS, () => {
            this._setIdle(true);
            this._activeWatch = monitor.add_user_active_watch(() => {
                this._activeWatch = 0;
                this._setIdle(false);
            });
        });
    }

    _unwatchIdle() {
        for (const id of [this._idleWatch, this._activeWatch]) {
            if (id)
                this._idleMonitor?.remove_watch(id);
        }
        this._idleWatch = this._activeWatch = 0;
        this._idleMonitor = null;
    }

    _setIdle(idle) {
        this._idle = idle;
        this._engine.send(`idle ${idle ? 1 : 0}`);
    }

    stop() {
        this._switching = false;
        global.display.disconnectObject(this);
        global.workspace_manager.disconnectObject(this);
        Main.layoutManager.disconnectObject(this);
        Main.overview.disconnectObject(this);
        Main.sessionMode.disconnectObject(this);
        this._pointerWatch?.remove();
        this._pointerWatch = null;
        this._unwatchIdle();
        if (this._laterId) {
            global.compositor.get_laters().remove(this._laterId);
            this._laterId = 0;
        }
        for (const window of [...this._windowSignals.keys()])
            this._unwatchWindow(window);
        this._powerProxy?.disconnectObject(this);
        this._powerProxy = null;
    }

    sendAll() {
        this._lastCursor = this._lastWindows = this._lastElsewhere = this._lastCovered = this._lastMonitors = '';
        this._sendMonitors();
        this._sendCovered();
        this._sendPower();
        this._engine.send(`idle ${this._idle ? 1 : 0}`);
        this._sendWindows(true);
        const [x, y] = global.get_pointer();
        this._sendCursor(x, y);
    }

    setSnowOnWindows(on) {
        this._snowOnWindows = on;
        this._lastWindows = '';
        this._sendWindows(true);
    }

    _watchWindow(window) {
        if (!window || this._windowSignals.has(window))
            return;
        const schedule = () => this._schedule();
        const ids = [
            window.connect('position-changed', schedule),
            window.connect('size-changed', schedule),
            window.connect('workspace-changed', schedule),
            window.connect('notify::minimized', schedule),
            window.connect('notify::fullscreen', schedule),
            window.connect('unmanaged', () => {
                this._unwatchWindow(window);
                this._schedule();
            }),
        ];
        this._windowSignals.set(window, ids);
    }

    _unwatchWindow(window) {
        for (const id of this._windowSignals.get(window) ?? []) {
            try {
                window.disconnect(id);
            } catch {}
        }
        this._windowSignals.delete(window);
    }

    _schedule() {
        if (this._laterId)
            return;
        this._laterId = global.compositor.get_laters().add(Meta.LaterType.BEFORE_REDRAW, () => {
            this._laterId = 0;
            this._sendWindows(false);
            return GLib.SOURCE_REMOVE;
        });
    }

    _sendWindows(force) {
        const workspace = global.workspace_manager.get_active_workspace();
        const entries = [];
        const elsewhere = [];
        if (this._snowOnWindows) {
            const actors = global.get_window_actors().slice().reverse();
            for (const actor of actors) {
                const window = actor.meta_window;
                if (!window || this._engine.isOverlay(window))
                    continue;
                const type = window.get_window_type();
                const solid = SOLID_TYPES.has(type);
                const coverOnly = COVER_TYPES.has(type) || window.is_override_redirect();
                if (!solid && !coverOnly)
                    continue;
                const onWorkspace = window.is_on_all_workspaces() || window.located_on_workspace(workspace);
                if (!onWorkspace || this._switching) {
                    if (solid && !window.minimized)
                        elsewhere.push(window.get_id());
                    continue;
                }
                if (window.minimized || !actor.visible || !window.showing_on_its_workspace())
                    continue;
                const rect = window.get_frame_rect();
                if (rect.width < 2 || rect.height < 2)
                    continue;
                let flags = 0;
                if (isMaximized(window))
                    flags |= FLAG_MAXIMIZED;
                if (window.is_fullscreen())
                    flags |= FLAG_FULLSCREEN;
                if (coverOnly)
                    flags |= FLAG_COVER_ONLY;
                if (solid && !isTiled(window))
                    flags |= FLAG_ROUNDED;
                entries.push(`${window.get_id()}:${rect.x},${rect.y},${rect.width},${rect.height},${flags}`);
            }
        }
        const windows = `windows ${this._moving ? 1 : 0}${entries.length ? ' ' : ''}${entries.join(' ')}`;
        const elsewhereLine = `elsewhere${elsewhere.length ? ' ' : ''}${elsewhere.join(' ')}`;
        if (elsewhereLine !== this._lastElsewhere) {
            this._lastElsewhere = elsewhereLine;
            this._engine.send(elsewhereLine);
        }
        if (force || windows !== this._lastWindows) {
            this._lastWindows = windows;
            this._engine.send(windows);
        }
    }

    _sendCursor(x, y) {
        const line = `cursor ${Math.round(x)} ${Math.round(y)}`;
        if (line === this._lastCursor)
            return;
        this._lastCursor = line;
        this._engine.send(line);
    }

    _sendMonitors() {
        const entries = Main.layoutManager.monitors.map((monitor, index) => {
            const area = Main.layoutManager.getWorkAreaForMonitor(index);
            const scale = global.display.get_monitor_scale(index);
            return `${index}:${area.x},${area.y},${area.width},${area.height},${scale.toFixed(3)},` +
                `${monitor.x},${monitor.y},${monitor.width},${monitor.height}`;
        });
        const line = `monitors ${entries.join(' ')}`;
        if (line === this._lastMonitors)
            return;
        this._lastMonitors = line;
        this._engine.send(line);
    }

    _sendCovered() {
        const all = Main.overview.visible || Main.sessionMode.isLocked;
        const fullscreen = [];
        const count = global.display.get_n_monitors();
        for (let i = 0; i < count; i++) {
            if (global.display.get_monitor_in_fullscreen(i))
                fullscreen.push(i);
        }
        const line = `covered ${all ? 1 : 0}${fullscreen.length ? ' ' : ''}${fullscreen.join(' ')}`;
        if (line === this._lastCovered)
            return;
        this._lastCovered = line;
        this._engine.send(line);
    }

    _watchPowerProfile() {
        const names = [
            ['org.freedesktop.UPower.PowerProfiles', '/org/freedesktop/UPower/PowerProfiles'],
            ['net.hadess.PowerProfiles', '/net/hadess/PowerProfiles'],
        ];
        const tryNext = index => {
            if (index >= names.length)
                return;
            const [name, path] = names[index];
            Gio.DBusProxy.new_for_bus(Gio.BusType.SYSTEM, Gio.DBusProxyFlags.DO_NOT_AUTO_START, null,
                name, path, name, null, (_source, result) => {
                    let proxy;
                    try {
                        proxy = Gio.DBusProxy.new_for_bus_finish(result);
                    } catch {
                        tryNext(index + 1);
                        return;
                    }
                    if (!proxy.get_name_owner()) {
                        tryNext(index + 1);
                        return;
                    }
                    this._powerProxy = proxy;
                    proxy.connectObject('g-properties-changed', () => this._readPowerProfile(), this);
                    this._readPowerProfile();
                });
        };
        tryNext(0);
    }

    _readPowerProfile() {
        const profile = this._powerProxy?.get_cached_property('ActiveProfile')?.unpack();
        const saver = profile === 'power-saver';
        if (saver === this._powerSaver)
            return;
        this._powerSaver = saver;
        this._sendPower();
    }

    _sendPower() {
        this._engine.send(`power ${this._powerSaver ? 'saver' : 'normal'}`);
    }
}

function isMaximized(window) {
    if (window.is_maximized)
        return window.is_maximized();
    return window.get_maximized?.() === Meta.MaximizeFlags.BOTH;
}

function isTiled(window) {
    if (window.is_fullscreen())
        return true;
    const flags = window.get_maximize_flags?.() ?? window.get_maximized?.() ?? 0;
    return flags !== 0;
}
