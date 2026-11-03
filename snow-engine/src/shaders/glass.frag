#version 300 es
precision highp float;
precision highp int;

uniform highp sampler2D field;
uniform vec4 light;
uniform float scale;

out vec4 fragment;

void main() {
    const float threshold = 0.3086;
    ivec2 last = textureSize(field, 0) - 1;
    ivec2 c = ivec2(gl_FragCoord.xy);
    float f = texelFetch(field, c, 0).r;
    float edge = max(fwidth(f), 0.002);
    float above = texelFetch(field, clamp(c + ivec2(0, max(1, int(2.0 * scale + 0.5))), ivec2(0), last), 0).r;
    if (max(f, above) < threshold * 0.55) discard;

    float left = texelFetch(field, clamp(c - ivec2(1, 0), ivec2(0), last), 0).g;
    float right = texelFetch(field, clamp(c + ivec2(1, 0), ivec2(0), last), 0).g;
    float up = texelFetch(field, clamp(c + ivec2(0, 1), ivec2(0), last), 0).g;
    float down = texelFetch(field, clamp(c - ivec2(0, 1), ivec2(0), last), 0).g;
    vec2 slope = -vec2(right - left, down - up) * 0.5 * scale * 1.1;
    vec3 n = normalize(vec3(slope, 1.0));
    vec2 toward = normalize(vec2(-0.55, -0.83));
    float tilt = clamp(length(n.xy) / 0.72, 0.0, 1.0);
    float side = dot(n.xy, toward) / 0.72;

    float mask = smoothstep(threshold - edge, threshold + edge, f);
    float shadow = smoothstep(threshold * 0.55, threshold * 1.2, above) * (1.0 - mask) * 0.1;
    float rim = smoothstep(0.62, 1.0, tilt);
    float rimLight = rim * (0.1 + 0.6 * pow(clamp(side, 0.0, 1.0), 1.5) + 0.3 * pow(clamp(-side, 0.0, 1.0), 1.5));
    vec3 halfway = normalize(vec3(toward * 0.5, 0.87));
    float specular = pow(clamp(dot(n, halfway), 0.0, 1.0), 80.0) * 0.8;
    float crescent = 0.16 * clamp(-side, 0.0, 1.0) * smoothstep(0.25, 0.8, tilt) * (1.0 - rim);
    float shade = 0.04 + 0.32 * rim * rim * (0.6 + 0.4 * clamp(-side, 0.0, 1.0));

    float glow = (rimLight + specular + crescent + 0.01) * mask;
    float alpha = shade * mask + shadow;
    fragment = vec4(vec3(0.95, 0.975, 1.0) * light.rgb * min(glow, 1.0), alpha);
}
