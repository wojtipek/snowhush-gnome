use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::OnceLock;

use crate::ffi::Library;
use crate::load;

#[repr(C)]
pub struct WlInterface {
    name: *const c_char,
    version: c_int,
    method_count: c_int,
    methods: *const WlMessage,
    event_count: c_int,
    events: *const WlMessage,
}

#[repr(C)]
struct WlMessage {
    name: *const c_char,
    signature: *const c_char,
    types: *const *const WlInterface,
}

#[repr(C)]
#[derive(Clone, Copy)]
union WlArgument {
    i: i32,
    u: u32,
    s: *const c_char,
    o: *mut c_void,
}

type Proxy = *mut c_void;
const MARSHAL_DESTROY: u32 = 1;

struct Functions {
    display_connect: unsafe extern "C" fn(*const c_char) -> Proxy,
    display_get_fd: unsafe extern "C" fn(Proxy) -> c_int,
    display_dispatch_pending: unsafe extern "C" fn(Proxy) -> c_int,
    display_roundtrip: unsafe extern "C" fn(Proxy) -> c_int,
    display_flush: unsafe extern "C" fn(Proxy) -> c_int,
    display_prepare_read: unsafe extern "C" fn(Proxy) -> c_int,
    display_read_events: unsafe extern "C" fn(Proxy) -> c_int,
    display_cancel_read: unsafe extern "C" fn(Proxy),
    display_get_error: unsafe extern "C" fn(Proxy) -> c_int,
    proxy_marshal_array_flags: unsafe extern "C" fn(Proxy, u32, *const WlInterface, u32, u32, *mut WlArgument) -> Proxy,
    proxy_add_listener: unsafe extern "C" fn(Proxy, *const *const c_void, *mut c_void) -> c_int,
    proxy_get_version: unsafe extern "C" fn(Proxy) -> u32,
    proxy_destroy: unsafe extern "C" fn(Proxy),
    egl_window_create: unsafe extern "C" fn(Proxy, c_int, c_int) -> *mut c_void,
    egl_window_resize: unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, c_int),
    egl_window_destroy: unsafe extern "C" fn(*mut c_void),
}

struct Interfaces {
    registry: usize,
    compositor: usize,
    surface: usize,
    region: usize,
    callback: usize,
    wm_base: usize,
    xdg_surface: usize,
    toplevel: usize,
}

static FUNCTIONS: OnceLock<Functions> = OnceLock::new();
static INTERFACES: OnceLock<Interfaces> = OnceLock::new();

fn f() -> &'static Functions {
    FUNCTIONS.get().expect("Wayland niezaładowany")
}

fn iface(address: usize) -> *const WlInterface {
    address as *const WlInterface
}

unsafe fn marshal(proxy: Proxy, opcode: u32, interface: *const WlInterface, flags: u32, args: &mut [WlArgument]) -> Proxy {
    unsafe {
        let version = (f().proxy_get_version)(proxy);
        (f().proxy_marshal_array_flags)(proxy, opcode, interface, version, flags, args.as_mut_ptr())
    }
}

fn arg_u(u: u32) -> WlArgument {
    WlArgument { u }
}
fn arg_i(i: i32) -> WlArgument {
    WlArgument { i }
}
fn arg_o(o: *mut c_void) -> WlArgument {
    WlArgument { o }
}
fn arg_s(s: &CStr) -> WlArgument {
    WlArgument { s: s.as_ptr() }
}

