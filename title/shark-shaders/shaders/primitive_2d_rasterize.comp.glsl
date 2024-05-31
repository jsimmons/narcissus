#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

#extension GL_KHR_shader_subgroup_vote : require
#extension GL_KHR_shader_subgroup_ballot : require

#include "primitive_2d.h"

layout (local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

#define DEBUG_SHOW_TILES 0

#if DEBUG_SHOW_TILES != 0

vec3 plasma_quintic(float x)
{
	x = clamp(x, 0.0, 1.0);
	vec4 x1 = vec4(1.0, x, x * x, x * x * x); // 1 x x2 x3
	vec4 x2 = x1 * x1.w * x; // x4 x5 x6 x7
	return vec3(
		dot(x1.xyzw, vec4(+0.063861086, +1.992659096, -1.023901152, -0.490832805)) + dot(x2.xy, vec2(+1.308442123, -0.914547012)),
		dot(x1.xyzw, vec4(+0.049718590, -0.791144343, +2.892305078, +0.811726816)) + dot(x2.xy, vec2(-4.686502417, +2.717794514)),
		dot(x1.xyzw, vec4(+0.513275779, +1.580255060, -5.164414457, +4.559573646)) + dot(x2.xy, vec2(-1.916810682, +0.570638854))
    );
}

#endif

void main() {
    const uvec2 tile_coord = gl_WorkGroupID.xy / 4;
    const uint tile_index = tile_coord.y * primitive_uniforms.tile_stride + tile_coord.x;
    const uint tile_base = tile_index * TILE_STRIDE;
    const uint tile_bitmap_l1_base_fine = tile_base + TILE_BITMAP_L1_OFFSET;
    const uint tile_bitmap_l0_base_fine = tile_base + TILE_BITMAP_L0_OFFSET;

#if DEBUG_SHOW_TILES == 1

    int count = 0;
    // For each tile, iterate over all words in the L1 bitmap.
    for (int index_l1 = 0; index_l1 < primitive_uniforms.num_primitives_1024; index_l1++) {
        // For each word, iterate all set bits.
        uint bitmap_l1 = tile_bitmap_ro[tile_bitmap_l1_base_fine + index_l1];

        while (bitmap_l1 != 0) {
            const uint i = findLSB(bitmap_l1);
            bitmap_l1 ^= bitmap_l1 & -bitmap_l1;

            // For each set bit in the L1 bitmap, iterate the set bits in the
            // corresponding L0 bitmap.
            const uint index_l0 = index_l1 * 32 + i;
            uint bitmap_l0 = tile_bitmap_ro[tile_bitmap_l0_base_fine + index_l0];

            count += bitCount(bitmap_l0);
        }
    }

    const vec3 color = plasma_quintic(float(count) / 100.0);
    imageStore(ui_image, ivec2(gl_GlobalInvocationID.xy), vec4(color, 1.0));

#else

    vec4 accum = vec4(0.0);

    // For each tile, iterate over all words in the L1 bitmap. 
    for (int index_l1 = 0; index_l1 < primitive_uniforms.num_primitives_1024; index_l1++) {
        // For each word, iterate all set bits.
        uint bitmap_l1 = tile_bitmap_ro[tile_bitmap_l1_base_fine + index_l1];

        while (bitmap_l1 != 0) {
            const uint i = findLSB(bitmap_l1);
            bitmap_l1 ^= bitmap_l1 & -bitmap_l1;

            // For each set bit in the L1 bitmap, iterate the set bits in the
            // corresponding L0 bitmap.
            const uint index_l0 = index_l1 * 32 + i;
            uint bitmap_l0 = tile_bitmap_ro[tile_bitmap_l0_base_fine + index_l0];
            while (bitmap_l0 != 0) {
                const uint j = findLSB(bitmap_l0);
                bitmap_l0 ^= bitmap_l0 & -bitmap_l0;

                // Set bits in the L0 bitmap indicate binned primitives for this tile.
                const uint primitive_index = index_l0 * 32 + j;

                const GlyphInstance gi = glyph_instances[primitive_index];
                const Glyph gl = glyphs[gi.index];
                const vec2 glyph_min = gi.position + gl.offset_min;
                const vec2 glyph_max = gi.position + gl.offset_max;
                const vec2 sample_center = gl_GlobalInvocationID.xy + vec2(0.5);
                [[branch]]
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

#endif
}
