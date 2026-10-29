#![allow(clippy::missing_safety_doc, clippy::deref_addrof)]

#[path = "../../../snow-engine/src/ffi.rs"]
#[allow(dead_code)]
mod ffi;
#[path = "../../../snow-engine/src/wl.rs"]
#[allow(dead_code)]
mod wl;

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use ffi::Library;

type Ptr = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy)]
union Arg {
    i: i32,
    u: u32,
    a: *mut WlArray,
}

#[repr(C)]
struct WlArray {
    size: usize,
    alloc: usize,
    data: *mut c_void,
}

struct S {
    display_create: unsafe extern "C" fn() -> Ptr,
    display_add_socket: unsafe extern "C" fn(Ptr, *const c_char) -> c_int,
    display_init_shm: unsafe extern "C" fn(Ptr) -> c_int,
    display_get_event_loop: unsafe extern "C" fn(Ptr) -> Ptr,
    display_flush_clients: unsafe extern "C" fn(Ptr),
    event_loop_dispatch: unsafe extern "C" fn(Ptr, c_int) -> c_int,
    global_create: unsafe extern "C" fn(Ptr, *const c_void, c_int, Ptr, unsafe extern "C" fn(Ptr, Ptr, u32, u32)) -> Ptr,
    resource_create: unsafe extern "C" fn(Ptr, *const c_void, c_int, u32) -> Ptr,
    resource_set_implementation: unsafe extern "C" fn(Ptr, *const c_void, Ptr, Option<unsafe extern "C" fn(Ptr)>),
    resource_get_user_data: unsafe extern "C" fn(Ptr) -> Ptr,
    resource_post_event_array: unsafe extern "C" fn(Ptr, u32, *mut Arg),
    resource_destroy: unsafe extern "C" fn(Ptr),
    resource_get_version: unsafe extern "C" fn(Ptr) -> c_int,
    shm_buffer_get: unsafe extern "C" fn(Ptr) -> Ptr,
    shm_buffer_get_data: unsafe extern "C" fn(Ptr) -> *mut u8,
    shm_buffer_get_width: unsafe extern "C" fn(Ptr) -> i32,
    shm_buffer_get_height: unsafe extern "C" fn(Ptr) -> i32,
    shm_buffer_get_stride: unsafe extern "C" fn(Ptr) -> i32,
    shm_buffer_begin_access: unsafe extern "C" fn(Ptr),
    shm_buffer_end_access: unsafe extern "C" fn(Ptr),
}

struct Ifaces {
    compositor: usize,
    surface: usize,
    region: usize,
    callback: usize,
    wm_base: usize,
    xdg_surface: usize,
    toplevel: usize,
}

static SV: OnceLock<S> = OnceLock::new();
static IF: OnceLock<Ifaces> = OnceLock::new();
fn s() -> &'static S {
    SV.get().unwrap()
}
fn i() -> &'static Ifaces {
    IF.get().unwrap()
}

#[derive(Default)]
struct Surface {
    id: usize,
    title: String,
    app_id: String,
    input_region_empty: Option<bool>,
    buffer_scale: i32,
    pending_buffer: Option<Ptr>,
    toplevel: Option<Ptr>,
    xdg_surface: Option<Ptr>,
    configured_once: bool,
    acked: u32,
    commits: u32,
    frames_with_buffer: u32,
    last_frame: Option<(i32, i32, Vec<u8>)>,
}

#[derive(Default)]
struct Region {
    rects: u32,
}

static mut SURFACES: Vec<*mut Surface> = Vec::new();
static mut SERIAL: u32 = 1;
static mut NO_FRAME_CALLBACKS: bool = false;

unsafe extern "C" fn noop() {}

unsafe extern "C" fn destroy_request(_c: Ptr, r: Ptr) {
    unsafe { (s().resource_destroy)(r) }
}

fn table(entries: &[*const c_void]) -> *const c_void {
    Box::leak(entries.to_vec().into_boxed_slice()).as_ptr().cast()
}

