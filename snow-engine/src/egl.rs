#![allow(non_snake_case, dead_code)]

use std::ffi::{CString, c_char, c_void};

use crate::ffi::Library;
use crate::load;

pub type EGLDisplay = *mut c_void;
pub type EGLConfig = *mut c_void;
pub type EGLContext = *mut c_void;
pub type EGLSurface = *mut c_void;
type EGLint = i32;
type EGLBoolean = u32;

const EGL_NONE: EGLint = 0x3038;
const EGL_RED_SIZE: EGLint = 0x3024;
const EGL_GREEN_SIZE: EGLint = 0x3023;
const EGL_BLUE_SIZE: EGLint = 0x3022;
const EGL_ALPHA_SIZE: EGLint = 0x3021;
const EGL_SURFACE_TYPE: EGLint = 0x3033;
const EGL_WINDOW_BIT: EGLint = 0x0004;
const EGL_PBUFFER_BIT: EGLint = 0x0001;
const EGL_RENDERABLE_TYPE: EGLint = 0x3040;
const EGL_OPENGL_ES3_BIT: EGLint = 0x0040;
const EGL_CONTEXT_MAJOR_VERSION: EGLint = 0x3098;
const EGL_OPENGL_ES_API: u32 = 0x30A0;
const EGL_PLATFORM_WAYLAND_KHR: u32 = 0x31D8;
const EGL_PLATFORM_SURFACELESS_MESA: u32 = 0x31DD;
const EGL_VENDOR: EGLint = 0x3053;
const EGL_VERSION: EGLint = 0x3054;
const EGL_WIDTH: EGLint = 0x3057;
const EGL_EXTENSIONS: EGLint = 0x3055;
type SwapWithDamage = unsafe extern "C" fn(EGLDisplay, EGLSurface, *const EGLint, EGLint) -> EGLBoolean;
const EGL_HEIGHT: EGLint = 0x3056;

struct Functions {
    GetProcAddress: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    GetDisplay: unsafe extern "C" fn(*mut c_void) -> EGLDisplay,
    Initialize: unsafe extern "C" fn(EGLDisplay, *mut EGLint, *mut EGLint) -> EGLBoolean,
    BindAPI: unsafe extern "C" fn(u32) -> EGLBoolean,
    ChooseConfig: unsafe extern "C" fn(EGLDisplay, *const EGLint, *mut EGLConfig, EGLint, *mut EGLint) -> EGLBoolean,
    CreateContext: unsafe extern "C" fn(EGLDisplay, EGLConfig, EGLContext, *const EGLint) -> EGLContext,
    CreateWindowSurface: unsafe extern "C" fn(EGLDisplay, EGLConfig, *mut c_void, *const EGLint) -> EGLSurface,
    CreatePbufferSurface: unsafe extern "C" fn(EGLDisplay, EGLConfig, *const EGLint) -> EGLSurface,
    DestroySurface: unsafe extern "C" fn(EGLDisplay, EGLSurface) -> EGLBoolean,
    MakeCurrent: unsafe extern "C" fn(EGLDisplay, EGLSurface, EGLSurface, EGLContext) -> EGLBoolean,
    SwapBuffers: unsafe extern "C" fn(EGLDisplay, EGLSurface) -> EGLBoolean,
    SwapInterval: unsafe extern "C" fn(EGLDisplay, EGLint) -> EGLBoolean,
    GetError: unsafe extern "C" fn() -> EGLint,
    QueryString: unsafe extern "C" fn(EGLDisplay, EGLint) -> *const c_char,
}

pub struct Egl {
    f: Functions,
    pub display: EGLDisplay,
    config: EGLConfig,
    pub context: EGLContext,
    swap_with_damage: Option<SwapWithDamage>,
    _lib: Library,
}

pub enum Platform {
    Wayland(*mut c_void),
    Surfaceless,
}

