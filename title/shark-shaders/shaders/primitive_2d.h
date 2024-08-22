#ifndef PRIMITIVE_2D_INCLUDE
#define PRIMITIVE_2D_INCLUDE

const uint TILE_SIZE = 32;

const uint MAX_PRIMS = 1 << 18;
const uint TILE_BITMAP_L1_WORDS = (MAX_PRIMS / 32 / 32);
const uint TILE_BITMAP_L0_WORDS = (MAX_PRIMS / 32);
const uint TILE_STRIDE = (TILE_BITMAP_L0_WORDS + TILE_BITMAP_L1_WORDS + 2);
const uint TILE_BITMAP_RANGE_LO_OFFSET = 0;
const uint TILE_BITMAP_RANGE_HI_OFFSET = (TILE_BITMAP_RANGE_LO_OFFSET + 1);
const uint TILE_BITMAP_L1_OFFSET = (TILE_BITMAP_RANGE_HI_OFFSET + 1);
const uint TILE_BITMAP_L0_OFFSET = (TILE_BITMAP_L1_OFFSET + TILE_BITMAP_L1_WORDS);

const uint PRIMITIVE_TYPE_RECT = 0;
const uint PRIMITIVE_TYPE_GLYPH = 1;

struct PrimitiveInstance {
    uint packed;
    uint color;
    vec2 position;
};

struct Rect {
    vec2 half_extent;
    float border_width;
    float border_radius;
};

struct Glyph {
    ivec2 atlas_min;
    ivec2 atlas_max;

    vec2 offset_min;
    vec2 offset_max;
};

#endif