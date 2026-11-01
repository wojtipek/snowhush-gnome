export class Extension {
    constructor(meta) { Object.assign(this, meta); }
    openPreferences() { this.opened = (this.opened ?? 0) + 1; }
}
export class InjectionManager {
    overrideMethod() {}
    clear() {}
}