pub fn build_xdg(wl_surface: *const WlInterface, wl_seat: *const WlInterface, wl_output: *const WlInterface) -> [usize; 5] {
    let empty = WlInterface {
        name: std::ptr::null(),
        version: 3,
        method_count: 0,
        methods: std::ptr::null(),
        event_count: 0,
        events: std::ptr::null(),
    };
    let all: &'static mut [WlInterface; 5] = Box::leak(Box::new([
        WlInterface { name: c"xdg_wm_base".as_ptr(), ..empty },
        WlInterface { name: c"xdg_positioner".as_ptr(), ..empty },
        WlInterface { name: c"xdg_surface".as_ptr(), ..empty },
        WlInterface { name: c"xdg_toplevel".as_ptr(), ..empty },
        WlInterface { name: c"xdg_popup".as_ptr(), ..empty },
    ]));
    let wm_base = &all[0] as *const WlInterface;
    let positioner = &all[1] as *const WlInterface;
    let surface = &all[2] as *const WlInterface;
    let toplevel = &all[3] as *const WlInterface;
    let popup = &all[4] as *const WlInterface;
    let n: *const WlInterface = std::ptr::null();

    fn types(list: &[*const WlInterface]) -> *const *const WlInterface {
        Box::leak(list.to_vec().into_boxed_slice()).as_ptr()
    }
    fn messages(list: Vec<(&'static CStr, &'static CStr, *const *const WlInterface)>) -> (*const WlMessage, c_int) {
        let count = list.len() as c_int;
        let built: Vec<WlMessage> =
            list.into_iter().map(|(name, signature, types)| WlMessage { name: name.as_ptr(), signature: signature.as_ptr(), types }).collect();
        (Box::leak(built.into_boxed_slice()).as_ptr(), count)
    }
    let none = types(&[n, n, n, n]);

    let (m, mc) = messages(vec![
        (c"destroy", c"", none),
        (c"create_positioner", c"n", types(&[positioner])),
        (c"get_xdg_surface", c"no", types(&[surface, wl_surface])),
        (c"pong", c"u", none),
    ]);
    let (e, ec) = messages(vec![(c"ping", c"u", none)]);
    all[0].methods = m;
    all[0].method_count = mc;
    all[0].events = e;
    all[0].event_count = ec;

    let (m, mc) = messages(vec![
        (c"destroy", c"", none),
        (c"set_size", c"ii", none),
        (c"set_anchor_rect", c"iiii", none),
        (c"set_anchor", c"u", none),
        (c"set_gravity", c"u", none),
        (c"set_constraint_adjustment", c"u", none),
        (c"set_offset", c"ii", none),
        (c"set_reactive", c"3", none),
        (c"set_parent_size", c"3ii", none),
        (c"set_parent_configure", c"3u", none),
    ]);
    all[1].methods = m;
    all[1].method_count = mc;

    let (m, mc) = messages(vec![
        (c"destroy", c"", none),
        (c"get_toplevel", c"n", types(&[toplevel])),
        (c"get_popup", c"n?oo", types(&[popup, surface, positioner])),
        (c"set_window_geometry", c"iiii", none),
        (c"ack_configure", c"u", none),
    ]);
    let (e, ec) = messages(vec![(c"configure", c"u", none)]);
    all[2].methods = m;
    all[2].method_count = mc;
    all[2].events = e;
    all[2].event_count = ec;

    let (m, mc) = messages(vec![
        (c"destroy", c"", none),
        (c"set_parent", c"?o", types(&[toplevel])),
        (c"set_title", c"s", none),
        (c"set_app_id", c"s", none),
        (c"show_window_menu", c"ouii", types(&[wl_seat, n, n, n])),
        (c"move", c"ou", types(&[wl_seat, n])),
        (c"resize", c"ouu", types(&[wl_seat, n, n])),
        (c"set_max_size", c"ii", none),
        (c"set_min_size", c"ii", none),
        (c"set_maximized", c"", none),
        (c"unset_maximized", c"", none),
        (c"set_fullscreen", c"?o", types(&[wl_output])),
        (c"unset_fullscreen", c"", none),
        (c"set_minimized", c"", none),
    ]);
    let (e, ec) = messages(vec![(c"configure", c"iia", none), (c"close", c"", none)]);
    all[3].methods = m;
    all[3].method_count = mc;
    all[3].events = e;
    all[3].event_count = ec;

    let (m, mc) = messages(vec![(c"destroy", c"", none), (c"grab", c"ou", types(&[wl_seat, n])), (c"reposition", c"3ou", types(&[positioner, n]))]);
    let (e, ec) = messages(vec![(c"configure", c"iiii", none), (c"popup_done", c"", none), (c"repositioned", c"3u", none)]);
    all[4].methods = m;
    all[4].method_count = mc;
    all[4].events = e;
    all[4].event_count = ec;

    [wm_base as usize, positioner as usize, surface as usize, toplevel as usize, popup as usize]
}

