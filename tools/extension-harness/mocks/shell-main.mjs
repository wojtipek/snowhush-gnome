import {Emitter} from './common.mjs';
export const layoutManager = Object.assign(new Emitter(), {
    monitors: [{x: 0, y: 0, width: 1920, height: 1080}, {x: 1920, y: 0, width: 2560, height: 1440}],
    getWorkAreaForMonitor(i) { return i === 0 ? {x: 0, y: 32, width: 1920, height: 1048} : {x: 1920, y: 0, width: 2560, height: 1440}; },
});
export const overview = Object.assign(new Emitter(), {visible: false});
export const sessionMode = Object.assign(new Emitter(), {isLocked: false});
export const panel = {statusArea: {quickSettings: {addExternalIndicator() {}}}};
