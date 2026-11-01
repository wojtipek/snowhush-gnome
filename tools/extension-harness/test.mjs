import assert from 'node:assert/strict';
import {Emitter} from './mocks/common.mjs';
import {FakeWindow} from './mocks/gi-Meta.mjs';
import * as Main from './mocks/shell-main.mjs';

const EXT = '../../extension/snowhush-gnome@wojtipek';

globalThis.spawned = [];
globalThis.timeouts = [];
const laters = [];
const written = [];
let partialNext = false;
const pendingWrites = [];

globalThis.makeProcess = () => {
    const stdin = {
        closed: false,
        write_bytes_async(bytes, prio, cancellable, cb) {
            const text = new TextDecoder().decode(bytes.data);
            const n = partialNext ? Math.ceil(bytes.data.length / 2) : bytes.data.length;
            partialNext = !partialNext;
            written.push(text.slice(0, new TextDecoder().decode(bytes.data.subarray(0, n)).length));
            pendingWrites.push(() => cb(stdin, n));
        },
        write_bytes_finish(n) { return n; },
        close_async(p, c, cb) { stdin.closed = true; cb(stdin, null); },
        close_finish() {},
    };
    const proc = {
        identifier: '4242',
        get_identifier() { return this.identifier; },
        get_stdin_pipe: () => stdin,
        get_stdout_pipe: () => ({}),
        wait_async(c, cb) { proc._wait = cb; },
        wait_finish() {},
        get_if_exited: () => true,
        get_exit_status: () => 1,
        force_exit() { proc.killed = true; },
        stdin,
    };
    globalThis.lastProcess = proc;
    return proc;
};

const display = Object.assign(new Emitter(), {
    get_monitor_scale: i => (i === 0 ? 1.25 : 1),
    get_n_monitors: () => 2,
    get_monitor_in_fullscreen: i => fullscreen.has(i),
});
const fullscreen = new Set();
const workspaces = [{index: 0}, {index: 1}];
let activeWorkspace = 0;
let actors = [];
const idleMonitor = {
    watches: new Map(), next: 1,
    get_idletime: () => 0,
    add_idle_watch(ms, fn) { const id = this.next++; this.watches.set(id, {kind: 'idle', ms, fn}); return id; },
    add_user_active_watch(fn) { const id = this.next++; this.watches.set(id, {kind: 'active', fn}); return id; },
    remove_watch(id) { this.watches.delete(id); },
    fire(kind) { for (const [id, w] of [...this.watches]) if (w.kind === kind) { if (kind === 'active') this.watches.delete(id); w.fn(); } },
};
globalThis.global = {
    backend: {get_core_idle_monitor: () => idleMonitor},
    display,
    context: {},
    workspace_manager: Object.assign(new Emitter(), {get_active_workspace: () => workspaces[activeWorkspace], get_active_workspace_index: () => activeWorkspace}),
    compositor: {get_laters: () => ({add(type, fn) { laters.push(fn); return laters.length; }, remove() {}})},
    get_window_actors: () => actors,
    get_pointer: () => [100, 200],
};

function drainWrites() {
    while (pendingWrites.length)
        pendingWrites.shift()();
}
function runLaters() {
    while (laters.length)
        laters.shift()();
}
function sent() {
    drainWrites();
    const text = written.join('');
    written.length = 0;
    return text.split('\n').filter(Boolean);
}

const {Engine} = await import(`${EXT}/engine.js`);
const {Scout} = await import(`${EXT}/scout.js`);

const back = new FakeWindow({id: 1, rect: {x: 100, y: 200, width: 800, height: 600}});
const front = new FakeWindow({id: 2, rect: {x: 300, y: 100, width: 400, height: 300}, maximized: 0});
const maximized = new FakeWindow({id: 3, rect: {x: 0, y: 32, width: 1920, height: 1048}, maximized: 3});
const otherWs = new FakeWindow({id: 4, ws: 1, rect: {x: 10, y: 10, width: 300, height: 300}});
const minimized = new FakeWindow({id: 5, minimized: true, rect: {x: 10, y: 10, width: 300, height: 300}});
const menu = new FakeWindow({id: 6, type: 10, rect: {x: 50, y: 60, width: 120, height: 200}});
const desktopIcons = new FakeWindow({id: 7, type: 1, rect: {x: 0, y: 0, width: 1920, height: 1080}});
actors = [desktopIcons, maximized, back, otherWs, minimized, front, menu].map(w => ({meta_window: w, visible: true}));

