use std::ffi::{CString, c_void};
use std::mem::size_of;

use snow_core::simulation::{COLUMN_WIDTH, SnowSimulation, Sprite};

use crate::damage::PileShape;
use crate::gl::{self, Gl, GLint, GLuint};

const HEIGHTS_WIDTH: i32 = 4096;
const MAX_OCCLUDERS: usize = 16;
const MAX_MASKS: usize = 32;

const SPRITE_VERTEX: &str = r#"#version 300 es
precision highp float;
precision highp int;
layout(location = 0) in vec2 aPosition;
layout(location = 1) in vec2 aVelocity;
layout(location = 2) in float aSize;
layout(location = 3) in float aAlpha;
layout(location = 4) in float aSoft;
layout(location = 5) in float aTintAmount;
layout(location = 6) in vec3 aTintColor;
layout(location = 7) in float aLump;
uniform vec2 uViewSize;
uniform float uScale;
uniform float uTime;
out vec2 vCoord;
out float vAlpha;
out float vSoft;
out vec4 vTint;
out vec2 vStreak;
out float vRadius;
out float vLump;
out float vSpin;
out vec2 vWorld;
void main() {
    float pixels = aSize * uScale;
    vec2 travel = aVelocity * 0.018;
    float smear = min(length(travel), aSize * (aLump > 0.0 ? 1.2 : 2.2));
    vec2 streak = smear > 0.25 ? normalize(travel) * smear * uScale : vec2(0.0);
    float reach = aLump > 0.0 ? 1.3 : 1.0;
    float extent = max(pixels * reach + length(streak), 2.0);
    int vid = gl_VertexID;
    vec2 corner = vec2(float(vid & 1), float(vid >> 1));
    vec2 center = aPosition / uViewSize * 2.0 - 1.0;
    gl_Position = vec4(center + (corner - 0.5) * extent * 2.0 / (uViewSize * uScale), 0.0, 1.0);
    vCoord = vec2(corner.x, 1.0 - corner.y);
    vStreak = streak / extent;
    vRadius = pixels / extent * 0.5;
    vAlpha = aAlpha * min(pixels / 2.0, 1.0) * pixels / max(pixels + length(streak) * 0.6, 0.001);
    vSoft = aSoft;
    vTint = vec4(aTintColor, aTintAmount);
    vLump = aLump;
    vSpin = aLump > 0.0 ? uTime * (aLump - 0.5) * 9.0 + aLump * 40.0 : 0.0;
    vWorld = aPosition + (corner - 0.5) * extent / uScale;
}
"#;

const SPRITE_FRAGMENT: &str = r#"#version 300 es
precision highp float;
uniform vec4 uLight;
uniform vec4 uMasks[32];
uniform int uMaskCount;
in vec2 vWorld;
in vec2 vCoord;
in float vAlpha;
in float vSoft;
in vec4 vTint;
in vec2 vStreak;
in float vRadius;
in float vLump;
in float vSpin;
out vec4 fragColor;
void main() {
    for (int k = 0; k < 32; k++) {
        if (k >= uMaskCount) break;
        vec4 m = uMasks[k];
        if (vWorld.x >= m.x && vWorld.x < m.z && vWorld.y >= m.y && vWorld.y < m.w) discard;
    }
    vec2 p = vec2(vCoord.x - 0.5, 0.5 - vCoord.y);
    vec2 half_ = vStreak * 0.5;
    float span = dot(half_, half_);
    float t = span > 0.0 ? clamp(dot(p, half_) / span, -1.0, 1.0) : 0.0;
    vec2 d = p - half_ * t;
    float r = length(d) / max(vRadius, 0.001);
    float shading = 0.0;
    if (vLump > 0.0) {
        float angle = atan(d.y, d.x) + vSpin;
        r /= 1.0 + 0.14 * sin(3.0 * angle) + 0.08 * sin(5.0 * angle + vLump * 21.0) + 0.05 * sin(2.0 * angle + vLump * 9.0);
        shading = clamp(dot(d / max(vRadius, 0.001), vec2(0.45, -0.6)), 0.0, 1.0) * 0.4;
    }
    float head = span > 0.0 ? 0.15 + 0.85 * pow(t * 0.5 + 0.5, 1.5) : 1.0;
    float a = (1.0 - smoothstep(mix(0.6, 0.0, vSoft), 1.0, r)) * vAlpha * head;
    if (a < 0.002) discard;
    vec3 core = vec3(0.965, 0.975, 0.995);
    vec3 rim = vec3(0.62, 0.67, 0.77);
    vec3 color = mix(core, rim, clamp(smoothstep(0.35, 1.0, r) * 0.65 + shading, 0.0, 1.0));
    color = mix(color, vTint.rgb, vTint.a) * uLight.rgb;
    fragColor = vec4(color * a, a);
}
"#;