unsafe extern "C" fn create_surface(c: Ptr, r: Ptr, id: u32) {
    unsafe {
        let res = (s().resource_create)(c, i().surface as _, (s().resource_get_version)(r), id);
        let surface = Box::into_raw(Box::new(Surface { buffer_scale: 1, ..Default::default() }));
        let raw = &raw mut SURFACES;
        (*surface).id = (*raw).len();
        (*raw).push(surface);
        let n = noop as *const c_void;
        let imp = table(&[
            destroy_request as _,
            surface_attach as _,
            n,
            surface_frame as _,
            n,
            surface_set_input_region as _,
            surface_commit as _,
            n,
            surface_set_buffer_scale as _,
            n,
            n,
        ]);
        (s().resource_set_implementation)(res, imp, surface.cast(), None);
    }
}

unsafe extern "C" fn create_region(c: Ptr, r: Ptr, id: u32) {
    unsafe {
        let res = (s().resource_create)(c, i().region as _, (s().resource_get_version)(r), id);
        let region = Box::into_raw(Box::new(Region::default()));
        let imp = table(&[destroy_request as _, region_add as _, noop as _]);
        (s().resource_set_implementation)(res, imp, region.cast(), None);
    }
}

unsafe extern "C" fn region_add(_c: Ptr, r: Ptr, _x: i32, _y: i32, _w: i32, _h: i32) {
    unsafe {
        let region = (s().resource_get_user_data)(r) as *mut Region;
        (*region).rects += 1;
    }
}

fn surface_of(r: Ptr) -> &'static mut Surface {
    unsafe { &mut *((s().resource_get_user_data)(r) as *mut Surface) }
}

unsafe extern "C" fn surface_attach(_c: Ptr, r: Ptr, buffer: Ptr, _x: i32, _y: i32) {
    surface_of(r).pending_buffer = if buffer.is_null() { None } else { Some(buffer) };
}

unsafe extern "C" fn surface_frame(c: Ptr, _r: Ptr, id: u32) {
    unsafe {
        let cb = (s().resource_create)(c, i().callback as _, 1, id);
        if NO_FRAME_CALLBACKS {
            return;
        }
        let mut args = [Arg { u: 0 }];
        (s().resource_post_event_array)(cb, 0, args.as_mut_ptr());
        (s().resource_destroy)(cb);
    }
}

unsafe extern "C" fn surface_set_input_region(_c: Ptr, r: Ptr, region: Ptr) {
    let surface = surface_of(r);
    surface.input_region_empty = Some(if region.is_null() {
        false
    } else {
        unsafe { (*((s().resource_get_user_data)(region) as *mut Region)).rects == 0 }
    });
}

unsafe extern "C" fn surface_set_buffer_scale(_c: Ptr, r: Ptr, scale: i32) {
    surface_of(r).buffer_scale = scale;
}

unsafe extern "C" fn surface_commit(_c: Ptr, r: Ptr) {
    let surface = surface_of(r);
    surface.commits += 1;
    unsafe {
        if let Some(buffer) = surface.pending_buffer.take() {
            let shm = (s().shm_buffer_get)(buffer);
            if !shm.is_null() {
                (s().shm_buffer_begin_access)(shm);
                let (w, h, stride) = ((s().shm_buffer_get_width)(shm), (s().shm_buffer_get_height)(shm), (s().shm_buffer_get_stride)(shm));
                let data = std::slice::from_raw_parts((s().shm_buffer_get_data)(shm), (stride * h) as usize);
                let mut pixels = Vec::with_capacity((w * h * 4) as usize);
                for row in 0..h {
                    pixels.extend_from_slice(&data[(row * stride) as usize..(row * stride + w * 4) as usize]);
                }
                (s().shm_buffer_end_access)(shm);
                surface.last_frame = Some((w, h, pixels));
                surface.frames_with_buffer += 1;
            }
            (s().resource_post_event_array)(buffer, 0, std::ptr::null_mut());
        }
        if !surface.configured_once
            && let (Some(toplevel), Some(xdg)) = (surface.toplevel, surface.xdg_surface)
        {
            surface.configured_once = true;
            send_configure(toplevel, xdg, 0, 0);
        }
    }
}

unsafe fn send_configure(toplevel: Ptr, xdg: Ptr, w: i32, h: i32) {
    unsafe {
        let mut array = WlArray { size: 0, alloc: 0, data: std::ptr::null_mut() };
        let mut args = [Arg { i: w }, Arg { i: h }, Arg { a: &mut array }];
        (s().resource_post_event_array)(toplevel, 0, args.as_mut_ptr());
        let serial = SERIAL;
        SERIAL += 1;
        let mut args = [Arg { u: serial }];
        (s().resource_post_event_array)(xdg, 0, args.as_mut_ptr());
    }
}

