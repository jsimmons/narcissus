#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_scalar_block_layout : require

#include "compute_bindings.h"
#include "primitive_2d.h"

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
    const vec3 stimulus = imageLoad(color_layer, ivec2(gl_GlobalInvocationID.xy)).rgb;
    const vec3 transformed = tony_mc_mapface(stimulus);
    vec3 composited = srgb_oetf(transformed);

    const uvec2 tile_coord = gl_WorkGroupID.xy / 4;
    const uint tile_index = tile_coord.y * uniforms.tile_stride + tile_coord.x;
    const uint tile_base = tile_index * TILE_STRIDE;

    TilesRead tiles_read = TilesRead(uniforms.tiles);

    const uint first = tiles_read.values[tile_base + TILE_BITMAP_RANGE_OFFSET + 0];
    const uint last = tiles_read.values[tile_base + TILE_BITMAP_RANGE_OFFSET + 1];
    if (first <= last) {
        const vec4 ui = imageLoad(ui_layer_read, ivec2(gl_GlobalInvocationID.xy)).rgba;
        composited = ui.rgb + (composited * (1.0 - ui.a));
    }

    imageStore(composited_output, ivec2(gl_GlobalInvocationID.xy), vec4(composited, 1.0));
}
