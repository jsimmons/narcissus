#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

#include "primitive_2d_constants.h"
#include "primitive_2d_types.h"

layout(std430, set = 0, binding = 0) uniform uniformBuffer {
    uint screen_width;
    uint screen_height;
    uint atlas_width;
    uint atlas_height;
    uint num_primitives;
};

layout (set = 0, binding = 1) uniform sampler bilinear_sampler;
layout (set = 0, binding = 2) uniform texture2D glyph_atlas;

layout(std430, set = 0, binding = 3) readonly buffer glyphBuffer {
    Glyph glyphs[];
};

layout(std430, set = 0, binding = 4) readonly buffer glyphInstanceBuffer {
    GlyphInstance glyph_instances[];
};

layout(std430, set = 0, binding = 5) readonly buffer primitiveInstanceBuffer {
    PrimitiveInstance primitive_instances[];
};

layout(std430, set = 0, binding = 6) readonly buffer tileBuffer {
    uint tile_bitmap[];
};

layout (set = 0, binding = 7, rgba16f) uniform writeonly image2D ui_image;

layout (local_size_x = 16, local_size_y = 16, local_size_z = 1) in;

void main() {
    vec4 accum = vec4(0.0);

    const ivec2 tile_coord = ivec2(gl_WorkGroupID.xy);
    const int tile_index = tile_coord.y * MAX_TILES_X + tile_coord.x;
    const uint bitmap_base_offset = uint(tile_index * TILE_STRIDE);

    for (int i = 0; i < num_primitives / 32; i++) {
        uint bitmap = tile_bitmap[bitmap_base_offset + i];
        while (bitmap != 0) {
            const uint t = bitmap & -bitmap;
            const int index = i * 32 + findLSB(bitmap);
            bitmap ^= t;

            const GlyphInstance gi = glyph_instances[index];
            const Glyph gl = glyphs[gi.index];
            const vec4 color = unpackUnorm4x8(gi.color).bgra;
            const vec2 glyph_min = gi.position + gl.offset_min;
            const vec2 glyph_max = gi.position + gl.offset_max;
            const vec2 glyph_size = gl.offset_max - gl.offset_min;
            const vec2 sample_center = gl_GlobalInvocationID.xy; // half pixel offset goes here?
            if (all(greaterThanEqual(sample_center, glyph_min)) && all(lessThanEqual(sample_center, glyph_max))) {
                const vec2 uv = mix(vec2(gl.atlas_min), vec2(gl.atlas_max), (sample_center - glyph_min) / glyph_size) / vec2(atlas_width, atlas_height);
                const float coverage = textureLod(sampler2D(glyph_atlas, bilinear_sampler), uv, 0.0).r * color.a;
                accum.rgb = (coverage * color.rgb) + accum.rgb * (1.0 - coverage);
                accum.a = coverage + accum.a * (1.0 - coverage);
            }
        }
    }

    imageStore(ui_image, ivec2(gl_GlobalInvocationID.xy), accum);
}