let started = 0;
const engine = new Engine('/ext/bin/snowhush-engine', () => started++);
const scout = new Scout(engine);
engine.start();
assert.deepEqual(globalThis.spawned[0], ['/ext/bin/snowhush-engine'], 'silnik uruchomiony z właściwą ścieżką');
assert.equal(started, 1);
scout.start();
let lines = sent();
assert.ok(lines.includes('monitors 0:0,32,1920,1048,1.250,0,0,1920,1080 1:1920,0,2560,1440,1.000,1920,0,2560,1440'), `monitory: ${lines}`);
assert.ok(lines.includes('covered 0'), 'nic nie zasłania');
assert.ok(lines.includes('power normal'));
assert.ok(lines.includes('elsewhere 4'), `okno na innym obszarze: ${lines}`);
const windowsLine = lines.find(l => l.startsWith('windows'));
assert.equal(windowsLine, 'windows 0 6:50,60,120,200,4 2:300,100,400,300,8 1:100,200,800,600,8 3:0,32,1920,1048,1',
    'okna od najbliższego, bez zminimalizowanych, pulpitu i okien z innego obszaru');
assert.ok(lines.includes('cursor 100 200'));

display.emit('grab-op-begin');
lines = sent();
assert.ok(lines.some(l => l.startsWith('windows 1 ')), 'początek przeciągania');
front.rect = {x: 320, y: 110, width: 400, height: 300};
front.emit('position-changed');
front.rect = {x: 340, y: 120, width: 400, height: 300};
front.emit('position-changed');
assert.equal(laters.length, 1, 'zmiany z jednej klatki łączone');
runLaters();
lines = sent();
assert.equal(lines.filter(l => l.startsWith('windows')).length, 1);
assert.ok(lines[0].includes('2:340,120,400,300,8'));
display.emit('grab-op-end');
assert.ok(sent().some(l => l.startsWith('windows 0 ')), 'koniec przeciągania');

globalThis.pointerCallback(500.4, 600.6);
assert.deepEqual(sent(), ['cursor 500 601']);
globalThis.pointerCallback(500.2, 600.7);
assert.deepEqual(sent(), []);

for (let i = 0; i < 50; i++)
    globalThis.pointerCallback(i, i);
lines = sent();
assert.ok(lines.length <= 3, `kursor sklejony: ${lines.length} wiadomości`);
assert.equal(lines.at(-1), 'cursor 49 49');

Main.overview.visible = true;
Main.overview.emit('showing');
assert.deepEqual(sent(), ['covered 1']);
Main.overview.visible = false;
Main.overview.emit('hidden');
fullscreen.add(1);
display.emit('in-fullscreen-changed');
assert.equal(sent().at(-1), 'covered 0 1');
fullscreen.clear();
Main.sessionMode.isLocked = true;
Main.sessionMode.emit('updated');
assert.deepEqual(sent(), ['covered 1']);
Main.sessionMode.isLocked = false;
Main.sessionMode.emit('updated');
sent();

assert.ok([...idleMonitor.watches.values()].some(w => w.kind === 'idle' && w.ms === 300000), 'czujka bezczynności 5 min');
idleMonitor.fire('idle');
assert.ok(sent().includes('idle 1'), 'bezczynność zgłoszona');
idleMonitor.fire('active');
assert.ok(sent().includes('idle 0'), 'powrót aktywności zgłoszony');

activeWorkspace = 1;
global.workspace_manager.emit('active-workspace-changed');
lines = sent();
assert.equal(lines[0], 'workspace');
assert.ok(lines.includes('slide -6000'), `wiatr w lewo przy przejściu w prawo: ${lines}`);
assert.ok(lines.some(l => l.startsWith('windows 0 4:')), `okna nowego obszaru: ${lines}`);
assert.ok(lines.includes('elsewhere 2 1 3'), `stare okna „gdzie indziej”: ${lines}`);
activeWorkspace = 0;
global.workspace_manager.emit('active-workspace-changed');
assert.ok(sent().includes('slide 6000'), 'wiatr w prawo przy powrocie');

scout.setSwitching(true);
lines = sent();
assert.ok(lines.includes('elsewhere 2 4 1 3'), `wszystkie okna „gdzie indziej”: ${lines}`);
assert.ok(lines.includes('windows 0'), `żadnych okien: ${lines}`);
scout.setSwitching(false);
lines = sent();
assert.ok(lines.includes('elsewhere 4'), `po animacji: ${lines}`);
assert.ok(lines.some(l => l.startsWith('windows 0 6:') && l.includes('2:')), `okna wracają: ${lines}`);

