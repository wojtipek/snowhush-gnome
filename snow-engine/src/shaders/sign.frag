#version 300 es
precision highp float;
precision highp int;
precision highp sampler2D;

uniform float time;
uniform float ground;
uniform vec4 light;
uniform vec4 lantern;
uniform vec4 pumpkins[3];
uniform vec4 panel;
uniform sampler2D picture;

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

float candlelight(vec2 point) {
    vec2 d = point - lantern.xy;
    float glow = lantern.w * flicker(time, 0.0) * exp(-dot(d, d) / (lantern.z * lantern.z));
    for (int k = 0; k < 3; k++) {
        vec4 pumpkin = pumpkins[k];
        if (pumpkin.w <= 0.0) continue;
        vec2 e = point - pumpkin.xy;
        glow += pumpkin.w * flicker(time, float(k + 1)) * exp(-dot(e, e) / (pumpkin.z * pumpkin.z));
    }
    return min(glow, 1.0);
}

void main() {
    if (vWorld.x >= panel.x && vWorld.x < panel.z && vWorld.y >= panel.y && vWorld.y < panel.w) discard;
    if (masked(vWorld)) discard;
    vec4 color = texture(picture, vUv);
    color *= smoothstep(ground - 1.0, ground + 1.0, vWorld.y);
    color.rgb *= mix(vec3(1.0), light.rgb, 0.5);
    color.rgb = mix(color.rgb, candle * color.a, candlelight(vWorld) * 0.3);
    fragment = color;
}
