import {Emitter} from './common.mjs';
class Scale extends Emitter {
    constructor(min, max) { super(); this.min = min; this.max = max; this.value = min; }
    set_value(v) { this.value = Math.min(this.max, Math.max(this.min, v)); this.emit('value-changed'); }
    get_value() { return this.value; }
    set_size_request() {}
    set_format_value_func(fn) { this.format = fn; }
}
const Gtk = {
    Scale: {new_with_range(_o, min, max) { return new Scale(min, max); }},
    Orientation: {HORIZONTAL: 0}, Align: {CENTER: 3}, PositionType: {LEFT: 0},
    StringList: {new(items) { return {items: [...items], splice(pos, n, add) { this.items.splice(pos, n, ...add); }, get_string(i) { return this.items[i]; }}; }},
};
export default Gtk;
