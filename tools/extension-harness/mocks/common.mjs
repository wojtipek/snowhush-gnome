export class Emitter {
    constructor() { this._handlers = []; this._next = 1; }
    connect(name, fn) { const id = this._next++; this._handlers.push({id, name, fn, owner: null}); return id; }
    disconnect(id) { this._handlers = this._handlers.filter(h => h.id !== id); }
    connectObject(...args) {
        const owner = args.pop();
        for (let i = 0; i < args.length; i += 2) { const id = this.connect(args[i], args[i + 1]); this._handlers.find(h => h.id === id).owner = owner; }
    }
    disconnectObject(owner) { this._handlers = this._handlers.filter(h => h.owner !== owner); }
    emit(name, ...args) { for (const h of [...this._handlers]) if (h.name === name) h.fn(this, ...args); }
    count(name) { return this._handlers.filter(h => h.name === name).length; }
}
