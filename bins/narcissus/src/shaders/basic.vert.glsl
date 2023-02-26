#version 460

struct VertexData {
    vec4 position;
    vec4 normal;
    vec4 texcoord;
};

struct TransformData {
    vec4 transform[3];
};

layout(set = 0, binding = 0) uniform uniformBuffer {
    mat4 viewProj;
};

layout(std430, set = 1, binding = 0) readonly buffer vertexBuffer {
    VertexData vertices[];
};

layout(std430, set = 1, binding = 1) readonly buffer transformBuffer {
    TransformData transforms[];
};

layout(location = 0) out vec2 outTexcoord;
layout(location = 1) out vec3 outNormal;

void main() {
    TransformData td = transforms[gl_InstanceIndex];
    VertexData vd = vertices[gl_VertexIndex];

    mat3 modelRot = mat3(
        td.transform[0].x, td.transform[0].y, td.transform[0].z,
        td.transform[0].w, td.transform[1].x, td.transform[1].y,
        td.transform[1].z, td.transform[1].w, td.transform[2].x
    );
    vec3 modelOff = vec3(td.transform[2].y, td.transform[2].z, td.transform[2].w);
    vec3 posWorld = transpose(modelRot) * vd.position.xyz + modelOff;
    vec4 posClip = transpose(viewProj) * vec4(posWorld, 1.0);
    gl_Position = posClip;

    outNormal = vd.normal.xyz;
    outTexcoord = vec2(vd.texcoord.x, 1.0 - vd.texcoord.y);
}
