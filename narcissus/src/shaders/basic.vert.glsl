#version 460

layout(set = 0, binding = 0) uniform uniformBuffer {
    mat4 viewProj;
};

struct VertexData {
    vec4 position;
    vec4 normal;
    vec4 texcoord;
};

struct TransformData {
    vec4 transform[3];
};

layout(std430, set = 1, binding = 0) readonly buffer vertexBuffer {
    VertexData vertices[];
};

layout(std430, set = 1, binding = 1) readonly buffer transformBuffer {
    TransformData transforms[];
};

layout(location = 0) out vec3 fragColor;

void main() {
    TransformData td = transforms[gl_InstanceIndex];
    VertexData vd = vertices[gl_VertexIndex];

    vec4 posLocal = vec4(vd.position.xyz, 1.0);
    vec3 posWorld = mat4x3(td.transform[0], td.transform[1], td.transform[2]) * posLocal;
    vec4 posClip = vec4(posWorld, 1.0) * viewProj;

    gl_Position = posClip;
    fragColor = vd.normal.xyz * 0.5 + 0.5;
}
