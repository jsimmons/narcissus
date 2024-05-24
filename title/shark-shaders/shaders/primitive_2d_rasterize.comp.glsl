#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

#include "primitive_2d.h"

layout (local_size_x = TILE_SIZE_FINE, local_size_y = TILE_SIZE_FINE, local_size_z = 1) in;

void main() {
    const uvec2 tile_coord = gl_WorkGroupID.xy;
    const uint tile_index = tile_coord.y * primitive_uniforms.tile_resolution_fine.x + tile_coord.x;
    const uint tile_base_fine = tile_index * TILE_STRIDE_FINE;
    const uint tile_bitmap_l1_base_fine = tile_base_fine + TILE_BITMAP_L1_OFFSET_FINE;
    const uint tile_bitmap_l0_base_fine = tile_base_fine + TILE_BITMAP_L0_OFFSET_FINE;

    vec4 accum = vec4(0.0);

    // For each tile, iterate over all words in the L1 bitmap.
    //
    // TODO: Count the non-zero words in the tile with atomics, so we can early out on empty tiles? 
    for (int index_l1 = 0; index_l1 < primitive_uniforms.num_primitives_1024; index_l1++) {
        // For each word, iterate all set bits.
        uint bitmap_l1 = fine_bitmap_ro[tile_bitmap_l1_base_fine + index_l1];
        while (bitmap_l1 != 0) {
            const uint i = findLSB(bitmap_l1);
            bitmap_l1 ^= bitmap_l1 & -bitmap_l1;

            // For each set bit in the L1 bitmap, iterate the set bits in the
            // corresponding L0 bitmap.
            const uint index_l0 = index_l1 * 32 + i;
            uint bitmap_l0 = fine_bitmap_ro[tile_bitmap_l0_base_fine + index_l0];
            while (bitmap_l0 != 0) {
                const uint j = findLSB(bitmap_l0);
                bitmap_l0 ^= bitmap_l0 & -bitmap_l0;

                // Set bits in the L0 bitmap indicate binned primitives for this tile.
                const uint primitive_index = index_l0 * 32 + j;

                const GlyphInstance gi = glyph_instances[primitive_index];
                const Glyph gl = glyphs[gi.index];
                const vec2 glyph_min = gi.position + gl.offset_min;
                const vec2 glyph_max = gi.position + gl.offset_max;
                const vec2 sample_center = gl_GlobalInvocationID.xy; // half pixel offset goes here?
                if (all(greaterThanEqual(sample_center, glyph_min)) && all(lessThanEqual(sample_center, glyph_max))) {
                    const vec2 glyph_size = gl.offset_max - gl.offset_min;
                    const vec4 color = unpackUnorm4x8(gi.color).bgra;
                    const vec2 uv = mix(gl.atlas_min, gl.atlas_max, (sample_center - glyph_min) / glyph_size) / primitive_uniforms.atlas_resolution;
                    const float coverage = textureLod(sampler2D(glyph_atlas, bilinear_sampler), uv, 0.0).r * color.a;
                    accum.rgb = (coverage * color.rgb) + accum.rgb * (1.0 - coverage);
                    accum.a = coverage + accum.a * (1.0 - coverage);
                }
            }
        }
    }

    imageStore(ui_image, ivec2(gl_GlobalInvocationID.xy), accum);
}
