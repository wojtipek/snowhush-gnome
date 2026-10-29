import {Emitter} from './common.mjs';
const Meta = {
    WindowType: {NORMAL: 0, DESKTOP: 1, DOCK: 2, DIALOG: 3, MODAL_DIALOG: 4, TOOLBAR: 5, MENU: 6, UTILITY: 7, SPLASHSCREEN: 8, DROPDOWN_MENU: 9, POPUP_MENU: 10, TOOLTIP: 11, NOTIFICATION: 12, COMBO: 13, DND: 14, OVERRIDE_OTHER: 15},
    LaterType: {BEFORE_REDRAW: 1},
    MaximizeFlags: {HORIZONTAL: 1, VERTICAL: 2, BOTH: 3},
    WaylandClient: {
        new_subprocess(context, launcher, argv) {
            globalThis.spawned.push(argv);
            const proc = globalThis.makeProcess();
            return {get_subprocess: () => proc, owns_window: w => w._owner === proc};
        },
    },
};
export class FakeWindow extends Emitter {
    constructor(props) { super(); Object.assign(this, {type: 0, minimized: false, maximized: 0, fullscreen: false, allWs: false, ws: 0, title: 'x', pid: 1, wmClass: 'app', moves: []}, props); }
    get_window_type() { return this.type; }
    is_override_redirect() { return false; }
    is_on_all_workspaces() { return this.allWs; }
    located_on_workspace(ws) { return ws.index === this.ws; }
    showing_on_its_workspace() { return !this.minimized; }
    get_frame_rect() { return {...this.rect}; }
    is_maximized() { return this.maximized === 3; }
    get_maximize_flags() { return this.maximized; }
    is_fullscreen() { return this.fullscreen; }
    get_layer() { return 2; }
    get_id() { return this.id; }
    get_title() { return this.title; }
    get_pid() { return this.pid; }
    get_wm_class() { return this.wmClass; }
    set_type(t) { this.setType = t; }
    hide_from_window_list() { this.hidden = true; }
    stick() { this.sticky = true; }
    move_resize_frame(user, x, y, w, h) { this.moves.push([x, y, w, h]); this.rect = {x, y, width: w, height: h}; }
}
export default Meta;
