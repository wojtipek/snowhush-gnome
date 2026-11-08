import {Emitter} from './common.mjs';
export class Slider extends Emitter {
    constructor(value) { super(); this._value = value; }
    get value() { return this._value; }
    set value(v) { this._value = v; this.emit('notify::value'); }
}