const PILE_VERTEX: &str = r#"#version 300 es
precision highp float;
precision highp int;
uniform vec2 uViewSize;
uniform float uColumnWidth;
uniform int uColumnCount;
uniform float uPileTop;
uniform float uPileBottom;
uniform vec2 uOrigin;
out vec2 vLocal;
void main() {
    int vid = gl_VertexID;
    vec2 corner = vec2(float(vid & 1), float(vid >> 1));
    vec2 p = vec2(corner.x * float(uColumnCount) * uColumnWidth, mix(uPileBottom, uPileTop, corner.y));
    gl_Position = vec4((p + uOrigin) / uViewSize * 2.0 - 1.0, 0.0, 1.0);
    vLocal = p;
}
"#;

const PILE_FRAGMENT: &str = r#"#version 300 es
precision highp float;
precision highp int;
precision highp sampler2D;
uniform sampler2D uHeights;
uniform float uScale;
uniform float uTime;
uniform float uColumnWidth;
uniform int uColumnCount;
uniform int uOccluderCount;
uniform vec2 uOrigin;
uniform int uHeightOffset;
uniform vec4 uLight;
uniform vec4 uShadowTint;
uniform vec4 uOccluders[16];
uniform vec4 uMasks[32];
uniform int uMaskCount;
in vec2 vLocal;
out vec4 fragColor;

float heightValue(int index) {
    return texelFetch(uHeights, ivec2(index % 4096, index / 4096), 0).r;
}

float heightAt(int start, float x) {
    float f = x / uColumnWidth - 0.5;
    float base = floor(f);
    int i = clamp(int(base), 0, uColumnCount - 1);
    int j = min(i + 1, uColumnCount - 1);
    return mix(heightValue(start + i), heightValue(start + j), clamp(f - base, 0.0, 1.0));
}

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

void main() {
    vec2 world = vLocal + uOrigin;
    for (int k = 0; k < 16; k++) {
        if (k >= uOccluderCount) break;
        vec4 o = uOccluders[k];
        if (world.x >= o.x && world.x < o.z && world.y >= o.y && world.y < o.w) discard;
    }
    for (int k = 0; k < 32; k++) {
        if (k >= uMaskCount) break;
        vec4 m = uMasks[k];
        if (world.x >= m.x && world.x < m.z && world.y >= m.y && world.y < m.w) discard;
    }

    float x = vLocal.x;
    float level = heightAt(uHeightOffset, x);
    float bed = heightAt(uHeightOffset + uColumnCount, x);
    float depth = level - vLocal.y;
    float coverage = clamp(depth * uScale + 0.5, 0.0, 1.0)
        * clamp((vLocal.y - bed) * uScale + 0.5, 0.0, 1.0)
        * clamp((level - bed) / 0.75, 0.0, 1.0);
    if (coverage <= 0.0) discard;

    float slope = (heightAt(uHeightOffset, x + 2.0) - heightAt(uHeightOffset, x - 2.0)) * 0.25;
    float lit = dot(normalize(vec2(-slope, 1.0)), normalize(vec2(-0.5, 1.0)));

    vec3 shadow = mix(vec3(0.72, 0.78, 0.88), uShadowTint.rgb, uShadowTint.a);
    vec3 surface = mix(vec3(0.925, 0.94, 0.965), min(uShadowTint.rgb * 1.18, vec3(1.0)), uShadowTint.a * 0.2);
    float shade = (1.0 - exp(-depth / 30.0)) * 0.32 + (1.0 - lit) * 0.6 * exp(-depth / 5.0);
    vec3 color = mix(surface, shadow, clamp(shade, 0.0, 1.0));
    color = mix(color, vec3(0.975, 0.982, 0.995), exp(-depth * 0.9) * 0.6);
    color += (hash(floor(gl_FragCoord.xy)) - 0.5) * 0.02;

    float seed = hash(floor(gl_FragCoord.xy * 0.5) + 31.7);
    if (seed > 0.992) {
        float twinkle = pow(clamp(sin(uTime * (0.6 + seed * 1.8) + seed * 90.0), 0.0, 1.0), 28.0);
        color = mix(color, vec3(1.0), twinkle * exp(-depth / 7.0));
    }

    color *= uLight.rgb;
    fragColor = vec4(color * coverage, coverage);
}
"#;

