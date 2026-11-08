import GObject from 'gi://GObject';
import Gio from 'gi://Gio';

import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {Slider} from 'resource:///org/gnome/shell/ui/slider.js';
import {QuickMenuToggle, SystemIndicator} from 'resource:///org/gnome/shell/ui/quickSettings.js';

import {_, setLanguage} from './i18n.js';

const ICON = 'weather-snow-symbolic';

const MODES = [
    ['snow', 'Śnieg', 'Śnieg', 'Zdmuchnij śnieg'],
    ['autumn', 'Jesienne liście', 'Liście', 'Zdmuchnij liście'],
    ['rain', 'Deszcz', 'Deszcz', null],
    ['blossom', 'Kwiaty wiśni', 'Płatki', 'Zdmuchnij płatki'],
];

export function activeModes(settings) {
    return settings.get_boolean('mixed') ? settings.get_strv('mix') : [settings.get_string('mode')];
}

export function pickMode(settings, mode) {
    if (settings.get_boolean('mixed')) {
        const mix = settings.get_strv('mix');
        if (mix.includes(mode))
            return;
        settings.set_strv('mix', [mix[1], mode]);
    }
    settings.set_string('mode', mode);
}

export function toggleMixed(settings) {
    const mixed = !settings.get_boolean('mixed');
    if (mixed) {
        const mode = settings.get_string('mode');
        const mix = settings.get_strv('mix');
        const other = mix.find(m => m !== mode) ?? (mode === 'snow' ? 'autumn' : 'snow');
        settings.set_strv('mix', [other, mode]);
    }
    settings.set_boolean('mixed', mixed);
}

const SnowToggle = GObject.registerClass(
class SnowToggle extends QuickMenuToggle {
    _init(settings, clearSnow, openOptions, path) {
        super._init({
            iconName: ICON,
            toggleMode: true,
        });
        this._settings = settings;
        this._labels = [];
        this._icons = {
            snow: new Gio.ThemedIcon({name: ICON}),
            autumn: new Gio.FileIcon({file: Gio.File.new_for_path(`${path}/icons/leaf-symbolic.svg`)}),
            rain: new Gio.ThemedIcon({name: 'weather-showers-symbolic'}),
            blossom: new Gio.FileIcon({file: Gio.File.new_for_path(`${path}/icons/blossom-symbolic.svg`)}),
        };
        settings.bind('enabled', this, 'checked', Gio.SettingsBindFlags.DEFAULT);

        this._modeItems = new Map();
        const modeSection = new PopupMenu.PopupMenuSection();
        for (const [mode, text] of MODES) {
            const item = new PopupMenu.PopupMenuItem(text);
            item.connect('activate', () => pickMode(settings, mode));
            modeSection.addMenuItem(item);
            this._modeItems.set(mode, item);
            this._labels.push([item.label, text]);
        }
        this._mixedItem = new PopupMenu.PopupSwitchMenuItem('Mieszane', settings.get_boolean('mixed'));
        this._mixedItem.connect('toggled', () => toggleMixed(settings));
        modeSection.addMenuItem(this._mixedItem);
        this._labels.push([this._mixedItem.label, 'Mieszane']);
        this.menu.addMenuItem(modeSection);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        this._intensityLabel = new PopupMenu.PopupMenuItem('', {reactive: false});
        this.menu.addMenuItem(this._intensityLabel);
        this._intensityItem = new PopupMenu.PopupBaseMenuItem({activate: false});
        this._intensity = new Slider(settings.get_int('intensity') / 100);
        this._intensity.x_expand = true;
        this._intensity.connect('notify::value', () => {
            const value = Math.round(this._intensity.value * 100);
            if (value !== settings.get_int('intensity'))
                settings.set_int('intensity', value);
        });
        this._intensityItem.add_child(this._intensity);
        this.menu.addMenuItem(this._intensityItem);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        this._clearItem = new PopupMenu.PopupMenuItem('Zdmuchnij śnieg');
        this._clearItem.connect('activate', () => clearSnow());
        this.menu.addMenuItem(this._clearItem);

        const optionsItem = new PopupMenu.PopupMenuItem('Opcje');
        optionsItem.connect('activate', () => openOptions());
        this.menu.addMenuItem(optionsItem);
        this._labels.push([optionsItem.label, 'Opcje']);

        this._language = null;
        this._mode = null;
        settings.connectObject('changed', () => this._sync(), this);
        this._sync();
    }

    _sync() {
        const language = this._settings.get_string('language');
        const known = MODES.map(([value]) => value);
        const active = activeModes(this._settings).filter(m => known.includes(m));
        const modes = active.length ? active : ['snow'];
        const key = `${language}|${modes.join(',')}`;
        if (key !== this._mode) {
            this._mode = key;
            setLanguage(language);
            const entries = modes.map(m => MODES.find(([value]) => value === m));
            const newest = entries.at(-1);
            const icon = this._icons[newest[0]];
            this.gicon = icon;
            this.title = entries.length > 1 ? _('Mieszane') : _(newest[2]);
            this.menu.setHeader(icon, this.title, _('Snowhush dla GNOME'));
            const clears = entries.map(entry => entry[3]).filter(Boolean);
            this._clearItem.visible = clears.length > 0;
            this._clearItem.label.text = clears.length > 1 ? _('Zdmuchnij wszystko') : _(clears[0] ?? 'Zdmuchnij śnieg');
            for (const [label, text] of this._labels)
                label.text = _(text);
        }
        for (const [value, item] of this._modeItems)
            item.setOrnament(modes.includes(value) ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NO_DOT);
        this._mixedItem.setToggleState(this._settings.get_boolean('mixed'));
        const intensity = this._settings.get_int('intensity');
        if (Math.round(this._intensity.value * 100) !== intensity)
            this._intensity.value = intensity / 100;
        this._intensityLabel.label.text = `${_('Intensywność')}: ${intensity}%`;
        this._clearItem.setSensitive(this.checked);
        this.subtitle = this.checked ? `${intensity}%` : null;
    }

    destroy() {
        this._settings.disconnectObject(this);
        super.destroy();
    }
});

export const SnowIndicator = GObject.registerClass(
class SnowIndicator extends SystemIndicator {
    _init(settings, clearSnow, openOptions, path) {
        super._init();
        this.quickSettingsItems.push(new SnowToggle(settings, clearSnow, openOptions, path));
    }

    destroy() {
        this.quickSettingsItems.forEach(item => item.destroy());
        super.destroy();
    }
});
