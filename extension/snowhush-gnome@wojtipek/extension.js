import St from 'gi://St';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as WorkspaceAnimation from 'resource:///org/gnome/shell/ui/workspaceAnimation.js';
import {Extension, InjectionManager} from 'resource:///org/gnome/shell/extensions/extension.js';

import {Engine} from './engine.js';
import {Scout} from './scout.js';
import {SnowIndicator} from './indicator.js';

export default class SnowhushExtension extends Extension {
    enable() {
        this._settings = this.getSettings();
        this._engine = new Engine(`${this.path}/bin/snowhush-engine`, () => this._onEngineStarted());
        this._scout = new Scout(this._engine);
        this._watchWorkspaceAnimation();
        this._indicator = new SnowIndicator(this._settings, () => this._clearSnow(), () => this.openPreferences(), this.path);
        Main.panel.statusArea.quickSettings.addExternalIndicator(this._indicator);

        this._settings.connectObject('changed', (_settings, key) => this._settingChanged(key), this);
        this._engine.setBehindWindows(this._settings.get_boolean('behind-windows'));
        this._scout.setSnowOnWindows(!this._settings.get_boolean('behind-windows'));
        if (this._settings.get_boolean('enabled'))
            this._startSnow();
    }

    disable() {
        this._settings?.disconnectObject(this);
        this._injections?.clear();
        this._injections = null;
        this._stopSnow();
        this._engine?.destroy();
        this._engine = null;
        this._scout = null;
        this._indicator?.destroy();
        this._indicator = null;
        this._settings = null;
    }

    _watchWorkspaceAnimation() {
        const controller = WorkspaceAnimation.WorkspaceAnimationController?.prototype;
        if (!controller?._prepareWorkspaceSwitch || !controller?._finishWorkspaceSwitch)
            return;
        this._injections = new InjectionManager();
        const scout = () => this._scout;
        this._injections.overrideMethod(controller, '_prepareWorkspaceSwitch', original => function (...args) {
            const result = original.apply(this, args);
            scout()?.setSwitching(true);
            return result;
        });
        this._injections.overrideMethod(controller, '_finishWorkspaceSwitch', original => function (...args) {
            const result = original.apply(this, args);
            scout()?.setSwitching(false);
            return result;
        });
    }

    _clearSnow() {
        if (!this._running)
            return;
        const calm = !St.Settings.get().enable_animations;
        this._engine.send(`clear ${calm ? 1 : 0}`);
    }

    _overWindows() {
        return this._settings.get_boolean('over-windows') && !this._settings.get_boolean('behind-windows');
    }

    _snowOnWindows() {
        return this._settings.get_boolean('snow-on-windows') && !this._settings.get_boolean('behind-windows');
    }

    _startSnow() {
        if (this._running)
            return;
        this._running = true;
        this._engine.start();
        this._syncScout();
    }

    _syncScout() {
        const want = !!this._running;
        if (want === !!this._scoutRunning)
            return;
        this._scoutRunning = want;
        if (want)
            this._scout.start();
        else
            this._scout.stop();
    }

    _stopSnow() {
        if (!this._running)
            return;
        this._running = false;
        this._syncScout();
        this._engine.stop();
    }

    _onEngineStarted() {
        const s = this._settings;
        this._engine.send(`set snowfall ${s.get_string('snowfall')}`);
        this._engine.send(`set accumulate ${s.get_boolean('accumulate') ? 1 : 0}`);
        this._engine.send(`set melt ${s.get_boolean('melt') ? 1 : 0}`);
        this._engine.send(`set snow-on-windows ${this._snowOnWindows() ? 1 : 0}`);
        this._engine.send(`set over-windows ${this._overWindows() ? 1 : 0}`);
        for (const key of ['idle-slowdown', 'rest-when-hidden', 'battery-slowdown'])
            this._engine.send(`set ${key} ${s.get_boolean(key) ? 1 : 0}`);
        this._engine.send(`set sharp ${s.get_boolean('sharp') ? 1 : 0}`);
        this._engine.send(`set calm-fps ${s.get_int('calm-fps')}`);
        this._engine.send(`set halloween ${s.get_boolean('halloween') ? 1 : 0}`);
        this._engine.send(`set autumn ${s.get_boolean('autumn') ? 1 : 0}`);
        if (this._running && this._scoutRunning)
            this._scout.sendAll();
    }

    _settingChanged(key) {
        const s = this._settings;
        switch (key) {
        case 'enabled':
            if (s.get_boolean('enabled'))
                this._startSnow();
            else
                this._stopSnow();
            break;
        case 'snowfall':
            this._engine.send(`set snowfall ${s.get_string('snowfall')}`);
            break;
        case 'autumn': {
            const calm = !St.Settings.get().enable_animations;
            this._engine.send(`mode ${s.get_boolean('autumn') ? 'autumn' : 'snow'} ${calm ? 1 : 0}`);
            break;
        }
        case 'calm-fps':
            this._engine.send(`set calm-fps ${s.get_int('calm-fps')}`);
            break;
        case 'accumulate':
        case 'melt':
        case 'sharp':
        case 'halloween':
        case 'idle-slowdown':
        case 'rest-when-hidden':
        case 'battery-slowdown':
            this._engine.send(`set ${key} ${s.get_boolean(key) ? 1 : 0}`);
            break;
        case 'over-windows':
        case 'snow-on-windows':
        case 'behind-windows':
            this._engine.send(`set over-windows ${this._overWindows() ? 1 : 0}`);
            this._engine.setBehindWindows(s.get_boolean('behind-windows'));
            this._engine.send(`set snow-on-windows ${this._snowOnWindows() ? 1 : 0}`);
            this._scout.setSnowOnWindows(!s.get_boolean('behind-windows'));
            break;
        }
    }
}
