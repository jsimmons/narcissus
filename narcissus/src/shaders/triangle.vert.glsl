#version 450

layout(set = 0, binding = 0) uniform uniformBuffer {
    float someValue;
};

layout(location = 0) out vec3 fragColor;

vec2 positions[3] = vec2[](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

vec3 colors[3] = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
);

void main() {
    vec2 pos = positions[gl_VertexIndex];
    pos.y += sin(someValue);
    gl_Position = vec4(pos, 0.0, 1.0);
    fragColor = colors[gl_VertexIndex];
}