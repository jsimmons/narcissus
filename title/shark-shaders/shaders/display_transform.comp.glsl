#version 460

#extension GL_EXT_control_flow_attributes : require

const uint MAX_PRIMS = 1 << 18;
const uint TILE_BITMAP_L1_WORDS = (MAX_PRIMS / 32 / 32);
const uint TILE_BITMAP_L0_WORDS = (MAX_PRIMS / 32);
const uint TILE_STRIDE = (TILE_BITMAP_L0_WORDS + TILE_BITMAP_L1_WORDS + 2);
const uint TILE_BITMAP_RANGE_OFFSET = 0;

struct PrimitiveUniforms {
    uvec2 screen_resolution;
    uvec2 atlas_resolution;

    uint num_primitives;
    uint num_primitives_32;
    uint num_primitives_1024;
    uint tile_stride;
};

layout(std430, push_constant) uniform uniformBuffer {
    PrimitiveUniforms primitive_uniforms; 
};

layout (set = 0, binding = 0) uniform sampler bilinear_sampler;

layout (set = 0, binding = 1) uniform texture3D tony_mc_mapface_lut;

layout(std430, set = 0, binding = 2) readonly buffer tileBufferRead {
    uint tile_bitmap_ro[];
};

layout (set = 0, binding = 3, rgba16f) uniform readonly image2D layer_rt;
layout (set = 0, binding = 4, rgba16f) uniform readonly image2D layer_ui;

layout (set = 0, binding = 5, rgba16f) uniform writeonly image2D composited_output;

float srgb_oetf(float a) {
    return (.0031308f >= a) ? 12.92f * a : 1.055f * pow(a, .4166666666666667f) - .055f;
}

vec3 srgb_oetf(vec3 a) {
	return vec3(srgb_oetf(a.r), srgb_oetf(a.g), srgb_oetf(a.b));
}

vec3 tony_mc_mapface(vec3 stimulus) {
    const vec3 encoded = stimulus / (stimulus + 1.0);
    const float LUT_DIMS = 48.0;
    const vec3 uv = (encoded * ((LUT_DIMS - 1.0) / LUT_DIMS) + 0.5 / LUT_DIMS);
    return textureLod(sampler3D(tony_mc_mapface_lut, bilinear_sampler), uv, 0.0).rgb;
}

layout (local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

void main() {
    const vec3 stimulus = imageLoad(layer_rt, ivec2(gl_GlobalInvocationID.xy)).rgb;
    const vec3 transformed = tony_mc_mapface(stimulus);
    vec3 composited = srgb_oetf(transformed);

    const uvec2 tile_coord = gl_WorkGroupID.xy / 4;
    const uint tile_index = tile_coord.y * primitive_uniforms.tile_stride + tile_coord.x;
    const uint tile_base = tile_index * TILE_STRIDE;

    const uint first = tile_bitmap_ro[tile_base + TILE_BITMAP_RANGE_OFFSET + 0];
    const uint last = tile_bitmap_ro[tile_base + TILE_BITMAP_RANGE_OFFSET + 1];
    if (first <= last) {
        const vec4 ui = imageLoad(layer_ui, ivec2(gl_GlobalInvocationID.xy)).rgba;
        composited = ui.rgb + (composited * (1.0 - ui.a));
    }

    imageStore(composited_output, ivec2(gl_GlobalInvocationID.xy), vec4(composited, 1.0));
}
