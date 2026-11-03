#version 300 es
precision highp float;
precision highp int;

uniform float scale;
uniform vec4 panel;

in vec2 vCoord;
in float vAlpha;
in float vSoft;
in vec2 vStreak;
in float vRadius;
flat in vec4 vRain;
flat in vec4 vDrop;

out vec4 fragment;

#include "drop.glsl"

void main() {
    if (vRain.x < 1.5 || vRain.x > 2.5) discard;
    vec2 at = gl_FragCoord.xy;
    bool hidden = at.x >= panel.x && at.x < panel.z && at.y >= panel.y && at.y < panel.w;
    if (hidden && vSoft < 0.5) discard;
    vec2 p = vec2(vCoord.x - 0.5, 0.5 - vCoord.y);
    vec2 halfStreak = vStreak * 0.5;
    float span = dot(halfStreak, halfStreak);
    vec2 axis = span > 0.0 ? halfStreak / sqrt(span) : vec2(0.0, 1.0);
    float total = 2.0 * sqrt(span);
    float smear = total * vRain.y;
    int moments = clamp(int(ceil(smear * vRain.w / 0.75)), 1, 16);
    float bump = 0.0;
    for (int k = 0; k < moments; k++) {
        float back = moments > 1 ? float(k) / float(moments - 1) : 0.0;
        float d = dropDistance(p, -halfStreak + axis * smear * back, axis, total - smear);
        float q = clamp(1.0 - d * d / 2.25, 0.0, 1.0);
        bump += q * q;
    }
    bump *= vAlpha / float(moments);
    if (bump < 0.001) discard;
    fragment = vec4(bump, bump * vRain.z * 0.5 / scale, 0.0, 0.0);
}