impl Egl {
    pub fn new(platform: Platform) -> Result<Egl, String> {
        let lib = Library::open(&["libEGL.so.1", "libEGL.so"])?;
        let f = load!(&lib, Functions {
            GetProcAddress: "eglGetProcAddress",
            GetDisplay: "eglGetDisplay",
            Initialize: "eglInitialize",
            BindAPI: "eglBindAPI",
            ChooseConfig: "eglChooseConfig",
            CreateContext: "eglCreateContext",
            CreateWindowSurface: "eglCreateWindowSurface",
            CreatePbufferSurface: "eglCreatePbufferSurface",
            DestroySurface: "eglDestroySurface",
            MakeCurrent: "eglMakeCurrent",
            SwapBuffers: "eglSwapBuffers",
            SwapInterval: "eglSwapInterval",
            GetError: "eglGetError",
            QueryString: "eglQueryString",
        })?;
        unsafe {
            type GetPlatformDisplay = unsafe extern "C" fn(u32, *mut c_void, *const isize) -> EGLDisplay;
            type GetPlatformDisplayExt = unsafe extern "C" fn(u32, *mut c_void, *const EGLint) -> EGLDisplay;
            let (kind, native) = match platform {
                Platform::Wayland(display) => (EGL_PLATFORM_WAYLAND_KHR, display),
                Platform::Surfaceless => (EGL_PLATFORM_SURFACELESS_MESA, std::ptr::null_mut()),
            };
            let mut display: EGLDisplay = std::ptr::null_mut();
            let core = (f.GetProcAddress)(c"eglGetPlatformDisplay".as_ptr());
            if !core.is_null() {
                let get: GetPlatformDisplay = std::mem::transmute(core);
                display = get(kind, native, std::ptr::null());
            }
            if display.is_null() {
                let ext = (f.GetProcAddress)(c"eglGetPlatformDisplayEXT".as_ptr());
                if !ext.is_null() {
                    let get: GetPlatformDisplayExt = std::mem::transmute(ext);
                    display = get(kind, native, std::ptr::null());
                }
            }
            if display.is_null() && kind == EGL_PLATFORM_WAYLAND_KHR {
                display = (f.GetDisplay)(native);
            }
            if display.is_null() {
                return Err(format!("EGL: brak ekranu (błąd 0x{:x})", (f.GetError)()));
            }
            let (mut major, mut minor) = (0, 0);
            if (f.Initialize)(display, &mut major, &mut minor) == 0 {
                return Err(format!("EGL: inicjalizacja nieudana (0x{:x})", (f.GetError)()));
            }
            if (f.BindAPI)(EGL_OPENGL_ES_API) == 0 {
                return Err("EGL: brak OpenGL ES".into());
            }
            let surface_type = if kind == EGL_PLATFORM_WAYLAND_KHR { EGL_WINDOW_BIT } else { 0 };
            let attributes = [
                EGL_SURFACE_TYPE,
                surface_type,
                EGL_RENDERABLE_TYPE,
                EGL_OPENGL_ES3_BIT,
                EGL_RED_SIZE,
                8,
                EGL_GREEN_SIZE,
                8,
                EGL_BLUE_SIZE,
                8,
                EGL_ALPHA_SIZE,
                8,
                EGL_NONE,
            ];
            let mut config: EGLConfig = std::ptr::null_mut();
            let mut count = 0;
            if (f.ChooseConfig)(display, attributes.as_ptr(), &mut config, 1, &mut count) == 0 || count == 0 {
                return Err("EGL: brak konfiguracji z kanałem przezroczystości".into());
            }
            let context_attributes = [EGL_CONTEXT_MAJOR_VERSION, 3, EGL_NONE];
            let context = (f.CreateContext)(display, config, std::ptr::null_mut(), context_attributes.as_ptr());
            if context.is_null() {
                return Err(format!("EGL: nie można utworzyć kontekstu OpenGL ES 3 (0x{:x})", (f.GetError)()));
            }
            let vendor = cstr((f.QueryString)(display, EGL_VENDOR));
            let extensions = cstr((f.QueryString)(display, EGL_EXTENSIONS));
            let mut swap_with_damage = None;
            for (extension, function) in
                [("EGL_KHR_swap_buffers_with_damage", c"eglSwapBuffersWithDamageKHR"), ("EGL_EXT_swap_buffers_with_damage", c"eglSwapBuffersWithDamageEXT")]
            {
                if swap_with_damage.is_none() && extensions.split_whitespace().any(|e| e == extension) {
                    let pointer = (f.GetProcAddress)(function.as_ptr());
                    if !pointer.is_null() {
                        swap_with_damage = Some(std::mem::transmute::<*mut c_void, SwapWithDamage>(pointer));
                    }
                }
            }
            crate::log!("EGL {major}.{minor} ({vendor}), częściowe odświeżanie: {}", if swap_with_damage.is_some() { "tak" } else { "nie" });
            Ok(Egl { f, display, config, context, swap_with_damage, _lib: lib })
        }
    }

    pub fn get_proc(&self, name: &CString) -> *mut c_void {
        unsafe { (self.f.GetProcAddress)(name.as_ptr()) }
    }

    pub fn create_window_surface(&self, native_window: *mut c_void) -> Result<EGLSurface, String> {
        let surface = unsafe { (self.f.CreateWindowSurface)(self.display, self.config, native_window, std::ptr::null()) };
        if surface.is_null() { Err(format!("EGL: nie można utworzyć powierzchni okna (0x{:x})", self.error())) } else { Ok(surface) }
    }

    pub fn destroy_surface(&self, surface: EGLSurface) {
        unsafe {
            (self.f.MakeCurrent)(self.display, std::ptr::null_mut(), std::ptr::null_mut(), self.context);
            (self.f.DestroySurface)(self.display, surface);
        }
    }

    pub fn make_current(&self, surface: EGLSurface) -> Result<(), String> {
        let ok = unsafe { (self.f.MakeCurrent)(self.display, surface, surface, self.context) };
        if ok == 0 { Err(format!("EGL: eglMakeCurrent nieudane (0x{:x})", self.error())) } else { Ok(()) }
    }

    pub fn no_vsync(&self) {
        unsafe {
            (self.f.SwapInterval)(self.display, 0);
        }
    }

    pub fn swap(&self, surface: EGLSurface) -> Result<(), String> {
        let ok = unsafe { (self.f.SwapBuffers)(self.display, surface) };
        if ok == 0 { Err(format!("EGL: eglSwapBuffers nieudane (0x{:x})", self.error())) } else { Ok(()) }
    }

    pub fn swap_damaged(&self, surface: EGLSurface, rects: Option<&[i32]>) -> Result<(), String> {
        let (Some(swap), Some(rects)) = (self.swap_with_damage, rects) else { return self.swap(surface) };
        if rects.is_empty() {
            return Ok(());
        }
        let ok = unsafe { swap(self.display, surface, rects.as_ptr(), (rects.len() / 4) as EGLint) };
        if ok == 0 { Err(format!("EGL: eglSwapBuffersWithDamage nieudane (0x{:x})", self.error())) } else { Ok(()) }
    }

    fn error(&self) -> EGLint {
        unsafe { (self.f.GetError)() }
    }
}

fn cstr(pointer: *const c_char) -> String {
    if pointer.is_null() { String::new() } else { unsafe { std::ffi::CStr::from_ptr(pointer) }.to_string_lossy().into_owned() }
}
