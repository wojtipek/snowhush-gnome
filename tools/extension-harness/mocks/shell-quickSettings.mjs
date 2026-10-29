import {Emitter} from './common.mjs';
import {PopupMenuSection} from './shell-popupMenu.mjs';
export class QuickMenuToggle extends Emitter {
    constructor(...args) { super(); this._init(...args); }
    _init(props) {
        Object.assign(this, props);
        this.menu = new PopupMenuSection();
        this.menu.setHeader = (icon, title, subtitle) => { this.menu.header = [icon, title, subtitle]; };
    }
    destroy() {}
}
export class SystemIndicator extends Emitter {
    constructor(...args) { super(); this._init(...args); }
    _init() { this.quickSettingsItems = []; }
    destroy() {}
}
