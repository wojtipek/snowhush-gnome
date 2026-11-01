import GObject from 'gi://GObject';
import Gio from 'gi://Gio';

import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {QuickMenuToggle, SystemIndicator} from 'resource:///org/gnome/shell/ui/quickSettings.js';

import {_, setLanguage} from './i18n.js';

const ICON = 'weather-snow-symbolic';

const MODES = [
    [false, 'Śnieg'],
    [true, 'Jesienne liście'],
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
        this._snowIcon = new Gio.ThemedIcon({name: ICON});
        this._leafIcon = new Gio.FileIcon({file: Gio.File.new_for_path(`${path}/icons/leaf-symbolic.svg`)});
        settings.bind('enabled', this, 'checked', Gio.SettingsBindFlags.DEFAULT);

        this._modeItems = new Map();
        const modeSection = new PopupMenu.PopupMenuSection();
        for (const [autumn, text] of MODES) {
            const item = new PopupMenu.PopupMenuItem(text);
            item.connect('activate', () => settings.set_boolean('autumn', autumn));
            modeSection.addMenuItem(item);
            this._modeItems.set(autumn, item);
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
        this._autumn = null;
        settings.connectObject('changed', () => this._sync(), this);
        this._sync();
    }

    _sync() {
        const language = this._settings.get_string('language');
        const autumn = this._settings.get_boolean('autumn');
        if (language !== this._language || autumn !== this._autumn) {
            this._language = language;
            this._autumn = autumn;
            setLanguage(language);
            const icon = autumn ? this._leafIcon : this._snowIcon;
            this.gicon = icon;
            this.title = _(autumn ? 'Liście' : 'Śnieg');
            this.menu.setHeader(icon, this.title, _('Snowhush dla GNOME'));
            this._clearItem.label.text = _(autumn ? 'Zdmuchnij liście' : 'Zdmuchnij śnieg');
            for (const [label, text] of this._labels)
                label.text = _(text);
        }
        for (const [value, item] of this._modeItems)
            item.setOrnament(value === autumn ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NO_DOT);
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
