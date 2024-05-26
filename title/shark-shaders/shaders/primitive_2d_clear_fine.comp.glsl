#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

#include "primitive_2d.h"

layout (local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

void main() {
    if (gl_GlobalInvocationID.x < (primitive_uniforms.tile_resolution_fine.x * primitive_uniforms.tile_resolution_fine.y)) {
        fine_count_wo[gl_GlobalInvocationID.x] = 0;
    }
}
