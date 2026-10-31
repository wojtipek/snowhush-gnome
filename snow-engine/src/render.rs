use std::collections::HashMap;
use std::ffi::{CString, c_void};
use std::mem::size_of;

use snow_core::geometry::V2;
use snow_core::halloween::{self, PUMPKIN_ASPECT, PUMPKIN_CANDLE, SIGN_CAPTIONS};
use snow_core::simulation::{COLUMN_WIDTH, PumpkinSprite, SnowSimulation, Sprite};

use crate::damage::PileShape;
use crate::gl::{self, Gl, GLint, GLuint};
use crate::png;

const HEIGHTS_WIDTH: i32 = 4096;
const MAX_OCCLUDERS: usize = 16;
const MAX_MASKS: usize = 32;
pub const LANTERN_REACH: f32 = 130.0;

const SPRITE_VERTEX: &str = include_str!("shaders/sprite.vert");
const SPRITE_FRAGMENT: &str = include_str!("shaders/sprite.frag");
const PILE_VERTEX: &str = include_str!("shaders/pile.vert");
const PILE_FRAGMENT: &str = include_str!("shaders/pile.frag");
const SIGN_VERTEX: &str = include_str!("shaders/sign.vert");
const SIGN_FRAGMENT: &str = include_str!("shaders/sign.frag");
const PUMPKIN_VERTEX: &str = include_str!("shaders/pumpkin.vert");
const PUMPKIN_FRAGMENT: &str = include_str!("shaders/pumpkin.frag");

const SIGNS: [&[u8]; SIGN_CAPTIONS.len()] = [
    include_bytes!("../assets/signs/sign-00.png"),
    include_bytes!("../assets/signs/sign-01.png"),
    include_bytes!("../assets/signs/sign-02.png"),
    include_bytes!("../assets/signs/sign-03.png"),
    include_bytes!("../assets/signs/sign-04.png"),
    include_bytes!("../assets/signs/sign-05.png"),
    include_bytes!("../assets/signs/sign-06.png"),
    include_bytes!("../assets/signs/sign-07.png"),
    include_bytes!("../assets/signs/sign-08.png"),
    include_bytes!("../assets/signs/sign-09.png"),
    include_bytes!("../assets/signs/sign-10.png"),
    include_bytes!("../assets/signs/sign-11.png"),
];
const PUMPKIN: [&[u8]; 2] = [include_bytes!("../assets/lanterns/shell.png"), include_bytes!("../assets/lanterns/face.png")];

#[cfg(test)]
pub const PICTURES_FOR_TESTS: [(&str, &[u8]); 14] = [
    ("sign-00", SIGNS[0]),
    ("sign-01", SIGNS[1]),
    ("sign-02", SIGNS[2]),
    ("sign-03", SIGNS[3]),
    ("sign-04", SIGNS[4]),
    ("sign-05", SIGNS[5]),
    ("sign-06", SIGNS[6]),
    ("sign-07", SIGNS[7]),
    ("sign-08", SIGNS[8]),
    ("sign-09", SIGNS[9]),
    ("sign-10", SIGNS[10]),
    ("sign-11", SIGNS[11]),
    ("shell", PUMPKIN[0]),
    ("face", PUMPKIN[1]),
];

#[derive(Clone, Copy)]
pub struct Ambient {
    pub light: [f32; 3],
    pub shadow_tint: [f32; 4],
    pub night: f32,
    pub lantern: V2,
}

struct Program {
    id: GLuint,
    uniforms: HashMap<&'static str, GLint>,
}

impl Program {
    fn at(&self, name: &str) -> GLint {
        self.uniforms.get(name).copied().unwrap_or(-1)
    }
}

pub struct Frame {
    pub sprites: usize,
    pub piles: Vec<PileShape>,
    pub extras: bool,
}

