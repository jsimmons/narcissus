#define TILE_SIZE 32

#define MAX_PRIMS (1 << 18)
#define TILE_BITMAP_L1_WORDS (MAX_PRIMS / 32 / 32)
#define TILE_BITMAP_L0_WORDS (MAX_PRIMS / 32)
#define TILE_STRIDE (TILE_BITMAP_L0_WORDS + TILE_BITMAP_L1_WORDS)
#define TILE_BITMAP_L1_OFFSET 0
#define TILE_BITMAP_L0_OFFSET TILE_BITMAP_L1_WORDS

struct PrimitiveUniforms {
    uvec2 screen_resolution;
    uvec2 atlas_resolution;

    uint num_primitives;
    uint num_primitives_32;
    uint num_primitives_1024;
    uint tile_stride;
};

struct Glyph {
    ivec2 atlas_min;
    ivec2 atlas_max;

    vec2 offset_min;
    vec2 offset_max;
};

struct GlyphInstance {
    vec2 position;
    uint index;
    uint color;
};

#include "primitive_2d_bindings.h"

bool test_glyph(uint index, uvec2 tile_min, uvec2 tile_max) {
    const GlyphInstance gi = glyph_instances[index];
    const Glyph gl = glyphs[gi.index];
    const vec2 glyph_min = gi.position + gl.offset_min;
    const vec2 glyph_max = gi.position + gl.offset_max;
    return !(any(lessThan(tile_max, glyph_min)) || any(greaterThan(tile_min, glyph_max)));
}
