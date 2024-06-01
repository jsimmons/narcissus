#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

#extension GL_KHR_shader_subgroup_vote : require
#extension GL_KHR_shader_subgroup_ballot : require

#include "primitive_2d.h"

#define SUBGROUP_SIZE 64
#define NUM_PRIMS_WG (SUBGROUP_SIZE * 32)

// TODO: Spec constant support for different subgroup sizes.
layout (local_size_x = SUBGROUP_SIZE, local_size_y = 1, local_size_z = 1) in;

shared uint bitmap_0[SUBGROUP_SIZE];

void main() {
    const uvec2 bin_coord = gl_GlobalInvocationID.yz;
    const uvec2 bin_min = bin_coord * TILE_SIZE * 8;
    const uvec2 bin_max = min(bin_min + TILE_SIZE * 8, primitive_uniforms.screen_resolution);

    for (uint i = 0; i < NUM_PRIMS_WG; i += gl_SubgroupSize.x) {
        const uint prim_index = gl_WorkGroupID.x * NUM_PRIMS_WG + i + gl_SubgroupInvocationID;
        bool intersects = false;
        if (prim_index < primitive_uniforms.num_primitives) {
            const GlyphInstance gi = glyph_instances[prim_index];
            const Glyph gl = glyphs[gi.index];
            const vec2 glyph_min = gi.position + gl.offset_min;
            const vec2 glyph_max = gi.position + gl.offset_max;
            intersects = !(any(lessThan(bin_max, glyph_min)) || any(greaterThan(bin_min, glyph_max)));
        }
        const uvec4 ballot = subgroupBallot(intersects);
        bitmap_0[i / 32 + 0] = ballot.x;
        bitmap_0[i / 32 + 1] = ballot.y;
    }

    memoryBarrierShared();

    const uint x = gl_SubgroupInvocationID.x & 7;
    const uint y = gl_SubgroupInvocationID.x >> 3;
    const uvec2 tile_coord = gl_GlobalInvocationID.yz * 8 + uvec2(x, y);
    const uvec2 tile_min = tile_coord * TILE_SIZE;
    const uvec2 tile_max = min(tile_min + TILE_SIZE, primitive_uniforms.screen_resolution);

    if (all(lessThan(tile_min, tile_max))) {
        const uint tile_index = tile_coord.y * primitive_uniforms.tile_stride + tile_coord.x;

        for (uint i = 0; i < 2; i++) {
            uint out_1 = 0;

            for (uint j = 0; j < 32; j++) {
                uint out_0 = 0;
                uint index_0 = i * 32 + j;
                uint word_0 = bitmap_0[index_0];
                while (word_0 != 0) {
                    const uint bit_0 = findLSB(word_0);
                    word_0 ^= word_0 & -word_0;

                    const uint prim_index = gl_WorkGroupID.x * NUM_PRIMS_WG + index_0 * 32 + bit_0;
                    if (test_glyph(prim_index, tile_min, tile_max)) {
                        out_0 |= 1 << bit_0;
                    }
                }

                if (out_0 != 0) {
                    out_1 |= 1 << j;
                    tile_bitmap_wo[tile_index * TILE_STRIDE + TILE_BITMAP_L0_OFFSET + gl_WorkGroupID.x * 64 + index_0] = out_0;
                }
            }

            tile_bitmap_wo[tile_index * TILE_STRIDE + TILE_BITMAP_L1_OFFSET + gl_WorkGroupID.x * 2 + i] = out_1;
        }
    }
}
