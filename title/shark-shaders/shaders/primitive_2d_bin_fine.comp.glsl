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
    const uvec2 tile_coord = gl_GlobalInvocationID.yz;
    const uvec2 tile_min = tile_coord * TILE_SIZE_FINE;
    const uvec2 tile_max = min(tile_min + TILE_SIZE_FINE, primitive_uniforms.screen_resolution);
    const uint tile_index = tile_coord.y * primitive_uniforms.tile_stride_fine + tile_coord.x;

    const uint index = gl_WorkGroupID.x * gl_WorkGroupSize.x + gl_SubgroupID * gl_SubgroupSize + gl_SubgroupInvocationID;

    uint bitmap_l0 = 0;
    if (index < primitive_uniforms.num_primitives_32) {
        const uvec2 tile_coord_coarse = tile_coord >> TILE_SIZE_SHIFT;
        const uint tile_index_coarse = tile_coord_coarse.y * primitive_uniforms.tile_stride_coarse + tile_coord_coarse.x;
        const uint tile_base_coarse = tile_index_coarse * TILE_STRIDE_COARSE;
        const uint tile_bitmap_base_coarse = tile_base_coarse + TILE_BITMAP_OFFSET_COARSE;

        uint bitmap_coarse = coarse_bitmap_ro[tile_bitmap_base_coarse + index];
        while (bitmap_coarse != 0) {
            const uint i = findLSB(bitmap_coarse);
            bitmap_coarse ^= bitmap_coarse & -bitmap_coarse;

            const uint primitive_index = index * 32 + i;
            if (test_glyph(primitive_index, tile_min, tile_max)) {
                bitmap_l0 |= 1 << i;
            }
        }
    }

    const uint tile_base_fine = tile_index * TILE_STRIDE_FINE;

    // Write the L0 per-primitive bitmap.
    const uint tile_bitmap_l0_base_fine = tile_base_fine + TILE_BITMAP_L0_OFFSET_FINE;
    fine_bitmap_wo[tile_bitmap_l0_base_fine + index] = bitmap_l0;

    // Write the L1 per-bitmap-word bitmap.
    uvec4 ballot_result = subgroupBallot(bitmap_l0 != 0);
    if (subgroupElect()) {
        const uint tile_bitmap_l1_base_fine = tile_base_fine + TILE_BITMAP_L1_OFFSET_FINE;
        fine_bitmap_wo[tile_bitmap_l1_base_fine + 2 * gl_WorkGroupID.x + 0] = ballot_result.x;
        fine_bitmap_wo[tile_bitmap_l1_base_fine + 2 * gl_WorkGroupID.x + 1] = ballot_result.y;

        const uint count = uint(ballot_result.x != 0) + uint(ballot_result.y != 0);
        if (count != 0) {
            atomicAdd(fine_count_wo[tile_index], count);
        }
    }
}