unsafe extern "C" fn get_xdg_surface(c: Ptr, r: Ptr, id: u32, surface: Ptr) {
    unsafe {
        let res = (s().resource_create)(c, i().xdg_surface as _, (s().resource_get_version)(r), id);
        let state = surface_of(surface);
        state.xdg_surface = Some(res);
        let n = noop as *const c_void;
        let imp = table(&[destroy_request as _, get_toplevel as _, n, n, ack_configure as _]);
        (s().resource_set_implementation)(res, imp, (state as *mut Surface).cast(), None);
    }
}

unsafe extern "C" fn get_toplevel(c: Ptr, r: Ptr, id: u32) {
    unsafe {
        let res = (s().resource_create)(c, i().toplevel as _, (s().resource_get_version)(r), id);
        let state = surface_of(r);
        state.toplevel = Some(res);
        let n = noop as *const c_void;
        let imp = table(&[destroy_request as _, n, set_title as _, set_app_id as _, n, n, n, n, n, n, n, n, n, n]);
        (s().resource_set_implementation)(res, imp, (state as *mut Surface).cast(), None);
    }
}

unsafe extern "C" fn ack_configure(_c: Ptr, r: Ptr, serial: u32) {
    surface_of(r).acked = serial;
}

unsafe extern "C" fn set_title(_c: Ptr, r: Ptr, title: *const c_char) {
    surface_of(r).title = unsafe { CStr::from_ptr(title) }.to_string_lossy().into_owned();
}

unsafe extern "C" fn set_app_id(_c: Ptr, r: Ptr, app_id: *const c_char) {
    surface_of(r).app_id = unsafe { CStr::from_ptr(app_id) }.to_string_lossy().into_owned();
}

unsafe extern "C" fn bind_compositor(c: Ptr, _d: Ptr, version: u32, id: u32) {
    unsafe {
        let res = (s().resource_create)(c, i().compositor as _, version as c_int, id);
        (s().resource_set_implementation)(res, table(&[create_surface as _, create_region as _]), std::ptr::null_mut(), None);
    }
}

unsafe extern "C" fn bind_wm_base(c: Ptr, _d: Ptr, version: u32, id: u32) {
    unsafe {
        let res = (s().resource_create)(c, i().wm_base as _, version as c_int, id);
        let imp = table(&[destroy_request as _, noop as _, get_xdg_surface as _, noop as _]);
        (s().resource_set_implementation)(res, imp, std::ptr::null_mut(), None);
    }
}

