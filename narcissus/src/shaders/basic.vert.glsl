#version 460

layout(set = 0, binding = 0) uniform uniformBuffer {
    mat4 viewProj;
};

struct Vertex {
    vec4 position;
    vec4 normal;
    vec4 texcoord;
};

layout(std430, set = 1, binding = 0) readonly buffer vertexBuffer {
    Vertex vertices[];
};

layout(location = 0) out vec3 fragColor;

void main() {
    Vertex vertex = vertices[gl_VertexIndex];
    vec3 pos = vertex.position.xyz;
    gl_Position = vec4(pos, 1.0) * viewProj;
    fragColor = vertex.normal.xyz * 0.5 + 0.5;
}
