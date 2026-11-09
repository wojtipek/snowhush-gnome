import Adw from 'gi://Adw';
import Gtk from 'gi://Gtk';

import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

import {_, LANGUAGES, setLanguage} from './i18n.js';

const HALLOWEEN = [
    ['season', 'Od 30 października do 1 listopada'],
    ['always', 'Zawsze'],
    ['off', 'Wyłączone'],
];

const SWITCHES = [
    ['melt', 'Topnienie'],
    ['over-windows', 'Płatki nad oknami'],
    ['behind-windows', 'Za oknami'],
    ['sharp', 'Ostrzejszy obraz'],
    ['idle-slowdown', 'Wolniej po 5 min bezczynności'],
    ['rest-when-hidden', 'Pauza, gdy okno zasłania śnieg'],
    ['battery-slowdown', 'Wolniej na baterii'],
];

export default class SnowhushPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        const settings = this.getSettings();
        window._settings = settings;
        window.set_default_size(480, 640);
        setLanguage(settings.get_string('language'));

        const labels = [];
        const page = new Adw.PreferencesPage({icon_name: 'weather-snow-symbolic'});
        window.add(page);

        const snowGroup = new Adw.PreferencesGroup();
        page.add(snowGroup);
        labels.push([snowGroup, 'Śnieg']);
        const rows = new Map();
        for (const [key, text] of SWITCHES) {
            const row = new Adw.SwitchRow();
            settings.bind(key, row, 'active', 0);
            snowGroup.add(row);
            rows.set(key, row);
            labels.push([row, text]);
        }

        const opacityRow = new Adw.ActionRow();
        const opacity = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 10, 100, 5);
        opacity.set_size_request(200, -1);
        opacity.valign = Gtk.Align.CENTER;
        opacity.draw_value = true;
        opacity.value_pos = Gtk.PositionType.LEFT;
        opacity.set_format_value_func((_scale, value) => `${Math.round(value)}%`);
        opacity.set_value(settings.get_int('opacity'));
        opacity.connect('value-changed', () => {
            const value = Math.round(opacity.get_value());
            if (value !== settings.get_int('opacity'))
                settings.set_int('opacity', value);
        });
        settings.connect('changed::opacity', () => {
            if (Math.round(opacity.get_value()) !== settings.get_int('opacity'))
                opacity.set_value(settings.get_int('opacity'));
        });
        opacityRow.add_suffix(opacity);
        snowGroup.add(opacityRow);
        labels.push([opacityRow, 'Widoczność opadów']);

        const halloweenNames = Gtk.StringList.new(HALLOWEEN.map(([, text]) => text));
        const halloweenRow = new Adw.ComboRow({model: halloweenNames});
        snowGroup.add(halloweenRow);
        labels.push([halloweenRow, 'Halloween']);

        const languageGroup = new Adw.PreferencesGroup();
        page.add(languageGroup);
        const sorted = [...LANGUAGES].sort(([, a], [, b]) => a.localeCompare(b));
        const codes = ['', ...sorted.map(([code]) => code)];
        const names = Gtk.StringList.new(['', ...sorted.map(([, name]) => name)]);
        const languageRow = new Adw.ComboRow({model: names});
        languageGroup.add(languageRow);
        labels.push([languageRow, 'Język']);

        let updating = false;
        const relabel = () => {
            updating = true;
            setLanguage(settings.get_string('language'));
            window.title = _('Snowhush dla GNOME');
            page.title = _('Opcje');
            for (const [widget, text] of labels)
                widget.title = _(text);
            names.splice(0, 1, [_('Automatycznie')]);
            halloweenNames.splice(0, HALLOWEEN.length, HALLOWEEN.map(([, text]) => _(text)));
            syncHalloween();
            updating = false;
        };
        const syncLanguage = () => {
            const index = Math.max(0, codes.indexOf(settings.get_string('language')));
            if (languageRow.selected !== index) {
                updating = true;
                languageRow.selected = index;
                updating = false;
            }
        };
        const syncHalloween = () => {
            const index = Math.max(0, HALLOWEEN.findIndex(([value]) => value === settings.get_string('halloween-days')));
            if (halloweenRow.selected !== index) {
                const was = updating;
                updating = true;
                halloweenRow.selected = index;
                updating = was;
            }
        };
        const syncSensitive = () => {
            const behind = settings.get_boolean('behind-windows');
            rows.get('over-windows').sensitive = !behind;
            rows.get('rest-when-hidden').sensitive = behind || !settings.get_boolean('over-windows');
        };

        relabel();
        syncLanguage();
        syncSensitive();
        halloweenRow.connect('notify::selected', () => {
            if (updating)
                return;
            const value = HALLOWEEN[halloweenRow.selected]?.[0] ?? 'season';
            if (settings.get_string('halloween-days') !== value)
                settings.set_string('halloween-days', value);
        });
        settings.connect('changed::halloween-days', syncHalloween);
        languageRow.connect('notify::selected', () => {
            if (updating)
                return;
            const code = codes[languageRow.selected] ?? '';
            if (settings.get_string('language') !== code)
                settings.set_string('language', code);
        });
        settings.connect('changed::language', () => {
            relabel();
            syncLanguage();
        });
        settings.connect('changed::behind-windows', syncSensitive);
        settings.connect('changed::over-windows', syncSensitive);
    }
}
