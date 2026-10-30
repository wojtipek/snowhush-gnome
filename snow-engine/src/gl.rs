#![allow(non_snake_case, dead_code)]

use std::ffi::{CString, c_char, c_void};

use crate::ffi::Library;

pub type GLenum = u32;
pub type GLuint = u32;
pub type GLint = i32;
pub type GLsizei = i32;
pub type GLfloat = f32;
pub type GLboolean = u8;
pub type GLbitfield = u32;
pub type GLsizeiptr = isize;
pub type GLintptr = isize;

pub const COLOR_BUFFER_BIT: GLbitfield = 0x4000;
pub const BLEND: GLenum = 0x0BE2;
pub const ONE: GLenum = 1;
pub const ONE_MINUS_SRC_ALPHA: GLenum = 0x0303;
pub const TRIANGLE_STRIP: GLenum = 0x0005;
pub const FLOAT: GLenum = 0x1406;
pub const UNSIGNED_BYTE: GLenum = 0x1401;
pub const ARRAY_BUFFER: GLenum = 0x8892;
pub const DYNAMIC_DRAW: GLenum = 0x88E8;
pub const VERTEX_SHADER: GLenum = 0x8B31;
pub const FRAGMENT_SHADER: GLenum = 0x8B30;
pub const COMPILE_STATUS: GLenum = 0x8B81;
pub const LINK_STATUS: GLenum = 0x8B82;
pub const INFO_LOG_LENGTH: GLenum = 0x8B84;
pub const TEXTURE_2D: GLenum = 0x0DE1;
pub const TEXTURE0: GLenum = 0x84C0;
pub const TEXTURE_MIN_FILTER: GLenum = 0x2801;
pub const TEXTURE_MAG_FILTER: GLenum = 0x2800;
pub const TEXTURE_WRAP_S: GLenum = 0x2802;
pub const TEXTURE_WRAP_T: GLenum = 0x2803;
pub const NEAREST: GLint = 0x2600;
pub const LINEAR: GLint = 0x2601;
pub const LINEAR_MIPMAP_LINEAR: GLint = 0x2703;
pub const TEXTURE1: GLenum = 0x84C1;
pub const CLAMP_TO_EDGE: GLint = 0x812F;
pub const R32F: GLint = 0x822E;
pub const RED: GLenum = 0x1903;
pub const RGBA: GLenum = 0x1908;
pub const RGBA8: GLenum = 0x8058;
pub const FRAMEBUFFER: GLenum = 0x8D40;
pub const RENDERBUFFER: GLenum = 0x8D41;
pub const COLOR_ATTACHMENT0: GLenum = 0x8CE0;
pub const FRAMEBUFFER_COMPLETE: GLenum = 0x8CD5;
pub const UNPACK_ALIGNMENT: GLenum = 0x0CF5;
pub const PACK_ALIGNMENT: GLenum = 0x0D05;
pub const RENDERER: GLenum = 0x1F01;
pub const VERSION: GLenum = 0x1F02;
pub const NO_ERROR: GLenum = 0;

macro_rules! gl_functions {
    ($($name:ident : fn($($arg:ty),*) $(-> $ret:ty)?;)*) => {
        pub struct Gl {
            $(pub $name: unsafe extern "C" fn($($arg),*) $(-> $ret)?,)*
        }

        impl Gl {
            pub fn load(lookup: &dyn Fn(&str) -> *mut c_void) -> Result<Gl, String> {
                Ok(Gl {
                    $($name: {
                        let symbol = concat!("gl", stringify!($name));
                        let pointer = lookup(symbol);
                        if pointer.is_null() {
                            return Err(format!("brak funkcji OpenGL ES {symbol}"));
                        }
                        unsafe { std::mem::transmute::<*mut c_void, unsafe extern "C" fn($($arg),*) $(-> $ret)?>(pointer) }
                    },)*
                })
            }
        }
    };
}

