export function getPointerWatcher() {
    return {addWatch(interval, cb) { globalThis.pointerCallback = cb; return {remove() { globalThis.pointerCallback = null; }}; }};
}
