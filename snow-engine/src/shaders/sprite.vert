#version 300 es
precision highp float;
precision highp int;

uniform vec2 viewSize;
uniform float scale;
uniform float time;
uniform vec4 lantern;
uniform vec4 pumpkins[3];
uniform vec2 shake;

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 velocity;
layout(location = 2) in float size;
layout(location = 3) in float alpha;
layout(location = 4) in float soft;
layout(location = 5) in float tintAmount;
layout(location = 6) in vec3 tintColor;
layout(location = 7) in float lump;
layout(location = 8) in vec4 ghost;
layout(location = 9) in vec4 pose;
layout(location = 10) in vec2 mood;
layout(location = 11) in vec3 leaf;
layout(location = 12) in vec2 spin;
layout(location = 13) in vec2 rain;
layout(location = 14) in vec4 drop;

out vec2 vCoord;
out float vAlpha;
out float vSoft;
out vec4 vTint;
out vec2 vStreak;
out float vRadius;
out float vLump;
out float vSpin;
out float vWarm;
out vec2 vWorld;
flat out vec4 vGhost;
flat out vec4 vPose;
flat out vec4 vMood;
flat out vec4 vLeaf;
flat out vec4 vSmear;
flat out vec4 vRain;
flat out vec4 vDrop;

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

vec3 hangingShape(float along) {
    float fill = clamp(along, 0.0, 1.0), pull = clamp(along - 1.0, 0.0, 1.0);
    vec2 axes = vec2(mix(1.2, 0.9, fill) * (1.0 - 0.15 * pull), mix(0.6, 1.25, fill) * (1.0 + 0.5 * pull));
    return vec3(axes, -axes.y * mix(0.15, 0.8, fill) - 0.3 * pull);
}

float hangingFlare(float along) {
    return mix(0.8, 0.6, clamp(along, 0.0, 1.0));
}

vec2 hangingReach(float along) {
    vec3 shape = hangingShape(along);
    return vec2(0.05, shape.z - shape.y);
}

void main() {
    vec2 corner = vec2(float(gl_VertexID & 1), float(gl_VertexID >> 1));
    vCoord = vec2(corner.x, 1.0 - corner.y);
    vWarm = candlelight(position);
    vRain = vec4(0.0);
    vDrop = vec4(0.0);
    if (rain.x > 4.5) {
        float thick = max(size * scale, 1.0);
        vec2 piece = velocity * scale;
        float span = length(piece) + thick + 2.0;
        vec2 at = (position + shake) / viewSize * 2.0 - 1.0;
        gl_Position = vec4(at + (corner - 0.5) * span * 2.0 / (viewSize * scale), 0.0, 1.0);
        vWorld = (gl_Position.xy + 1.0) * 0.5 * viewSize;
        vStreak = piece / span;
        vRadius = thick / span * 0.5;
        vAlpha = alpha;
        vSoft = soft;
        vRain = vec4(rain.x, 0.0, thick, span);
        vLeaf = vec4(0.0);
        vGhost = vec4(0.0);
        return;
    }
    if (rain.x > 0.0) {
        float pixels = max(size * scale, 1.5);
        bool glass = rain.x > 1.5;
        float speed = length(velocity);
        float tail = min(speed * 0.08, size * 0.9);
        float smear = speed * 0.024;
        vec2 travel = velocity * 0.028;
        if (glass) travel = speed > 0.0 ? -velocity / speed * (tail + smear) : vec2(0.0);
        vec2 streak = travel * scale;
        float reach = rain.x > 1.5 && rain.x < 2.5 ? 2.2 : glass ? 1.4 : 1.0;
        float extent = max(pixels * reach + length(streak) + (glass ? 2.0 : 0.0), 2.0);
        vec2 middle = position - (glass ? -travel : travel) * 0.5;
        if (rain.x > 2.5 && rain.x < 3.5) {
            vec2 reach = hangingReach(rain.y) * pixels * 0.5;
            vec3 shape = hangingShape(rain.y);
            extent = max(reach.x - reach.y, 2.0 * (shape.x + 1.5 * hangingFlare(rain.y)) * pixels * 0.5) + 4.0;
            middle = position + vec2(0.0, (reach.x + reach.y) * 0.5) / scale;
            streak = vec2(0.0);
        }
        vec2 center = (middle + shake) / viewSize * 2.0 - 1.0;
        gl_Position = vec4(center + (corner - 0.5) * extent * 2.0 / (viewSize * scale), 0.0, 1.0);
        vWorld = (gl_Position.xy + 1.0) * 0.5 * viewSize;
        vStreak = streak / extent;
        vRadius = pixels / extent * 0.5;
        vAlpha = alpha * min(pixels / 1.5, 1.0);
        vSoft = soft;
        float blur = rain.x < 2.5 && tail + smear > 0.0 ? smear / (tail + smear) : rain.y;
        vRain = vec4(rain.x, glass ? blur : rain.y, pixels, extent);
        vDrop = drop;
        vLeaf = vec4(0.0);
        vGhost = vec4(0.0);
        return;
    }
    if (leaf.x != 0.0) {
        float exposure = 0.024;
        float pixels = max(size * scale, 2.0);
        vec2 travel = velocity * exposure;
        float extent = pixels + length(travel) * scale;
        vec2 center = (position - travel * 0.5 + shake) / viewSize * 2.0 - 1.0;
        gl_Position = vec4(center + (corner - 0.5) * extent * 2.0 / (viewSize * scale), 0.0, 1.0);
        vWorld = (gl_Position.xy + 1.0) * 0.5 * viewSize;
        vStreak = travel / max(size, 0.001);
        vec2 smear = spin * exposure;
        float motion = length(travel) * scale + (abs(smear.x) + abs(smear.y)) * pixels * 0.5;
        vLeaf = vec4(leaf, extent / pixels);
        vSmear = vec4(smear, clamp(ceil(motion / 1.2), 1.0, 8.0), 0.0);
        vAlpha = alpha;
        return;
    }
    vLeaf = vec4(0.0);
    float pixels = size * scale;
    vec2 travel = velocity * 0.018;
    float smear = min(length(travel), size * (lump > 0.0 ? 1.2 : 2.2));
    vec2 streak = smear > 0.25 ? normalize(travel) * smear * scale : vec2(0.0);
    float reach = lump > 0.0 ? 1.3 : 1.0;
    float extent = max(pixels * reach + length(streak), 2.0);
    vec2 center = (position + shake) / viewSize * 2.0 - 1.0;
    gl_Position = vec4(center + (corner - 0.5) * extent * 2.0 / (viewSize * scale), 0.0, 1.0);
    vWorld = (gl_Position.xy + 1.0) * 0.5 * viewSize;
    vStreak = streak / extent;
    vRadius = pixels / extent * 0.5;
    vAlpha = alpha * min(pixels / 2.0, 1.0) * pixels / max(pixels + length(streak) * 0.6, 0.001);
    vSoft = soft;
    vTint = vec4(tintColor, tintAmount);
    vLump = lump;
    vSpin = lump > 0.0 ? time * (lump - 0.5) * 9.0 + lump * 40.0 : 0.0;
    vGhost = vec4(ghost.x, (ghost.y - position.y) / max(size, 0.001), ghost.zw);
    vPose = pose;
    vMood = vec4(mood, size, 0.0);
    if (ghost.x > 0.0) vSpin = time + ghost.x * 40.0;
}