const overlay = new FakeWindow({id: 99, title: null, rect: {x: 0, y: 0, width: 1, height: 1}});
overlay._owner = globalThis.lastProcess;
actors.push({meta_window: overlay, visible: true});
display.emit('window-created', overlay);
assert.equal(overlay.setType, 2, 'typ DOCK');
assert.ok(overlay.hidden && overlay.sticky);
assert.equal(overlay.moves.length, 0, 'bez tytułu nie wiadomo, który monitor');
overlay.title = 'snowhush-overlay-1';
overlay.emit('notify::title');
assert.deepEqual(overlay.moves.at(-1), [1920, 0, 2560, 1440], 'ustawiona na monitorze 1');
runLaters();
scout._sendWindows(true);
assert.ok(!sent().join(' ').includes('99:'), 'nakładka nie jest traktowana jak okno');
engine.setBehindWindows(true);
assert.equal(overlay.setType, 1, 'typ DESKTOP za oknami');
engine.setBehindWindows(false);

Main.layoutManager.getWorkAreaForMonitor = i => ({x: 1920, y: 40, width: 2500, height: 1400});
display.emit('workareas-changed');
assert.deepEqual(overlay.moves.at(-1), [1920, 40, 2500, 1400]);

overlay.rect = {x: 1920, y: 0, width: 2500, height: 1400};
const movesBefore = overlay.moves.length;
overlay.emit('size-changed');
assert.deepEqual(overlay.moves.at(-1), [1920, 40, 2500, 1400], 'nakładka poprawiona po niedokończonym przesunięciu');
overlay.emit('position-changed');
assert.equal(overlay.moves.length, movesBefore + 1, 'na miejscu: bez kolejnych przesunięć');
overlay.move_resize_frame = function (u, x, y, w, h) { this.moves.push([x, y, w, h]); };
for (let k = 0; k < 20; k++)
    overlay.emit('position-changed');
assert.ok(overlay.moves.length <= movesBefore + 1 + 5, `ogranicznik prób: ${overlay.moves.length - movesBefore}`);
delete overlay.move_resize_frame;

const first = globalThis.lastProcess;
first._wait(first, null);
assert.equal(globalThis.timeouts.length, 1, 'zaplanowany restart');
globalThis.timeouts.shift()();
assert.equal(globalThis.spawned.length, 2, 'drugi start');
assert.equal(started, 2);

scout.stop();
engine.destroy();
assert.ok(globalThis.lastProcess.stdin.closed, 'wejście zamknięte');
globalThis.timeouts.forEach(fn => fn());
assert.ok(globalThis.lastProcess.killed, 'silnik zabity po czasie');
assert.equal(display.count('window-created'), 0, 'sygnały odłączone');
assert.equal(front.count('position-changed'), 0, 'sygnały okien odłączone');
assert.equal(globalThis.pointerCallback, null, 'kursor nieobserwowany');
const i18n = await import(`${EXT}/i18n.js`);
const pick = i18n.pickLanguage;
assert.equal(pick(['pl_PL.UTF-8', 'pl', 'C']), null, 'polski = podstawa');
assert.equal(pick(['en_US.UTF-8', 'en_US', 'en', 'C']), 'en', 'amerykański → brytyjski');
assert.equal(pick(['es_ES.UTF-8', 'es', 'C']), 'es', 'Hiszpania → meksykański');
assert.equal(pick(['pt_PT.UTF-8', 'pt', 'C']), 'pt', 'Portugalia → brazylijski');
assert.equal(pick(['it_IT.UTF-8', 'it', 'C']), 'la', 'włoski jest spoza listy → łacina');
assert.equal(pick(['fr_FR.UTF-8', 'fr', 'C']), 'la', 'spoza listy → łacina');
assert.equal(pick(['fr_FR.UTF-8', 'fr', 'de_DE', 'de', 'C']), 'de', 'kolejny język z preferencji');
assert.equal(pick(['C']), 'la', 'brak języka → łacina');
assert.equal(pick(['sr_RS@latin', 'C']), 'la');
assert.equal(i18n._('Śnieg'), 'Śnieg', 'w atrapie system jest po polsku');
for (const [code, list] of Object.entries(i18n.TRANSLATIONS)) {
    assert.equal(list.length, i18n.POLISH.length, `komplet napisów: ${code}`);
    assert.ok(list.every(text => typeof text === 'string' && text.trim()), `puste napisy: ${code}`);
}
assert.deepEqual(Object.keys(i18n.TRANSLATIONS).sort(),
    ['ca', 'cs', 'de', 'en', 'eo', 'es', 'hr', 'hu', 'la', 'lt', 'pt', 'ru', 'sk', 'uk']);
