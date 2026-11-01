const Gio = {
    SubprocessFlags: {STDIN_PIPE: 1, STDOUT_PIPE: 2},
    SubprocessLauncher: class { constructor(p) { this.p = p; } setenv() {} spawnv(argv) { return globalThis.makeProcess(); } },
    DataInputStream: class { constructor() {} read_line_async() {} },
    DBusProxy: {new_for_bus() {}},
    BusType: {SYSTEM: 1}, DBusProxyFlags: {DO_NOT_AUTO_START: 1},
    SettingsBindFlags: {DEFAULT: 0},
    ThemedIcon: class { constructor(p) { this.name = p.name; } },
    FileIcon: class { constructor(p) { this.file = p.file; } },
    File: {new_for_path(path) { return {path}; }},
};
export default Gio;