#[derive(Default)]
struct Globals {
    list: Vec<(u32, String, u32)>,
}

unsafe extern "C" fn registry_global(data: *mut c_void, _registry: Proxy, name: u32, interface: *const c_char, version: u32) {
    let globals = unsafe { &mut *(data as *mut Globals) };
    let interface = unsafe { CStr::from_ptr(interface) }.to_string_lossy().into_owned();
    globals.list.push((name, interface, version));
}

unsafe extern "C" fn registry_global_remove(_data: *mut c_void, _registry: Proxy, _name: u32) {}

unsafe extern "C" fn wm_base_ping(_data: *mut c_void, wm_base: Proxy, serial: u32) {
    unsafe {
        marshal(wm_base, 3, std::ptr::null(), 0, &mut [arg_u(serial)]);
    }
}

#[derive(Default)]
pub struct WindowEvents {
    pending_size: Option<(i32, i32)>,
    configure_serial: Option<u32>,
    pub closed: bool,
    frame_callback: Option<Proxy>,
}

unsafe extern "C" fn frame_done(data: *mut c_void, callback: Proxy, _time: u32) {
    let events = unsafe { &mut *(data as *mut WindowEvents) };
    if events.frame_callback == Some(callback) {
        events.frame_callback = None;
    }
    unsafe { (f().proxy_destroy)(callback) };
}

unsafe extern "C" fn xdg_surface_configure(data: *mut c_void, _surface: Proxy, serial: u32) {
    let events = unsafe { &mut *(data as *mut WindowEvents) };
    events.configure_serial = Some(serial);
}

unsafe extern "C" fn toplevel_configure(data: *mut c_void, _toplevel: Proxy, width: i32, height: i32, _states: *mut c_void) {
    let events = unsafe { &mut *(data as *mut WindowEvents) };
    events.pending_size = Some((width, height));
}

unsafe extern "C" fn toplevel_close(data: *mut c_void, _toplevel: Proxy) {
    let events = unsafe { &mut *(data as *mut WindowEvents) };
    events.closed = true;
}

unsafe extern "C" fn toplevel_ignore(_data: *mut c_void, _toplevel: Proxy, _a: i32, _b: i32) {}