const fs = await import('node:fs');
const used = ['indicator.js', 'prefs.js'].flatMap(file => fs.readFileSync(new URL(`${EXT}/${file}`, import.meta.url), 'utf8')
    .match(/'([A-ZĆŁŚŻŹ][^']*)'/g).map(m => m.slice(1, -1)));
for (const text of used)
    assert.ok(i18n.POLISH.includes(text), `napis bez tłumaczenia: ${text}`);

class FakeSettings extends Emitter {
    constructor(values) { super(); this.values = values; }
    get_string(k) { return this.values[k]; }
    get_boolean(k) { return this.values[k]; }
    get_int(k) { return this.values[k]; }
    set_string(k, v) { this.values[k] = v; this._changed(k); }
    set_boolean(k, v) { this.values[k] = v; this._changed(k); }
    _changed(k) {
        for (const h of [...this._handlers])
            if (h.name === 'changed' || h.name === `changed::${k}`)
                h.fn(this, k);
    }
    bind(key, obj, prop) {
        obj[prop] = this.values[key];
        this.connect(`changed::${key}`, () => { obj[prop] = this.values[key]; });
    }
}
const menuSettings = new FakeSettings({enabled: true, snowfall: 'heavy', language: '', accumulate: true, melt: true,
    'snow-on-windows': true, 'over-windows': true, 'behind-windows': false, sharp: false, halloween: true,
    'idle-slowdown': false, 'rest-when-hidden': false, 'battery-slowdown': false, autumn: false});
let cleared = 0, opened = 0;
const {SnowIndicator} = await import(`${EXT}/indicator.js`);
const indicator = new SnowIndicator(menuSettings, () => cleared++, () => opened++, '/ext');
const toggle = indicator.quickSettingsItems[0];
const menuItems = () => toggle.menu.items.flatMap(i => i.items?.length ? i.items : [i]).filter(i => i.label);
const labels = () => menuItems().map(i => i.label.text);
const item = text => menuItems().find(i => i.label.text === text);
assert.deepEqual(labels(), ['Śnieg', 'Jesienne liście', 'Bez opadów', 'Lekki', 'Umiarkowany', 'Gęsty', 'Zmienny', 'Zdmuchnij śnieg', 'Opcje']);
assert.equal(toggle.title, 'Śnieg');
assert.equal(toggle.subtitle, 'Gęsty');
assert.equal(toggle.gicon.name, 'weather-snow-symbolic');
assert.deepEqual(toggle.menu.header.slice(1), ['Śnieg', 'Snowhush dla GNOME']);
assert.equal(toggle.menu.header[0].name, 'weather-snow-symbolic');
assert.equal(menuItems()[0].ornament, 2);
assert.equal(menuItems()[1].ornament, 3);
item('Jesienne liście').activate();
assert.equal(menuSettings.values.autumn, true);
assert.equal(toggle.title, 'Liście');
assert.equal(toggle.gicon.file.path, '/ext/icons/leaf-symbolic.svg');
assert.equal(toggle.menu.header[1], 'Liście');
assert.equal(menuItems()[0].ornament, 3);
assert.equal(menuItems()[1].ornament, 2);
assert.ok(labels().includes('Zdmuchnij liście') && !labels().includes('Zdmuchnij śnieg'));
menuSettings.set_string('language', 'de');
assert.equal(toggle.title, 'Laub');
assert.ok(labels().includes('Herbstlaub') && labels().includes('Laub wegpusten'), `${labels()}`);
menuSettings.set_string('language', '');
item('Śnieg').activate();
assert.equal(menuSettings.values.autumn, false);
assert.equal(toggle.title, 'Śnieg');
assert.equal(toggle.gicon.name, 'weather-snow-symbolic');
item('Lekki').activate();
assert.equal(menuSettings.values.snowfall, 'light');
assert.equal(toggle.subtitle, 'Lekki');
item('Zdmuchnij śnieg').activate();
assert.equal(cleared, 1);
item('Opcje').activate();
assert.equal(opened, 1);

