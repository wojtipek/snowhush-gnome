import {Emitter} from './common.mjs';
export const Ornament = {NONE: 0, DOT: 1, CHECK: 2, NO_DOT: 3};
class Base extends Emitter {
    constructor(...args) { super(); this._init(...args); }
    _init() { this.items = []; this.sensitive = true; }
    addMenuItem(item) { this.items.push(item); }
    setSensitive(value) { this.sensitive = value; }
    destroy() {}
}
export class PopupMenuSection extends Base {}
export class PopupSeparatorMenuItem extends Base {}
export class PopupMenuItem extends Base {
    _init(text) { super._init(); this.label = {text}; this.ornament = Ornament.NONE; }
    setOrnament(o) { this.ornament = o; }
    activate() { this.emit('activate'); }
}
export class PopupSwitchMenuItem extends PopupMenuItem {
    _init(text, state) { super._init(text); this.state = state; }
    setToggleState(state) { this.state = state; }
    toggle() { this.state = !this.state; this.emit('toggled', this.state); }
}
export class PopupSubMenuMenuItem extends PopupMenuItem {
    _init(text) { super._init(text); this.menu = new PopupMenuSection(); this.menu.actor = {style: null}; }
}
