use std::ffi::{CStr, CString, c_char, c_int, c_long, c_void};

unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;

    pub fn poll(fds: *mut PollFd, nfds: u64, timeout: c_int) -> c_int;
    pub fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    pub fn fcntl(fd: c_int, cmd: c_int, ...) -> c_int;
    pub fn prctl(option: c_int, ...) -> c_int;
    fn time(t: *mut i64) -> i64;
    fn localtime_r(t: *const i64, result: *mut Tm) -> *mut Tm;
}

const RTLD_NOW: c_int = 2;
pub const F_GETFL: c_int = 3;
pub const F_SETFL: c_int = 4;
pub const O_NONBLOCK: c_int = 0o4000;
pub const POLLIN: i16 = 0x001;
pub const POLLERR: i16 = 0x008;
pub const POLLHUP: i16 = 0x010;
pub const PR_SET_PDEATHSIG: c_int = 1;
pub const SIGTERM: c_int = 15;

#[repr(C)]
pub struct PollFd {
    pub fd: c_int,
    pub events: i16,
    pub revents: i16,
}

#[repr(C)]
struct Tm {
    sec: c_int,
    min: c_int,
    hour: c_int,
    mday: c_int,
    mon: c_int,
    year: c_int,
    wday: c_int,
    yday: c_int,
    isdst: c_int,
    gmtoff: c_long,
    zone: *const c_char,
}

pub fn local_hour() -> f32 {
    unsafe {
        let now = time(std::ptr::null_mut());
        let mut tm: Tm = std::mem::zeroed();
        if localtime_r(&now, &mut tm).is_null() {
            return 12.0;
        }
        tm.hour as f32 + tm.min as f32 / 60.0
    }
}

pub struct Library {
    handle: *mut c_void,
    name: String,
}

impl Library {
    pub fn open(names: &[&str]) -> Result<Library, String> {
        let mut errors = Vec::new();
        for name in names {
            let c = CString::new(*name).unwrap();
            let handle = unsafe { dlopen(c.as_ptr(), RTLD_NOW) };
            if !handle.is_null() {
                return Ok(Library { handle, name: name.to_string() });
            }
            errors.push(last_error());
        }
        Err(format!("nie można załadować {}: {}", names.join(" / "), errors.join("; ")))
    }

    pub fn symbol(&self, name: &str) -> Result<*mut c_void, String> {
        let c = CString::new(name).unwrap();
        let pointer = unsafe { dlsym(self.handle, c.as_ptr()) };
        if pointer.is_null() { Err(format!("brak symbolu {name} w {}", self.name)) } else { Ok(pointer) }
    }

    pub unsafe fn function<F: Copy>(&self, name: &str) -> Result<F, String> {
        let pointer = self.symbol(name)?;
        assert_eq!(std::mem::size_of::<F>(), std::mem::size_of::<*mut c_void>());
        Ok(unsafe { std::mem::transmute_copy::<*mut c_void, F>(&pointer) })
    }
}

fn last_error() -> String {
    let error = unsafe { dlerror() };
    if error.is_null() { "nieznany błąd".into() } else { unsafe { CStr::from_ptr(error) }.to_string_lossy().into_owned() }
}

#[macro_export]
#[allow(clippy::macro_metavars_in_unsafe)]
macro_rules! load {
    ($lib:expr, $ty:ident { $($field:ident : $name:literal),* $(,)? }) => {{
        let lib = $lib;
        Ok::<$ty, String>($ty { $($field: unsafe { lib.function($name)? }),* })
    }};
}

pub fn set_nonblocking(fd: c_int) {
    unsafe {
        let flags = fcntl(fd, F_GETFL);
        if flags >= 0 {
            fcntl(fd, F_SETFL, flags | O_NONBLOCK);
        }
    }
}

pub fn die_with_parent() {
    unsafe {
        prctl(PR_SET_PDEATHSIG, SIGTERM as std::ffi::c_ulong);
    }
}