gl_functions! {
    GetString: fn(GLenum) -> *const u8;
    GetError: fn() -> GLenum;
    Viewport: fn(GLint, GLint, GLsizei, GLsizei);
    ClearColor: fn(GLfloat, GLfloat, GLfloat, GLfloat);
    Clear: fn(GLbitfield);
    Enable: fn(GLenum);
    Disable: fn(GLenum);
    BlendFunc: fn(GLenum, GLenum);
    CreateShader: fn(GLenum) -> GLuint;
    ShaderSource: fn(GLuint, GLsizei, *const *const c_char, *const GLint);
    CompileShader: fn(GLuint);
    GetShaderiv: fn(GLuint, GLenum, *mut GLint);
    GetShaderInfoLog: fn(GLuint, GLsizei, *mut GLsizei, *mut c_char);
    DeleteShader: fn(GLuint);
    CreateProgram: fn() -> GLuint;
    AttachShader: fn(GLuint, GLuint);
    LinkProgram: fn(GLuint);
    GetProgramiv: fn(GLuint, GLenum, *mut GLint);
    GetProgramInfoLog: fn(GLuint, GLsizei, *mut GLsizei, *mut c_char);
    UseProgram: fn(GLuint);
    GetUniformLocation: fn(GLuint, *const c_char) -> GLint;
    Uniform1i: fn(GLint, GLint);
    Uniform1f: fn(GLint, GLfloat);
    Uniform2f: fn(GLint, GLfloat, GLfloat);
    Uniform4f: fn(GLint, GLfloat, GLfloat, GLfloat, GLfloat);
    Uniform4fv: fn(GLint, GLsizei, *const GLfloat);
    GenBuffers: fn(GLsizei, *mut GLuint);
    BindBuffer: fn(GLenum, GLuint);
    BufferData: fn(GLenum, GLsizeiptr, *const c_void, GLenum);
    BufferSubData: fn(GLenum, GLintptr, GLsizeiptr, *const c_void);
    GenVertexArrays: fn(GLsizei, *mut GLuint);
    BindVertexArray: fn(GLuint);
    EnableVertexAttribArray: fn(GLuint);
    VertexAttribPointer: fn(GLuint, GLint, GLenum, GLboolean, GLsizei, *const c_void);
    VertexAttribDivisor: fn(GLuint, GLuint);
    DrawArrays: fn(GLenum, GLint, GLsizei);
    DrawArraysInstanced: fn(GLenum, GLint, GLsizei, GLsizei);
    GenTextures: fn(GLsizei, *mut GLuint);
    BindTexture: fn(GLenum, GLuint);
    ActiveTexture: fn(GLenum);
    TexParameteri: fn(GLenum, GLenum, GLint);
    GenerateMipmap: fn(GLenum);
    Uniform2fv: fn(GLint, GLsizei, *const GLfloat);
    TexImage2D: fn(GLenum, GLint, GLint, GLsizei, GLsizei, GLint, GLenum, GLenum, *const c_void);
    TexSubImage2D: fn(GLenum, GLint, GLint, GLint, GLsizei, GLsizei, GLenum, GLenum, *const c_void);
    PixelStorei: fn(GLenum, GLint);
    GenFramebuffers: fn(GLsizei, *mut GLuint);
    BindFramebuffer: fn(GLenum, GLuint);
    GenRenderbuffers: fn(GLsizei, *mut GLuint);
    BindRenderbuffer: fn(GLenum, GLuint);
    RenderbufferStorage: fn(GLenum, GLenum, GLsizei, GLsizei);
    FramebufferRenderbuffer: fn(GLenum, GLenum, GLenum, GLuint);
    CheckFramebufferStatus: fn(GLenum) -> GLenum;
    ReadPixels: fn(GLint, GLint, GLsizei, GLsizei, GLenum, GLenum, *mut c_void);
    Finish: fn();
}

pub fn lookup_with(get_proc: impl Fn(&CString) -> *mut c_void, fallback: Option<Library>) -> impl Fn(&str) -> *mut c_void {
    move |name: &str| {
        let c = CString::new(name).unwrap();
        let pointer = get_proc(&c);
        if !pointer.is_null() {
            return pointer;
        }
        fallback.as_ref().and_then(|lib| lib.symbol(name).ok()).unwrap_or(std::ptr::null_mut())
    }
}
