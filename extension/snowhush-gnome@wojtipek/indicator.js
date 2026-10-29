import GObject from 'gi://GObject';
import Gio from 'gi://Gio';

import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {QuickMenuToggle, SystemIndicator} from 'resource:///org/gnome/shell/ui/quickSettings.js';

import {_, setLanguage} from './i18n.js';

const ICON = 'weather-snow-symbolic';

export const SNOWFALL = [
    ['off', 'Bez opadów'],
    ['light', 'Lekki'],
    ['steady', 'Umiarkowany'],
    ['heavy', 'Gęsty'],
    ['variable', 'Zmienny'],
];

const SnowToggle = GObject.registerClass(
class SnowToggle extends QuickMenuToggle {
    _init(settings, clearSnow, openOptions) {
        super._init({
            iconName: ICON,
            toggleMode: true,
        });
        this._settings = settings;
        this._labels = [];
        settings.bind('enabled', this, 'checked', Gio.SettingsBindFlags.DEFAULT);

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
        this._labels.push([this._clearItem.label, 'Zdmuchnij śnieg']);

        const optionsItem = new PopupMenu.PopupMenuItem('Opcje');
        optionsItem.connect('activate', () => openOptions());
        this.menu.addMenuItem(optionsItem);
        this._labels.push([optionsItem.label, 'Opcje']);

        this._language = null;
        settings.connectObject('changed', () => this._sync(), this);
        this._sync();
    }

    _sync() {
        const language = this._settings.get_string('language');
        if (language !== this._language) {
            this._language = language;
            setLanguage(language);
            this.title = _('Śnieg');
            this.menu.setHeader(ICON, _('Śnieg'), _('Snowhush dla GNOME'));
            for (const [label, text] of this._labels)
                label.text = _(text);
        }
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
    _init(settings, clearSnow, openOptions) {
        super._init();
        this.quickSettingsItems.push(new SnowToggle(settings, clearSnow, openOptions));
    }

    destroy() {
        this.quickSettingsItems.forEach(item => item.destroy());
        super.destroy();
    }
});
