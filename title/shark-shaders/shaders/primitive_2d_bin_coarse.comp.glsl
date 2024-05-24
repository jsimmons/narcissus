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
    const uvec2 tile_size = uvec2(TILE_SIZE_COARSE, TILE_SIZE_COARSE);
    const uvec2 tile_coord = gl_GlobalInvocationID.yz;
    const uvec2 tile_min = tile_coord * tile_size;
    const uvec2 tile_max = min(tile_min + tile_size, primitive_uniforms.screen_resolution);

    const uint primitive_index = gl_WorkGroupID.x * gl_WorkGroupSize.x + gl_SubgroupID * gl_SubgroupSize + gl_SubgroupInvocationID;

    bool intersects = false;
    if (primitive_index < primitive_uniforms.num_primitives) {
        intersects = test_glyph(primitive_index, tile_min, tile_max);
    }

    uvec4 ballot_result = subgroupBallot(intersects);
    if (subgroupElect()) { // managed democracy wins again
        const uint tile_index = tile_coord.y * primitive_uniforms.tile_resolution_coarse.x + tile_coord.x;
        const uint bitmap_offset = tile_index * TILE_STRIDE_COARSE;
        coarse_bitmap_wo[bitmap_offset + 2 * gl_WorkGroupID.x + 0] = ballot_result.x;
        coarse_bitmap_wo[bitmap_offset + 2 * gl_WorkGroupID.x + 1] = ballot_result.y;
    }
}