#[derive(Clone, Copy)]
pub struct Ambient {
    pub light: [f32; 3],
    pub shadow_tint: [f32; 4],
}

struct SpriteProgram {
    id: GLuint,
    view_size: GLint,
    scale: GLint,
    time: GLint,
    light: GLint,
    masks: GLint,
    mask_count: GLint,
}

struct PileProgram {
    id: GLuint,
    view_size: GLint,
    column_width: GLint,
    column_count: GLint,
    pile_top: GLint,
    pile_bottom: GLint,
    origin: GLint,
    heights: GLint,
    scale: GLint,
    time: GLint,
    occluder_count: GLint,
    height_offset: GLint,
    light: GLint,
    shadow_tint: GLint,
    occluders: GLint,
    masks: GLint,
    mask_count: GLint,
}

pub struct Renderer {
    pub gl: Gl,
    sprite: SpriteProgram,
    pile: PileProgram,
    sprite_vao: GLuint,
    pile_vao: GLuint,
    sprite_buffer: GLuint,
    sprite_capacity: usize,
    heights_texture: GLuint,
    heights_rows: i32,
    sprites: Vec<Sprite>,
    heights: Vec<f32>,
}

impl Renderer {
    pub fn new(gl: Gl) -> Result<Renderer, String> {
        unsafe {
            let renderer = cstr((gl.GetString)(gl::RENDERER));
            let version = cstr((gl.GetString)(gl::VERSION));
            crate::log!("GPU: {renderer} ({version})");

            let sprite_id = program(&gl, SPRITE_VERTEX, SPRITE_FRAGMENT)?;
            let pile_id = program(&gl, PILE_VERTEX, PILE_FRAGMENT)?;
            let u = |id: GLuint, name: &str| {
                let c = CString::new(name).unwrap();
                (gl.GetUniformLocation)(id, c.as_ptr())
            };
            let sprite = SpriteProgram {
                id: sprite_id,
                view_size: u(sprite_id, "uViewSize"),
                scale: u(sprite_id, "uScale"),
                time: u(sprite_id, "uTime"),
                light: u(sprite_id, "uLight"),
                masks: u(sprite_id, "uMasks"),
                mask_count: u(sprite_id, "uMaskCount"),
            };
            let pile = PileProgram {
                id: pile_id,
                view_size: u(pile_id, "uViewSize"),
                column_width: u(pile_id, "uColumnWidth"),
                column_count: u(pile_id, "uColumnCount"),
                pile_top: u(pile_id, "uPileTop"),
                pile_bottom: u(pile_id, "uPileBottom"),
                origin: u(pile_id, "uOrigin"),
                heights: u(pile_id, "uHeights"),
                scale: u(pile_id, "uScale"),
                time: u(pile_id, "uTime"),
                occluder_count: u(pile_id, "uOccluderCount"),
                height_offset: u(pile_id, "uHeightOffset"),
                light: u(pile_id, "uLight"),
                shadow_tint: u(pile_id, "uShadowTint"),
                occluders: u(pile_id, "uOccluders"),
                masks: u(pile_id, "uMasks"),
                mask_count: u(pile_id, "uMaskCount"),
            };

            let mut vaos = [0; 2];
            (gl.GenVertexArrays)(2, vaos.as_mut_ptr());
            let mut buffer = 0;
            (gl.GenBuffers)(1, &mut buffer);
            let mut texture = 0;
            (gl.GenTextures)(1, &mut texture);

            (gl.BindVertexArray)(vaos[0]);
            (gl.BindBuffer)(gl::ARRAY_BUFFER, buffer);
            let stride = size_of::<Sprite>() as i32;
            let attributes: [(u32, i32, usize); 8] = [(0, 2, 0), (1, 2, 8), (2, 1, 16), (3, 1, 20), (4, 1, 24), (5, 1, 28), (6, 3, 32), (7, 1, 44)];
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
                sprite_vao: vaos[0],
                pile_vao: vaos[1],
                sprite_buffer: buffer,
                sprite_capacity: 0,
                heights_texture: texture,
                heights_rows: 0,
                sprites: Vec::new(),
                heights: Vec::new(),
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
    ) -> Result<(usize, Vec<PileShape>), String> {
        let masks = &masks[..masks.len().min(MAX_MASKS)];
        let gl = &self.gl;
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

        unsafe {
            (gl.Viewport)(0, 0, size.0, size.1);
            (gl.ClearColor)(0.0, 0.0, 0.0, 0.0);
            (gl.Clear)(gl::COLOR_BUFFER_BIT);
            (gl.Enable)(gl::BLEND);
            (gl.BlendFunc)(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            let light = [ambient.light[0], ambient.light[1], ambient.light[2], 1.0];

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
                (gl.BindVertexArray)(self.pile_vao);
                (gl.Uniform1i)(p.heights, 0);
                (gl.Uniform2f)(p.view_size, simulation.width(), simulation.height());
                (gl.Uniform1f)(p.scale, scale);
                (gl.Uniform1f)(p.time, time);
                (gl.Uniform1f)(p.column_width, COLUMN_WIDTH);
                (gl.Uniform4f)(p.light, light[0], light[1], light[2], light[3]);
                let tint = ambient.shadow_tint;
                (gl.Uniform4f)(p.shadow_tint, tint[0], tint[1], tint[2], tint[3]);
                for pile in &piles {
                    let occluders = pile.occluders.len().min(MAX_OCCLUDERS);
                    (gl.Uniform1i)(p.column_count, pile.count as i32);
                    (gl.Uniform1f)(p.pile_top, pile.top + 2.0);
                    (gl.Uniform1f)(p.pile_bottom, pile.bottom);
                    (gl.Uniform2f)(p.origin, pile.origin.x, pile.origin.y);
                    (gl.Uniform1i)(p.height_offset, pile.offset as i32);
                    (gl.Uniform1i)(p.occluder_count, occluders as i32);
                    if occluders > 0 {
                        (gl.Uniform4fv)(p.occluders, occluders as i32, pile.occluders.as_ptr().cast());
                    }
                    let ground = pile.origin.y.abs() < 0.5;
                    let count = if ground { masks.len() } else { 0 };
                    (gl.Uniform1i)(p.mask_count, count as i32);
                    if count > 0 {
                        (gl.Uniform4fv)(p.masks, count as i32, masks.as_ptr().cast());
                    }
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
                let s = &self.sprite;
                (gl.UseProgram)(s.id);
                (gl.BindVertexArray)(self.sprite_vao);
                (gl.Uniform2f)(s.view_size, simulation.width(), simulation.height());
                (gl.Uniform1f)(s.scale, scale);
                (gl.Uniform1f)(s.time, time);
                (gl.Uniform4f)(s.light, light[0], light[1], light[2], light[3]);
                (gl.Uniform1i)(s.mask_count, masks.len() as i32);
                if !masks.is_empty() {
                    (gl.Uniform4fv)(s.masks, masks.len() as i32, masks.as_ptr().cast());
                }
                (gl.DrawArraysInstanced)(gl::TRIANGLE_STRIP, 0, 4, sprites as i32);
            }
            (gl.BindVertexArray)(0);
            check(gl, "rysowanie")?;
        }
        Ok((sprites, shapes))
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

unsafe fn program(gl: &Gl, vertex: &str, fragment: &str) -> Result<GLuint, String> {
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
        Ok(id)
    }
}