fn save(surface: &Surface, directory: &str) {
    let Some((w, h, pixels)) = &surface.last_frame else { return };
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for y in 0..*h {
        for x in 0..*w {
            let i = ((y * w + x) * 4) as usize;
            let checker = if ((x / 24) + (y / 24)) % 2 == 0 { 0.18 } else { 0.24 };
            let a = pixels[i + 3] as f32 / 255.0;
            for c in [2, 1, 0] {
                let v = pixels[i + c] as f32 / 255.0 + checker * (1.0 - a);
                out.push((v.clamp(0.0, 1.0) * 255.0) as u8);
            }
        }
    }
    let path = format!("{directory}/surface-{}.ppm", surface.id);
    std::fs::write(&path, out).unwrap();
    println!("fake-compositor: zapisano {path} ({w}×{h})");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let socket = args.get(1).cloned().unwrap_or_else(|| "snowtest-0".into());
    let directory = args.get(2).cloned().unwrap_or_else(|| ".".into());
    let resize = args.iter().position(|a| a == "--resize-after").map(|p| {
        let seconds: f32 = args[p + 1].parse().unwrap();
        let (w, h) = args[p + 2].split_once('x').unwrap();
        (Duration::from_secs_f32(seconds), w.parse::<i32>().unwrap(), h.parse::<i32>().unwrap())
    });
    let run_for = args.iter().position(|a| a == "--run-for").map(|p| Duration::from_secs_f32(args[p + 1].parse().unwrap())).unwrap_or(Duration::from_secs(20));

    unsafe {
        NO_FRAME_CALLBACKS = args.iter().any(|a| a == "--no-frame-callbacks");
    }
    let lib = Library::open(&["libwayland-server.so.0"]).unwrap();
    let functions = unsafe {
        S {
            display_create: lib.function("wl_display_create").unwrap(),
            display_add_socket: lib.function("wl_display_add_socket").unwrap(),
            display_init_shm: lib.function("wl_display_init_shm").unwrap(),
            display_get_event_loop: lib.function("wl_display_get_event_loop").unwrap(),
            display_flush_clients: lib.function("wl_display_flush_clients").unwrap(),
            event_loop_dispatch: lib.function("wl_event_loop_dispatch").unwrap(),
            global_create: lib.function("wl_global_create").unwrap(),
            resource_create: lib.function("wl_resource_create").unwrap(),
            resource_set_implementation: lib.function("wl_resource_set_implementation").unwrap(),
            resource_get_user_data: lib.function("wl_resource_get_user_data").unwrap(),
            resource_post_event_array: lib.function("wl_resource_post_event_array").unwrap(),
            resource_destroy: lib.function("wl_resource_destroy").unwrap(),
            resource_get_version: lib.function("wl_resource_get_version").unwrap(),
            shm_buffer_get: lib.function("wl_shm_buffer_get").unwrap(),
            shm_buffer_get_data: lib.function("wl_shm_buffer_get_data").unwrap(),
            shm_buffer_get_width: lib.function("wl_shm_buffer_get_width").unwrap(),
            shm_buffer_get_height: lib.function("wl_shm_buffer_get_height").unwrap(),
            shm_buffer_get_stride: lib.function("wl_shm_buffer_get_stride").unwrap(),
            shm_buffer_begin_access: lib.function("wl_shm_buffer_begin_access").unwrap(),
            shm_buffer_end_access: lib.function("wl_shm_buffer_end_access").unwrap(),
        }
    };
    let data = |name: &str| lib.symbol(name).unwrap() as usize;
    let [wm_base, _, xdg_surface, toplevel, _] = wl::build_xdg(
        data("wl_surface_interface") as *const wl::WlInterface,
        data("wl_seat_interface") as *const wl::WlInterface,
        data("wl_output_interface") as *const wl::WlInterface,
    );
    let _ = SV.set(functions);
    let _ = IF.set(Ifaces {
        compositor: data("wl_compositor_interface"),
        surface: data("wl_surface_interface"),
        region: data("wl_region_interface"),
        callback: data("wl_callback_interface"),
        wm_base,
        xdg_surface,
        toplevel,
    });
    unsafe {
        let display = (s().display_create)();
        let name = CString::new(socket.clone()).unwrap();
        assert_eq!((s().display_add_socket)(display, name.as_ptr()), 0, "gniazdo zajęte?");
        (s().display_init_shm)(display);
        (s().global_create)(display, i().compositor as _, 4, std::ptr::null_mut(), bind_compositor);
        (s().global_create)(display, i().wm_base as _, 3, std::ptr::null_mut(), bind_wm_base);
        println!("fake-compositor: gotowy na {socket}");
        let event_loop = (s().display_get_event_loop)(display);
        let started = Instant::now();
        let mut resized = false;
        while started.elapsed() < run_for {
            (s().event_loop_dispatch)(event_loop, 20);
            (s().display_flush_clients)(display);
            if let Some((after, w, h)) = resize
                && !resized
                && started.elapsed() > after
            {
                resized = true;
                for &surface in (*&raw const SURFACES).iter() {
                    if let (Some(t), Some(x)) = ((*surface).toplevel, (*surface).xdg_surface) {
                        println!("fake-compositor: zmiana rozmiaru okna {} na {w}×{h}", (*surface).id);
                        send_configure(t, x, w, h);
                    }
                }
            }
        }
        for &surface in (*&raw const SURFACES).iter() {
            let surface = &*surface;
            println!(
                "fake-compositor: okno {}: tytuł={:?} app_id={:?} pusty_obszar_klikniec={:?} skala={} zatwierdzeń={} klatek={} potwierdzona_konfiguracja={}",
                surface.id,
                surface.title,
                surface.app_id,
                surface.input_region_empty,
                surface.buffer_scale,
                surface.commits,
                surface.frames_with_buffer,
                surface.acked
            );
            save(surface, &directory);
        }
    }
}
