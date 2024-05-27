#define MAX_PRIMS 0x20000u
#define TILE_SIZE_COARSE 64
#define TILE_SIZE_FINE 16
#define TILE_SIZE_MUL (TILE_SIZE_COARSE / TILE_SIZE_FINE)
#define TILE_BITMAP_L1_WORDS (MAX_PRIMS / 32 / 32)
#define TILE_BITMAP_L0_WORDS (MAX_PRIMS / 32)
#define TILE_STRIDE_COARSE TILE_BITMAP_L0_WORDS
#define TILE_STRIDE_FINE (TILE_BITMAP_L0_WORDS + TILE_BITMAP_L1_WORDS)
#define TILE_BITMAP_OFFSET_COARSE 0
#define TILE_BITMAP_L1_OFFSET_FINE 0
#define TILE_BITMAP_L0_OFFSET_FINE TILE_BITMAP_L1_WORDS

#define TILE_DISPATCH_X 15

struct PrimitiveUniforms {
    uvec2 screen_resolution;
    uvec2 atlas_resolution;

    uint num_primitives;
    uint num_primitives_32;
    uint num_primitives_1024;
    uint tile_stride_fine;

    uvec2 tile_offset_coarse;
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
