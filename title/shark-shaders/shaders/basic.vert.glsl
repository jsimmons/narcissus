#version 460

#extension GL_EXT_scalar_block_layout : require

struct VertexData {
    vec4 position;
    vec4 normal;
    vec4 texcoord;
};

struct TransformData {
    vec4 transform[3];
};

layout(std430, row_major, set = 0, binding = 0) uniform uniformBuffer {
    mat4 clip_from_camera;
};

layout(std430, set = 1, binding = 0) readonly buffer vertexBuffer {
    VertexData vertices[];
};

layout(std430, set = 1, binding = 1) readonly buffer transformBuffer {
    TransformData transforms[];
};

layout(location = 0) out vec2 out_texcoord;
layout(location = 1) out vec3 out_normal;

void main() {
    const TransformData td = transforms[gl_InstanceIndex];
    const VertexData vd = vertices[gl_VertexIndex];

    const mat4 camera_from_model = mat4(
        td.transform[0].x, td.transform[0].w, td.transform[1].z, 0.0,
        td.transform[0].y, td.transform[1].x, td.transform[1].w, 0.0, 
        td.transform[0].z, td.transform[1].y, td.transform[2].x, 0.0, 
        td.transform[2].y, td.transform[2].z, td.transform[2].w, 1.0
    );

    const vec4 position_clip = clip_from_camera * camera_from_model * vec4(vd.position.xyz, 1.0);

    gl_Position = position_clip;

    out_normal = vd.normal.xyz;
    out_texcoord = vec2(vd.texcoord.x, 1.0 - vd.texcoord.y);
}