const {default: Prefs} = await import(`${EXT}/prefs.js`);
const window = {add(page) { this.page = page; }, set_default_size() {}, title: ''};
new Prefs(menuSettings).fillPreferencesWindow(window);
const [snowGroup, languageGroup] = window.page.children;
assert.equal(window.title, 'Snowhush dla GNOME');
assert.equal(snowGroup.title, 'Śnieg');
assert.deepEqual(snowGroup.children.map(r => r.title),
    ['Gromadzenie', 'Topnienie', 'Śnieg na oknach', 'Płatki nad oknami', 'Za oknami', 'Ostrzejszy obraz', 'Halloween',
        'Wolniej po 5 min bezczynności', 'Pauza, gdy okno zasłania śnieg', 'Wolniej na baterii']);
assert.deepEqual(snowGroup.children.map(r => r.active), [true, true, true, true, false, false, true, false, false, false]);
assert.equal(snowGroup.children[8].sensitive, false, 'pauza ma sens dopiero, gdy płatki są za oknami');
menuSettings.set_boolean('over-windows', false);
assert.equal(snowGroup.children[8].sensitive, true);
menuSettings.set_boolean('over-windows', true);
const languageRow = languageGroup.children[0];
assert.equal(languageRow.title, 'Język');
assert.equal(languageRow.model.items.length, 16);
assert.equal(languageRow.model.items[0], 'Automatycznie');
assert.equal(languageRow.selected, 0);
languageRow.selected = languageRow.model.items.indexOf('Latina');
assert.equal(menuSettings.values.language, 'la');
assert.equal(window.title, 'Snowhush ad GNOME');
assert.equal(languageRow.title, 'Lingua');
assert.equal(languageRow.model.items[0], 'Sponte');
assert.equal(languageRow.selected, languageRow.model.items.indexOf('Sponte') === 0 ? languageRow.model.items.indexOf('Latina') : -1);
assert.equal(snowGroup.children[0].title, 'Cumulatio');
assert.equal(toggle.title, 'Nix');
assert.ok(labels().includes('Nivem diffla') && labels().includes('Optiones'), `${labels()}`);
menuSettings.set_string('language', 'en');
assert.equal(languageRow.selected, languageRow.model.items.indexOf('English (UK)'));
assert.deepEqual(languageRow.model.items.slice(1, 5), ['Català', 'Čeština', 'Deutsch', 'English (UK)'], 'alfabetycznie');
assert.equal(languageRow.model.items.at(-1), 'Українська');
assert.equal(snowGroup.title, 'Snow');
languageRow.selected = 0;
assert.equal(menuSettings.values.language, '');
assert.equal(toggle.title, 'Śnieg');
assert.equal(languageRow.model.items[0], 'Automatycznie');
menuSettings.set_boolean('behind-windows', true);
assert.equal(snowGroup.children[4].active, true);
assert.equal(snowGroup.children[2].sensitive, false);
assert.equal(snowGroup.children[3].sensitive, false);
assert.equal(snowGroup.children[8].sensitive, true, 'za oknami pauza ma sens');
snowGroup.children[0].active = false;
menuSettings.set_boolean('enabled', false);
assert.equal(toggle.subtitle, null);
assert.equal(item('Zdmuchnij śnieg').sensitive, false);

const {TRANSLATIONS} = await import(`${EXT}/i18n.js`);
for (const [code, list] of Object.entries(TRANSLATIONS))
    assert.equal(list.length, 24, `pełne tłumaczenie: ${code}`);

const {default: SnowhushExtension} = await import(`${EXT}/extension.js`);
const extSettings = new FakeSettings({...menuSettings.values, 'calm-fps': 20, enabled: false, autumn: true});
const ext = new SnowhushExtension({path: '/ext'});
ext.getSettings = () => extSettings;
ext.enable();
const extSent = [];
ext._engine = {send: line => extSent.push(line), start() {}, stop() {}, setBehindWindows() {}, destroy() {}};
ext._onEngineStarted();
assert.ok(extSent.includes('set autumn 1'), 'start od razu w liściach, bez podmuchu');
assert.ok(!extSent.some(l => l.startsWith('mode')));
extSent.length = 0;
extSettings.set_boolean('autumn', false);
assert.deepEqual(extSent, ['mode snow 0']);
(await import('./mocks/gi-St.mjs')).default.Settings.get().enable_animations = false;
extSettings.set_boolean('autumn', true);
assert.deepEqual(extSent, ['mode snow 0', 'mode autumn 1']);
ext.disable();

console.log('rozszerzenie: wszystkie testy przeszły');