pub struct Renderer {
    pub gl: Gl,
    sprite: Program,
    pile: Program,
    sign: Program,
    pumpkin: Program,
    sprite_vao: GLuint,
    empty_vao: GLuint,
    sprite_buffer: GLuint,
    sprite_capacity: usize,
    heights_texture: GLuint,
    heights_rows: i32,
    sprites: Vec<Sprite>,
    heights: Vec<f32>,
    signs: Vec<Option<(GLuint, [f32; 2])>>,
    pumpkin_pictures: [Option<GLuint>; 2],
}

impl Renderer {
    pub fn new(gl: Gl) -> Result<Renderer, String> {
        unsafe {
            let renderer = cstr((gl.GetString)(gl::RENDERER));
            let version = cstr((gl.GetString)(gl::VERSION));
            crate::log!("GPU: {renderer} ({version})");

            let sprite = program(
                &gl,
                SPRITE_VERTEX,
                SPRITE_FRAGMENT,
                &["viewSize", "scale", "time", "light", "shadowTint", "rim", "lantern", "pumpkins", "shake", "panel", "masks", "maskCount"],
            )?;
            let pile = program(
                &gl,
                PILE_VERTEX,
                PILE_FRAGMENT,
                &[
                    "viewSize",
                    "scale",
                    "time",
                    "columnWidth",
                    "columnCount",
                    "pileTop",
                    "occluderCount",
                    "origin",
                    "pileBottom",
                    "heightOffset",
                    "light",
                    "shadowTint",
                    "rim",
                    "lantern",
                    "pumpkins",
                    "shake",
                    "occluders",
                    "heights",
                    "panel",
                    "masks",
                    "maskCount",
                ],
            )?;
            let sign = program(
                &gl,
                SIGN_VERTEX,
                SIGN_FRAGMENT,
                &["viewSize", "shake", "anchor", "size", "angle", "time", "ground", "light", "lantern", "pumpkins", "panel", "picture", "masks", "maskCount"],
            )?;
            let pumpkin = program(
                &gl,
                PUMPKIN_VERTEX,
                PUMPKIN_FRAGMENT,
                &["viewSize", "shake", "anchor", "size", "wick", "seed", "time", "light", "night", "panel", "shell", "face", "masks", "maskCount"],
            )?;

            let mut vaos = [0; 2];
            (gl.GenVertexArrays)(2, vaos.as_mut_ptr());
            let mut buffer = 0;
            (gl.GenBuffers)(1, &mut buffer);
            let mut texture = 0;
            (gl.GenTextures)(1, &mut texture);

            (gl.BindVertexArray)(vaos[0]);
            (gl.BindBuffer)(gl::ARRAY_BUFFER, buffer);
            let stride = size_of::<Sprite>() as i32;
            let attributes: [(u32, i32, usize); 11] =
                [(0, 2, 0), (1, 2, 8), (2, 1, 16), (3, 1, 20), (4, 1, 24), (5, 1, 28), (6, 3, 32), (7, 1, 44), (8, 4, 48), (9, 4, 64), (10, 2, 80)];
            for (location, size, offset) in attributes {
                (gl.EnableVertexAttribArray)(location);
                (gl.VertexAttribPointer)(location, size, gl::FLOAT, 0, stride, offset as *const c_void);
                (gl.VertexAttribDivisor)(location, 1);
            }
            (gl.BindVertexArray)(0);

            (gl.BindTexture)(gl::TEXTURE_2D, texture);
            (gl.TexParameteri)(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
            (gl.TexParameteri)(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
            (gl.TexParameteri)(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
            (gl.TexParameteri)(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);

            check(&gl, "przygotowanie")?;
            Ok(Renderer {
                gl,
                sprite,
                pile,
                sign,
                pumpkin,
                sprite_vao: vaos[0],
                empty_vao: vaos[1],
                sprite_buffer: buffer,
                sprite_capacity: 0,
                heights_texture: texture,
                heights_rows: 0,
                sprites: Vec::new(),
                heights: Vec::new(),
                signs: vec![None; SIGNS.len()],
                pumpkin_pictures: [None; 2],
            })
        }
    }

    pub fn draw(
        &mut self,
        simulation: &SnowSimulation,
        size: (i32, i32),
        scale: f32,
        ambient: Ambient,
        time: f32,
        masks: &[[f32; 4]],
    ) -> Result<Frame, String> {
        let masks = &masks[..masks.len().min(MAX_MASKS)];
        let sprite_count = simulation.sprite_count();
        if self.sprites.len() < sprite_count {
            self.sprites.resize(sprite_count.max(1024) * 3 / 2, Sprite::default());
        }
        let sprites = simulation.write_sprites(&mut self.sprites);
        let columns = simulation.column_count().max(256);
        if self.heights.len() < columns {
            self.heights.resize(columns, 0.0);
        }
        let piles = simulation.write_layers(&mut self.heights);
        let shapes: Vec<PileShape> = piles
            .iter()
            .map(|pile| PileShape {
                origin: (pile.origin.x, pile.origin.y),
                count: pile.count,
                bottom: pile.bottom,
                top: pile.top,
                heights: self.heights[pile.offset..pile.offset + 2 * pile.count].to_vec(),
            })
            .collect();
        let signs = simulation.signs();
        for sign in &signs {
            self.load_sign(sign.caption);
        }
        let pumpkins = simulation.pumpkins();
        if !pumpkins.is_empty() {
            self.load_pumpkin();
        }
        let ghosts = self.sprites[..sprites].iter().any(|sprite| sprite.ghost[0] > 0.0);
        let extras = ambient.night > 0.0 || ghosts || !signs.is_empty() || !pumpkins.is_empty();

        let view = [simulation.width(), simulation.height()];
        let (light, shadow_tint) = halloween::moonlit(ambient.light, ambient.shadow_tint, ambient.night);
        let light = [light[0], light[1], light[2], 1.0];
        let rim = [1.0, 0.56, 0.2, 0.3 * ambient.night];
        let lantern = [ambient.lantern.x, ambient.lantern.y, LANTERN_REACH, ambient.night];
        let mut candles = [[0.0f32; 4]; 3];
        for (candle, pumpkin) in candles.iter_mut().zip(&pumpkins) {
            let wick = wick(pumpkin);
            *candle = [wick.x, wick.y, pumpkin.width * 1.6, 0.4 + 0.6 * ambient.night];
        }
        let shake = simulation.shake();
        let shake = [shake.x, shake.y];
        let panel = [0.0f32; 4];

        let gl = &self.gl;
        unsafe {
            (gl.Viewport)(0, 0, size.0, size.1);
            (gl.ClearColor)(0.0, 0.0, 0.0, 0.0);
            (gl.Clear)(gl::COLOR_BUFFER_BIT);
            (gl.Enable)(gl::BLEND);
            (gl.BlendFunc)(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);

            let common = |p: &Program| {
                (gl.Uniform2f)(p.at("viewSize"), view[0], view[1]);
                (gl.Uniform2f)(p.at("shake"), shake[0], shake[1]);
                (gl.Uniform1f)(p.at("time"), time);
                (gl.Uniform4f)(p.at("light"), light[0], light[1], light[2], light[3]);
                (gl.Uniform4fv)(p.at("panel"), 1, panel.as_ptr());
                (gl.Uniform4fv)(p.at("lantern"), 1, lantern.as_ptr());
                (gl.Uniform4fv)(p.at("pumpkins"), 3, candles.as_ptr().cast());
                (gl.Uniform4fv)(p.at("rim"), 1, rim.as_ptr());
                (gl.Uniform4fv)(p.at("shadowTint"), 1, shadow_tint.as_ptr());
                (gl.Uniform1f)(p.at("scale"), scale);
            };
            let set_masks = |p: &Program, on: bool| {
                let count = if on { masks.len() } else { 0 };
                (gl.Uniform1i)(p.at("maskCount"), count as i32);
                if count > 0 {
                    (gl.Uniform4fv)(p.at("masks"), count as i32, masks.as_ptr().cast());
                }
            };

            if !pumpkins.is_empty()
                && let [Some(shell), Some(face)] = self.pumpkin_pictures
            {
                let p = &self.pumpkin;
                (gl.UseProgram)(p.id);
                (gl.BindVertexArray)(self.empty_vao);
                (gl.ActiveTexture)(gl::TEXTURE1);
                (gl.BindTexture)(gl::TEXTURE_2D, face);
                (gl.ActiveTexture)(gl::TEXTURE0);
                (gl.BindTexture)(gl::TEXTURE_2D, shell);
                common(p);
                set_masks(p, true);
                (gl.Uniform1f)(p.at("night"), ambient.night);
                (gl.Uniform1i)(p.at("shell"), 0);
                (gl.Uniform1i)(p.at("face"), 1);
                for sitting in &pumpkins {
                    let wick = wick(sitting);
                    (gl.Uniform2f)(p.at("anchor"), sitting.anchor.x, sitting.anchor.y);
                    (gl.Uniform2f)(p.at("size"), sitting.width, sitting.width * PUMPKIN_ASPECT);
                    (gl.Uniform2f)(p.at("wick"), wick.x, wick.y);
                    (gl.Uniform1f)(p.at("seed"), sitting.seed);
                    (gl.DrawArrays)(gl::TRIANGLE_STRIP, 0, 4);
                }
            }

            if !piles.is_empty() {
                let rows = (self.heights.len() as i32 + HEIGHTS_WIDTH - 1) / HEIGHTS_WIDTH;
                let padded = (rows * HEIGHTS_WIDTH) as usize;
                if self.heights.len() < padded {
                    self.heights.resize(padded, 0.0);
                }
                (gl.ActiveTexture)(gl::TEXTURE0);
                (gl.BindTexture)(gl::TEXTURE_2D, self.heights_texture);
                (gl.PixelStorei)(gl::UNPACK_ALIGNMENT, 4);
                if rows != self.heights_rows {
                    (gl.TexImage2D)(gl::TEXTURE_2D, 0, gl::R32F, HEIGHTS_WIDTH, rows, 0, gl::RED, gl::FLOAT, self.heights.as_ptr().cast());
                    self.heights_rows = rows;
                } else {
                    (gl.TexSubImage2D)(gl::TEXTURE_2D, 0, 0, 0, HEIGHTS_WIDTH, rows, gl::RED, gl::FLOAT, self.heights.as_ptr().cast());
                }
                let p = &self.pile;
                (gl.UseProgram)(p.id);
                (gl.BindVertexArray)(self.empty_vao);
                common(p);
                (gl.Uniform1i)(p.at("heights"), 0);
                (gl.Uniform1f)(p.at("columnWidth"), COLUMN_WIDTH);
                for pile in &piles {
                    let occluders = pile.occluders.len().min(MAX_OCCLUDERS);
                    (gl.Uniform1i)(p.at("columnCount"), pile.count as i32);
                    (gl.Uniform1f)(p.at("pileTop"), pile.top + 2.0);
                    (gl.Uniform1f)(p.at("pileBottom"), pile.bottom);
                    (gl.Uniform2f)(p.at("origin"), pile.origin.x, pile.origin.y);
                    (gl.Uniform1i)(p.at("heightOffset"), pile.offset as i32);
                    (gl.Uniform1i)(p.at("occluderCount"), occluders as i32);
                    if occluders > 0 {
                        (gl.Uniform4fv)(p.at("occluders"), occluders as i32, pile.occluders.as_ptr().cast());
                    }
                    set_masks(p, pile.origin.y.abs() < 0.5);
                    (gl.DrawArrays)(gl::TRIANGLE_STRIP, 0, 4);
                }
            }

            if !signs.is_empty() {
                let p = &self.sign;
                (gl.UseProgram)(p.id);
                (gl.BindVertexArray)(self.empty_vao);
                (gl.ActiveTexture)(gl::TEXTURE0);
                common(p);
                set_masks(p, true);
                (gl.Uniform1i)(p.at("picture"), 0);
                for held in &signs {
                    let Some(&Some((texture, sign_size))) = self.signs.get(held.caption) else { continue };
                    (gl.BindTexture)(gl::TEXTURE_2D, texture);
                    (gl.Uniform2f)(p.at("anchor"), held.anchor.x, held.anchor.y);
                    (gl.Uniform2f)(p.at("size"), sign_size[0], sign_size[1]);
                    (gl.Uniform1f)(p.at("angle"), held.angle);
                    (gl.Uniform1f)(p.at("ground"), held.ground);
                    (gl.DrawArrays)(gl::TRIANGLE_STRIP, 0, 4);
                }
            }

            if sprites > 0 {
                (gl.BindBuffer)(gl::ARRAY_BUFFER, self.sprite_buffer);
                let bytes = (sprites * size_of::<Sprite>()) as isize;
                if sprites > self.sprite_capacity {
                    self.sprite_capacity = self.sprites.len();
                    (gl.BufferData)(gl::ARRAY_BUFFER, (self.sprite_capacity * size_of::<Sprite>()) as isize, std::ptr::null(), gl::DYNAMIC_DRAW);
                }
                (gl.BufferSubData)(gl::ARRAY_BUFFER, 0, bytes, self.sprites.as_ptr().cast());
                let p = &self.sprite;
                (gl.UseProgram)(p.id);
                (gl.BindVertexArray)(self.sprite_vao);
                common(p);
                set_masks(p, true);
                (gl.DrawArraysInstanced)(gl::TRIANGLE_STRIP, 0, 4, sprites as i32);
            }
            (gl.BindVertexArray)(0);
            check(gl, "rysowanie")?;
        }
        Ok(Frame { sprites, piles: shapes, extras })
    }

    pub fn sprites(&self, count: usize) -> &[Sprite] {
        &self.sprites[..count.min(self.sprites.len())]
    }

    pub fn clear(&self, size: (i32, i32)) {
        unsafe {
            (self.gl.Viewport)(0, 0, size.0, size.1);
            (self.gl.ClearColor)(0.0, 0.0, 0.0, 0.0);
            (self.gl.Clear)(gl::COLOR_BUFFER_BIT);
        }
    }

    fn load_sign(&mut self, caption: usize) {
        if self.signs.get(caption).is_none_or(Option::is_some) {
            return;
        }
        if let Some((texture, [width, height])) = self.picture(SIGNS[caption], false) {
            self.signs[caption] = Some((texture, [width as f32 / 2.0, height as f32 / 2.0]));
        }
    }

    fn load_pumpkin(&mut self) {
        for (part, data) in PUMPKIN.iter().enumerate() {
            if self.pumpkin_pictures[part].is_none() {
                self.pumpkin_pictures[part] = self.picture(data, true).map(|(texture, _)| texture);
            }
        }
    }

    fn picture(&self, data: &[u8], mipmapped: bool) -> Option<(GLuint, [u32; 2])> {
        let mut picture = match png::decode(data) {
            Ok(picture) => picture,
            Err(error) => {
                crate::log!("obrazek Halloween: {error}");
                return None;
            }
        };
        for pixel in picture.pixels.chunks_exact_mut(4) {
            let alpha = pixel[3] as u32;
            for channel in &mut pixel[..3] {
                *channel = ((*channel as u32 * alpha + 127) / 255) as u8;
            }
        }
        let gl = &self.gl;
        unsafe {
            let mut texture = 0;
            (gl.GenTextures)(1, &mut texture);
            (gl.ActiveTexture)(gl::TEXTURE0);
            (gl.BindTexture)(gl::TEXTURE_2D, texture);
            (gl.TexParameteri)(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, if mipmapped { gl::LINEAR_MIPMAP_LINEAR } else { gl::LINEAR });
            (gl.TexParameteri)(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR);
            (gl.TexParameteri)(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
            (gl.TexParameteri)(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);
            (gl.PixelStorei)(gl::UNPACK_ALIGNMENT, 4);
            (gl.TexImage2D)(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as GLint,
                picture.width as i32,
                picture.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                picture.pixels.as_ptr().cast(),
            );
            if mipmapped {
                (gl.GenerateMipmap)(gl::TEXTURE_2D);
            }
            Some((texture, [picture.width, picture.height]))
        }
    }
}

fn wick(pumpkin: &PumpkinSprite) -> V2 {
    let [across, up] = PUMPKIN_CANDLE;
    pumpkin.anchor + V2::new((across - 0.5) * pumpkin.width, up * pumpkin.width * PUMPKIN_ASPECT)
}

fn check(gl: &Gl, what: &str) -> Result<(), String> {
    let error = unsafe { (gl.GetError)() };
    if error == gl::NO_ERROR { Ok(()) } else { Err(format!("OpenGL: błąd 0x{error:x} ({what})")) }
}

fn cstr(pointer: *const u8) -> String {
    if pointer.is_null() { String::new() } else { unsafe { std::ffi::CStr::from_ptr(pointer.cast()) }.to_string_lossy().into_owned() }
}

unsafe fn shader(gl: &Gl, kind: u32, source: &str) -> Result<GLuint, String> {
    unsafe {
        let id = (gl.CreateShader)(kind);
        let c = CString::new(source).unwrap();
        let pointer = c.as_ptr();
        (gl.ShaderSource)(id, 1, &pointer, std::ptr::null());
        (gl.CompileShader)(id);
        let mut ok = 0;
        (gl.GetShaderiv)(id, gl::COMPILE_STATUS, &mut ok);
        if ok == 0 {
            let mut length = 0;
            (gl.GetShaderiv)(id, gl::INFO_LOG_LENGTH, &mut length);
            let mut log = vec![0u8; length.max(1) as usize];
            (gl.GetShaderInfoLog)(id, length, std::ptr::null_mut(), log.as_mut_ptr().cast());
            return Err(format!("shader nie kompiluje się: {}", String::from_utf8_lossy(&log)));
        }
        Ok(id)
    }
}

unsafe fn program(gl: &Gl, vertex: &str, fragment: &str, names: &[&'static str]) -> Result<Program, String> {
    unsafe {
        let v = shader(gl, gl::VERTEX_SHADER, vertex)?;
        let f = shader(gl, gl::FRAGMENT_SHADER, fragment)?;
        let id = (gl.CreateProgram)();
        (gl.AttachShader)(id, v);
        (gl.AttachShader)(id, f);
        (gl.LinkProgram)(id);
        let mut ok = 0;
        (gl.GetProgramiv)(id, gl::LINK_STATUS, &mut ok);
        if ok == 0 {
            let mut length = 0;
            (gl.GetProgramiv)(id, gl::INFO_LOG_LENGTH, &mut length);
            let mut log = vec![0u8; length.max(1) as usize];
            (gl.GetProgramInfoLog)(id, length, std::ptr::null_mut(), log.as_mut_ptr().cast());
            return Err(format!("program nie łączy się: {}", String::from_utf8_lossy(&log)));
        }
        (gl.DeleteShader)(v);
        (gl.DeleteShader)(f);
        let uniforms = names
            .iter()
            .map(|&name| {
                let c = CString::new(name).unwrap();
                (name, (gl.GetUniformLocation)(id, c.as_ptr()))
            })
            .collect();
        Ok(Program { id, uniforms })
    }
}
