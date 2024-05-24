#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

#extension GL_KHR_shader_subgroup_vote : require
#extension GL_KHR_shader_subgroup_ballot : require

#include "primitive_2d.h"

// TODO: Spec constant support for different subgroup sizes.
layout (local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

void main() {
    const uvec2 tile_size = uvec2(TILE_SIZE_FINE, TILE_SIZE_FINE);
    const uvec2 tile_coord = gl_GlobalInvocationID.yz;
    const uvec2 tile_min = tile_coord * tile_size;
    const uvec2 tile_max = min(tile_min + tile_size, primitive_uniforms.screen_resolution);
    const uint tile_index = tile_coord.y * primitive_uniforms.tile_resolution_fine.x + tile_coord.x;

    const uint bitmap_index = gl_WorkGroupID.x * gl_WorkGroupSize.x + gl_SubgroupID * gl_SubgroupSize + gl_SubgroupInvocationID;

    uint bitmap_l0 = 0;
    if (bitmap_index < primitive_uniforms.num_primitives_32) {
        const uvec2 tile_coord_coarse = tile_coord >> TILE_SIZE_SHIFT;
        const uint tile_index_coarse = tile_coord_coarse.y * primitive_uniforms.tile_resolution_coarse.x + tile_coord_coarse.x;
        const uint bitmap_offset_coarse = tile_index_coarse * TILE_STRIDE_COARSE + bitmap_index;

        uint bitmap_coarse = coarse_bitmap_ro[bitmap_offset_coarse];
        while (bitmap_coarse != 0) {
            const uint i = findLSB(bitmap_coarse);
            const uint primitive_index = bitmap_index * 32 + i;
            bitmap_coarse ^= bitmap_coarse & -bitmap_coarse;

            if (test_glyph(primitive_index, tile_min, tile_max)) {
                bitmap_l0 |= 1 << i;
            }
        }
    }

    const uint fine_bitmap_l0_offset = tile_index * TILE_STRIDE_FINE + TILE_BITMAP_WORDS_L1 + bitmap_index;
    fine_bitmap_wo[fine_bitmap_l0_offset] = bitmap_l0;

    const bool bit_l1 = bitmap_l0 != 0;
    uvec4 ballot_result = subgroupBallot(bit_l1);
    if (subgroupElect()) {
        const uint fine_bitmap_l1_offset = tile_index * TILE_STRIDE_FINE;
        fine_bitmap_wo[fine_bitmap_l1_offset + 2 * gl_WorkGroupID.x + 0] = ballot_result.x;
        fine_bitmap_wo[fine_bitmap_l1_offset + 2 * gl_WorkGroupID.x + 1] = ballot_result.y;
    }
}
