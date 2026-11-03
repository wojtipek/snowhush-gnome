import GObject from 'gi://GObject';
import Gio from 'gi://Gio';

import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {QuickMenuToggle, SystemIndicator} from 'resource:///org/gnome/shell/ui/quickSettings.js';

import {_, setLanguage} from './i18n.js';

const ICON = 'weather-snow-symbolic';

const MODES = [
    ['snow', 'Śnieg', 'Śnieg', 'Zdmuchnij śnieg'],
    ['autumn', 'Jesienne liście', 'Liście', 'Zdmuchnij liście'],
    ['rain', 'Deszcz', 'Deszcz', null],
];

export const SNOWFALL = [
    ['off', 'Bez opadów'],
    ['light', 'Lekki'],
    ['steady', 'Umiarkowany'],
    ['heavy', 'Gęsty'],
    ['variable', 'Zmienny'],
];

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
        };
        settings.bind('enabled', this, 'checked', Gio.SettingsBindFlags.DEFAULT);

        this._modeItems = new Map();
        const modeSection = new PopupMenu.PopupMenuSection();
        for (const [mode, text] of MODES) {
            const item = new PopupMenu.PopupMenuItem(text);
            item.connect('activate', () => settings.set_string('mode', mode));
            modeSection.addMenuItem(item);
            this._modeItems.set(mode, item);
            this._labels.push([item.label, text]);
        }
        this.menu.addMenuItem(modeSection);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        this._snowfallItems = new Map();
        const snowfallSection = new PopupMenu.PopupMenuSection();
        for (const [value, text] of SNOWFALL) {
            const item = new PopupMenu.PopupMenuItem(text);
            item.connect('activate', () => settings.set_string('snowfall', value));
            snowfallSection.addMenuItem(item);
            this._snowfallItems.set(value, item);
            this._labels.push([item.label, text]);
        }
        this.menu.addMenuItem(snowfallSection);
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
        const found = MODES.find(([value]) => value === this._settings.get_string('mode')) ?? MODES[0];
        const [mode, , title, clear] = found;
        if (language !== this._language || mode !== this._mode) {
            this._language = language;
            this._mode = mode;
            setLanguage(language);
            const icon = this._icons[mode];
            this.gicon = icon;
            this.title = _(title);
            this.menu.setHeader(icon, this.title, _('Snowhush dla GNOME'));
            this._clearItem.visible = clear !== null;
            if (clear)
                this._clearItem.label.text = _(clear);
            for (const [label, text] of this._labels)
                label.text = _(text);
        }
        for (const [value, item] of this._modeItems)
            item.setOrnament(value === mode ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NO_DOT);
        const snowfall = this._settings.get_string('snowfall');
        for (const [value, item] of this._snowfallItems)
            item.setOrnament(value === snowfall ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NO_DOT);
        this._clearItem.setSensitive(this.checked);
        const text = SNOWFALL.find(([value]) => value === snowfall)?.[1];
        this.subtitle = this.checked && text ? _(text) : null;
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
