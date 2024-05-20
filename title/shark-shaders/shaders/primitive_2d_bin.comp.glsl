#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

#extension GL_KHR_shader_subgroup_vote : require
#extension GL_KHR_shader_subgroup_ballot : require

#include "primitive_2d_types.h"
#include "primitive_2d_constants.h"

layout(std430, set = 0, binding = 0) uniform uniformBuffer {
    ivec2 screen_size;
    ivec2 atlas_size;
    uint num_primitives;
};

layout(std430, set = 0, binding = 3) readonly buffer glyphBuffer {
    Glyph glyphs[];
};

layout(std430, set = 0, binding = 4) readonly buffer glyphInstanceBuffer {
    GlyphInstance glyph_instances[];
};

layout(std430, set = 0, binding = 5) readonly buffer primitiveInstanceBuffer {
    PrimitiveInstance primitive_instances[];
};

bool test_glyph(uint index, ivec2 tile_min, ivec2 tile_max) {
    const GlyphInstance gi = glyph_instances[index];
    const Glyph gl = glyphs[gi.index];
    const vec2 glyph_min = gi.position + gl.offset_min;
    const vec2 glyph_max = gi.position + gl.offset_max;
    return !(any(lessThan(tile_max, glyph_min)) || any(greaterThan(tile_min, glyph_max)));
}

layout(std430, set = 0, binding = 6) writeonly buffer tileBuffer {
    uint tile_bitmap[];
};

// TODO: Spec constant support for different subgroup sizes.
layout (local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

void main() {
    const ivec2 tile_size = ivec2(TILE_SIZE, TILE_SIZE);
    const ivec2 tile_coord = ivec2(gl_GlobalInvocationID.yz);
    const ivec2 tile_min = ivec2(tile_coord * tile_size);
    const ivec2 tile_max = min(tile_min + tile_size, screen_size);

    const uint local_index = gl_SubgroupID * gl_SubgroupSize + gl_SubgroupInvocationID;
    const uint primitive_index = gl_WorkGroupID.x * gl_WorkGroupSize.x + local_index;

    bool intersects = false;
    if (primitive_index < num_primitives) {
        intersects = test_glyph(primitive_index, tile_min, tile_max);
    }

    uvec4 ballot_result = subgroupBallot(intersects);
    if (subgroupElect()) { // managed democracy wins again
        const int tile_index = tile_coord.y * MAX_TILES_X + tile_coord.x;
        const uint bitmap_base_offset = uint(tile_index * TILE_STRIDE); 
        tile_bitmap[bitmap_base_offset + 2u * gl_WorkGroupID.x + 0u] = ballot_result.x;
        tile_bitmap[bitmap_base_offset + 2u * gl_WorkGroupID.x + 1u] = ballot_result.y;
    }
}