struct Listener(&'static [*const c_void]);
unsafe impl Sync for Listener {}

static REGISTRY_LISTENER: Listener = Listener(&[registry_global as *const c_void, registry_global_remove as *const c_void]);
static FRAME_LISTENER: Listener = Listener(&[frame_done as *const c_void]);
static WM_BASE_LISTENER: Listener = Listener(&[wm_base_ping as *const c_void]);
static XDG_SURFACE_LISTENER: Listener = Listener(&[xdg_surface_configure as *const c_void]);
static TOPLEVEL_LISTENER: Listener =
    Listener(&[toplevel_configure as *const c_void, toplevel_close as *const c_void, toplevel_ignore as *const c_void, toplevel_ignore as *const c_void]);

pub struct WlWindow {
    surface: Proxy,
    xdg_surface: Proxy,
    toplevel: Proxy,
    pub egl_window: *mut c_void,
    events: Box<WindowEvents>,
    pub size: (i32, i32),
    pub configured: bool,
    buffer_scale: i32,
    fallback_size: (i32, i32),
}

impl WlWindow {
    pub fn take_configure(&mut self) -> Option<(i32, i32)> {
        let serial = self.events.configure_serial.take()?;
        let mut changed = None;
        if let Some((w, h)) = self.events.pending_size.take() {
            let size = if w > 0 && h > 0 { (w, h) } else { self.fallback_size };
            if size != self.size || !self.configured {
                self.size = size;
                changed = Some(size);
                unsafe {
                    (f().egl_window_resize)(self.egl_window, size.0 * self.buffer_scale, size.1 * self.buffer_scale, 0, 0);
                }
            }
        }
        unsafe {
            marshal(self.xdg_surface, 4, std::ptr::null(), 0, &mut [arg_u(serial)]);
        }
        if !self.configured {
            changed = Some(self.size);
        }
        self.configured = true;
        changed
    }

    pub fn closed(&self) -> bool {
        self.events.closed
    }

    pub fn set_fallback_size(&mut self, size: (i32, i32)) {
        self.fallback_size = size;
    }

    pub fn set_buffer_scale(&mut self, scale: i32) {
        if scale == self.buffer_scale {
            return;
        }
        self.buffer_scale = scale;
        unsafe {
            marshal(self.surface, 8, std::ptr::null(), 0, &mut [arg_i(scale)]);
            (f().egl_window_resize)(self.egl_window, self.size.0 * scale, self.size.1 * scale, 0, 0);
        }
    }

    pub fn buffer_size(&self) -> (i32, i32) {
        (self.size.0 * self.buffer_scale, self.size.1 * self.buffer_scale)
    }

    pub fn buffer_scale(&self) -> i32 {
        self.buffer_scale
    }

    pub fn request_frame(&mut self) {
        if self.events.frame_callback.is_some() {
            return;
        }
        let i = INTERFACES.get().unwrap();
        unsafe {
            let callback = marshal(self.surface, 3, iface(i.callback), 0, &mut [arg_o(std::ptr::null_mut())]);
            if callback.is_null() {
                return;
            }
            let data = (&mut *self.events as *mut WindowEvents).cast::<c_void>();
            (f().proxy_add_listener)(callback, FRAME_LISTENER.0.as_ptr(), data);
            self.events.frame_callback = Some(callback);
        }
    }

    pub fn frame_pending(&self) -> bool {
        self.events.frame_callback.is_some()
    }
}

pub struct Wayland {
    display: Proxy,
    compositor: Proxy,
    wm_base: Proxy,
    _globals: Box<Globals>,
    _libs: (Library, Library),
}

impl Wayland {
    pub fn connect() -> Result<Wayland, String> {
        let client = Library::open(&["libwayland-client.so.0", "libwayland-client.so"])?;
        let egl = Library::open(&["libwayland-egl.so.1", "libwayland-egl.so"])?;
        let functions = load!(&client, PartialClient {
                display_connect: "wl_display_connect",
                display_get_fd: "wl_display_get_fd",
                display_dispatch_pending: "wl_display_dispatch_pending",
                display_roundtrip: "wl_display_roundtrip",
                display_flush: "wl_display_flush",
                display_prepare_read: "wl_display_prepare_read",
                display_read_events: "wl_display_read_events",
                display_cancel_read: "wl_display_cancel_read",
                display_get_error: "wl_display_get_error",
                proxy_marshal_array_flags: "wl_proxy_marshal_array_flags",
                proxy_add_listener: "wl_proxy_add_listener",
                proxy_get_version: "wl_proxy_get_version",
                proxy_destroy: "wl_proxy_destroy",
            })?
            .with(load!(&egl, PartialEgl {
                egl_window_create: "wl_egl_window_create",
                egl_window_resize: "wl_egl_window_resize",
                egl_window_destroy: "wl_egl_window_destroy",
            })?);
        let data = |name: &str| client.symbol(name).map(|p| p as usize);
        let [wm_base, _positioner, xdg_surface, toplevel, _popup] =
            build_xdg(iface(data("wl_surface_interface")?), iface(data("wl_seat_interface")?), iface(data("wl_output_interface")?));
        let interfaces = Interfaces {
            registry: data("wl_registry_interface")?,
            compositor: data("wl_compositor_interface")?,
            surface: data("wl_surface_interface")?,
            region: data("wl_region_interface")?,
            callback: data("wl_callback_interface")?,
            wm_base,
            xdg_surface,
            toplevel,
        };
        let _ = FUNCTIONS.set(functions);
        let _ = INTERFACES.set(interfaces);
        let i = INTERFACES.get().unwrap();

        unsafe {
            let display = (f().display_connect)(std::ptr::null());
            if display.is_null() {
                return Err("nie można połączyć się z Waylandem (czy to sesja GNOME na Waylandzie?)".into());
            }
            let registry = marshal(display, 1, iface(i.registry), 0, &mut [arg_o(std::ptr::null_mut())]);
            let mut globals = Box::new(Globals::default());
            (f().proxy_add_listener)(registry, REGISTRY_LISTENER.0.as_ptr(), (&mut *globals as *mut Globals).cast());
            (f().display_roundtrip)(display);

            let find = |name: &str| globals.list.iter().find(|(_, interface, _)| interface == name).map(|&(id, _, version)| (id, version));
            let bind = |name: &str, interface: usize, max: u32| -> Result<Proxy, String> {
                let (id, version) = find(name).ok_or_else(|| format!("kompozytor nie oferuje {name}"))?;
                let version = version.min(max);
                let interface_name = CStr::from_ptr((*iface(interface)).name);
                let mut args = [arg_u(id), arg_s(interface_name), arg_u(version), arg_o(std::ptr::null_mut())];
                let proxy = (f().proxy_marshal_array_flags)(registry, 0, iface(interface), version, 0, args.as_mut_ptr());
                if proxy.is_null() { Err(format!("nie można dołączyć {name}")) } else { Ok(proxy) }
            };
            let compositor = bind("wl_compositor", i.compositor, 4)?;
            let wm_base = bind("xdg_wm_base", i.wm_base, 3)?;
            (f().proxy_add_listener)(wm_base, WM_BASE_LISTENER.0.as_ptr(), std::ptr::null_mut());
            (f().display_roundtrip)(display);
            Ok(Wayland { display, compositor, wm_base, _globals: globals, _libs: (client, egl) })
        }
    }

    pub fn display(&self) -> *mut c_void {
        self.display
    }

    pub fn fd(&self) -> c_int {
        unsafe { (f().display_get_fd)(self.display) }
    }

    pub fn create_window(&self, title: &str, app_id: &str, size: (i32, i32)) -> WlWindow {
        let i = INTERFACES.get().unwrap();
        unsafe {
            let null = std::ptr::null_mut();
            let surface = marshal(self.compositor, 0, iface(i.surface), 0, &mut [arg_o(null)]);
            let region = marshal(self.compositor, 1, iface(i.region), 0, &mut [arg_o(null)]);
            marshal(surface, 5, std::ptr::null(), 0, &mut [arg_o(region)]);
            marshal(region, 0, std::ptr::null(), MARSHAL_DESTROY, &mut []);

            let mut events = Box::new(WindowEvents::default());
            let data = (&mut *events as *mut WindowEvents).cast::<c_void>();
            let xdg_surface = marshal(self.wm_base, 2, iface(i.xdg_surface), 0, &mut [arg_o(null), arg_o(surface)]);
            (f().proxy_add_listener)(xdg_surface, XDG_SURFACE_LISTENER.0.as_ptr(), data);
            let toplevel = marshal(xdg_surface, 1, iface(i.toplevel), 0, &mut [arg_o(null)]);
            (f().proxy_add_listener)(toplevel, TOPLEVEL_LISTENER.0.as_ptr(), data);
            let title = CString::new(title).unwrap();
            let app_id = CString::new(app_id).unwrap();
            marshal(toplevel, 2, std::ptr::null(), 0, &mut [arg_s(&title)]);
            marshal(toplevel, 3, std::ptr::null(), 0, &mut [arg_s(&app_id)]);
            marshal(surface, 6, std::ptr::null(), 0, &mut []);
            let egl_window = (f().egl_window_create)(surface, size.0.max(1), size.1.max(1));
            WlWindow { surface, xdg_surface, toplevel, egl_window, events, size, configured: false, buffer_scale: 1, fallback_size: size }
        }
    }

    pub fn destroy_window(&self, window: WlWindow) {
        unsafe {
            if let Some(callback) = window.events.frame_callback {
                (f().proxy_destroy)(callback);
            }
            (f().egl_window_destroy)(window.egl_window);
            marshal(window.toplevel, 0, std::ptr::null(), MARSHAL_DESTROY, &mut []);
            marshal(window.xdg_surface, 0, std::ptr::null(), MARSHAL_DESTROY, &mut []);
            marshal(window.surface, 0, std::ptr::null(), MARSHAL_DESTROY, &mut []);
        }
        drop(window.events);
    }

    pub fn prepare_read(&self) -> Result<(), String> {
        unsafe {
            while (f().display_prepare_read)(self.display) != 0 {
                if (f().display_dispatch_pending)(self.display) < 0 {
                    return Err(self.error());
                }
            }
            (f().display_flush)(self.display);
        }
        Ok(())
    }

    pub fn finish_read(&self, readable: bool) -> Result<(), String> {
        unsafe {
            if readable {
                if (f().display_read_events)(self.display) < 0 {
                    return Err(self.error());
                }
            } else {
                (f().display_cancel_read)(self.display);
            }
            if (f().display_dispatch_pending)(self.display) < 0 {
                return Err(self.error());
            }
        }
        Ok(())
    }

    pub fn flush(&self) {
        unsafe {
            (f().display_flush)(self.display);
        }
    }

    fn error(&self) -> String {
        let code = unsafe { (f().display_get_error)(self.display) };
        format!("połączenie z Waylandem zerwane (błąd {code})")
    }
}

struct PartialClient {
    display_connect: unsafe extern "C" fn(*const c_char) -> Proxy,
    display_get_fd: unsafe extern "C" fn(Proxy) -> c_int,
    display_dispatch_pending: unsafe extern "C" fn(Proxy) -> c_int,
    display_roundtrip: unsafe extern "C" fn(Proxy) -> c_int,
    display_flush: unsafe extern "C" fn(Proxy) -> c_int,
    display_prepare_read: unsafe extern "C" fn(Proxy) -> c_int,
    display_read_events: unsafe extern "C" fn(Proxy) -> c_int,
    display_cancel_read: unsafe extern "C" fn(Proxy),
    display_get_error: unsafe extern "C" fn(Proxy) -> c_int,
    proxy_marshal_array_flags: unsafe extern "C" fn(Proxy, u32, *const WlInterface, u32, u32, *mut WlArgument) -> Proxy,
    proxy_add_listener: unsafe extern "C" fn(Proxy, *const *const c_void, *mut c_void) -> c_int,
    proxy_get_version: unsafe extern "C" fn(Proxy) -> u32,
    proxy_destroy: unsafe extern "C" fn(Proxy),
}

struct PartialEgl {
    egl_window_create: unsafe extern "C" fn(Proxy, c_int, c_int) -> *mut c_void,
    egl_window_resize: unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, c_int),
    egl_window_destroy: unsafe extern "C" fn(*mut c_void),
}

impl PartialClient {
    fn with(self, egl: PartialEgl) -> Functions {
        Functions {
            display_connect: self.display_connect,
            display_get_fd: self.display_get_fd,
            display_dispatch_pending: self.display_dispatch_pending,
            display_roundtrip: self.display_roundtrip,
            display_flush: self.display_flush,
            display_prepare_read: self.display_prepare_read,
            display_read_events: self.display_read_events,
            display_cancel_read: self.display_cancel_read,
            display_get_error: self.display_get_error,
            proxy_marshal_array_flags: self.proxy_marshal_array_flags,
            proxy_add_listener: self.proxy_add_listener,
            proxy_get_version: self.proxy_get_version,
            proxy_destroy: self.proxy_destroy,
            egl_window_create: egl.egl_window_create,
            egl_window_resize: egl.egl_window_resize,
            egl_window_destroy: egl.egl_window_destroy,
        }
    }
}
