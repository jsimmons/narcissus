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
    tile_bitmap_wo[gl_GlobalInvocationID.x * TILE_STRIDE + TILE_BITMAP_RANGE_OFFSET + 0] = 0xffffffff;
    tile_bitmap_wo[gl_GlobalInvocationID.x * TILE_STRIDE + TILE_BITMAP_RANGE_OFFSET + 1] = 0;
}
