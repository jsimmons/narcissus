#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

#extension GL_KHR_shader_subgroup_arithmetic : require
#extension GL_KHR_shader_subgroup_ballot : require
#extension GL_KHR_shader_subgroup_vote : require

#include "compute_bindings.h"
#include "primitive_2d.h"

const uint SUBGROUP_SIZE = 64;
const uint NUM_SUBGROUPS = 16;
const uint NUM_PRIMITIVES_WG = (SUBGROUP_SIZE * NUM_SUBGROUPS);

// TODO: Spec constant support for different subgroup sizes.
layout (local_size_x = SUBGROUP_SIZE, local_size_y = 1, local_size_z = 1) in;

void main() {
    uint word_index = 0;

    for (uint i = 0; i < NUM_PRIMITIVES_WG; i += gl_SubgroupSize.x) {
        const uint primitive_index = gl_WorkGroupID.x * NUM_PRIMITIVES_WG + i + gl_SubgroupInvocationID;

        vec2 primitive_min = vec2(99999.9);
        vec2 primitive_max = vec2(-99999.9);

        if (primitive_index < uniforms.num_primitives) {
            const GlyphInstance gi = uniforms.glyph_instances.values[primitive_index];
            const Glyph gl = uniforms.glyphs.values[gi.index];
            primitive_min = gi.position + gl.offset_min;
            primitive_max = gi.position + gl.offset_max;
        }

        const vec2 primitives_min = subgroupMin(primitive_min);
        const vec2 primitives_max = subgroupMax(primitive_max);

        if (any(greaterThan(primitives_min, uniforms.screen_resolution)) || any(lessThan(primitives_max, vec2(0.0)))) {
            word_index += 2;
            continue;
        }

        const ivec2 tile_start = ivec2(floor(max(min(primitives_min, uniforms.screen_resolution), 0.0) / TILE_SIZE));
        const ivec2 tile_end = ivec2(floor((max(min(primitives_max, uniforms.screen_resolution), 0.0) + (TILE_SIZE - 1)) / TILE_SIZE));

        for (int y = tile_start.y; y < tile_end.y; y++) {
            for (int x = tile_start.x; x < tile_end.x; x++) {
                const uvec2 tile_coord = uvec2(x, y);
                const uint tile_index = tile_coord.y * uniforms.tile_stride + tile_coord.x;
                const vec2 tile_min = tile_coord * TILE_SIZE;
                const vec2 tile_max = min(tile_min + TILE_SIZE, uniforms.screen_resolution);

                const bool intersects = !(any(lessThan(tile_max, primitive_min)) || any(greaterThan(tile_min, primitive_max)));
                const uvec4 ballot = subgroupBallot(intersects);

                if (ballot.x == 0 && ballot.y == 0) {
                    continue;
                }

                if (ballot.x != 0) {
                    uniforms.tiles.values[tile_index * TILE_STRIDE + TILE_BITMAP_L0_OFFSET + gl_WorkGroupID.x * 32 + word_index + 0] = ballot.x;
                }

                if (ballot.y != 0) {
                    uniforms.tiles.values[tile_index * TILE_STRIDE + TILE_BITMAP_L0_OFFSET + gl_WorkGroupID.x * 32 + word_index + 1] = ballot.y;
                }

                if (subgroupElect()) {
                    uniforms.tiles.values[tile_index * TILE_STRIDE + TILE_BITMAP_L1_OFFSET + gl_WorkGroupID.x] |=
                        (uint(ballot.x != 0) << (word_index + 0)) |
                        (uint(ballot.y != 0) << (word_index + 1));

                    atomicMin(uniforms.tiles.values[tile_index * TILE_STRIDE + TILE_BITMAP_RANGE_LO_OFFSET], gl_WorkGroupID.x);
                    atomicMax(uniforms.tiles.values[tile_index * TILE_STRIDE + TILE_BITMAP_RANGE_HI_OFFSET], gl_WorkGroupID.x);
                }
            }
        }

        word_index += 2;
    }
}
