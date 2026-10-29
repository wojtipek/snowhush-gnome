import {Emitter} from './common.mjs';
class Widget extends Emitter {
    constructor(props = {}) { super(); this.children = []; this.sensitive = true; this.title = ''; Object.assign(this, props); }
    add(child) { this.children.push(child); }
}
class ComboRow extends Widget {
    constructor(props) { super(props); this._selected = 0; }
    get selected() { return this._selected; }
    set selected(v) { this._selected = v; this.emit('notify::selected'); }
}
const Adw = {PreferencesPage: Widget, PreferencesGroup: Widget, SwitchRow: Widget, ComboRow};
export default Adw;
