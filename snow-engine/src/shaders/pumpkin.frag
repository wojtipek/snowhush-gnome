#version 300 es
precision highp float;
precision highp int;
precision highp sampler2D;

uniform float time;
uniform vec4 light;
uniform float night;
uniform vec4 panel;
uniform vec2 size;
uniform vec2 wick;
uniform float seed;
uniform sampler2D shell;
uniform sampler2D face;

in vec2 vUv;
in vec2 vWorld;

out vec4 fragment;

const vec3 candle = vec3(1.0, 0.62, 0.26);

uniform vec4 masks[32];
uniform int maskCount;

bool masked(vec2 point) {
    for (int k = 0; k < 32; k++) {
        if (k >= maskCount) break;
        vec4 m = masks[k];
        if (point.x >= m.x && point.x < m.z && point.y >= m.y && point.y < m.w) return true;
    }
    return false;
}

float flicker(float t, float seed) {
    return 0.86 + 0.08 * sin(t * 11.0 + seed * 2.1) + 0.06 * sin(t * 29.0 + 1.3 + seed * 4.3);
}

void main() {
    float inside = step(0.0, vUv.x) * step(vUv.x, 1.0) * step(0.0, vUv.y) * step(vUv.y, 1.0);
    vec4 body = texture(shell, vUv) * inside;
    vec4 fire = texture(face, vUv) * inside;
    if (vWorld.x >= panel.x && vWorld.x < panel.z && vWorld.y >= panel.y && vWorld.y < panel.w) discard;
    if (masked(vWorld)) discard;
    float flame = flicker(time, seed + 1.0);
    body.rgb *= mix(vec3(1.0), light.rgb, 0.5);
    fire.rgb *= 0.8 + 0.25 * flame;
    vec4 color = fire + body * (1.0 - fire.a);
    vec2 d = (vWorld - wick) / (size.x * 0.45);
    float halo = night * 0.3 * flame * exp(-dot(d, d));
    fragment = color + vec4(candle * halo, halo) * (1.0 - color.a);
}
