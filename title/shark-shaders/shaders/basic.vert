#version 460

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_scalar_block_layout : require

struct Vertex {
    vec4 position;
    vec4 normal;
    vec4 texcoord;
};

struct Transform {
    vec4 transform[3];
};

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer VertexRef {
    Vertex values[];
};

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer TransformRef {
    Transform values[];
};

struct BasicConstants {
    mat4 clip_from_camera;
    VertexRef vertex_buffer;
    TransformRef transform_buffer;
};

layout(std430, row_major, push_constant) uniform BasicConstantsBlock {
    BasicConstants constants;
};

layout(location = 0) out vec2 out_texcoord;
layout(location = 1) out vec3 out_normal;

void main() {
    const Transform td = constants.transform_buffer.values[gl_InstanceIndex];
    const Vertex vd = constants.vertex_buffer.values[gl_VertexIndex];

    const mat4 camera_from_model = mat4(
        td.transform[0].x, td.transform[0].w, td.transform[1].z, 0.0,
        td.transform[0].y, td.transform[1].x, td.transform[1].w, 0.0, 
        td.transform[0].z, td.transform[1].y, td.transform[2].x, 0.0, 
        td.transform[2].y, td.transform[2].z, td.transform[2].w, 1.0
    );

    const vec4 position_clip = constants.clip_from_camera * camera_from_model * vec4(vd.position.xyz, 1.0);

    gl_Position = position_clip;

    out_normal = vd.normal.xyz;
    out_texcoord = vec2(vd.texcoord.x, 1.0 - vd.texcoord.y);
}
