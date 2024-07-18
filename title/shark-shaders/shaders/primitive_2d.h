const uint TILE_SIZE = 32;

const uint MAX_PRIMS = 1 << 18;
const uint TILE_BITMAP_L1_WORDS = (MAX_PRIMS / 32 / 32);
const uint TILE_BITMAP_L0_WORDS = (MAX_PRIMS / 32);
const uint TILE_STRIDE = (TILE_BITMAP_L0_WORDS + TILE_BITMAP_L1_WORDS + 2);
const uint TILE_BITMAP_RANGE_LO_OFFSET = 0;
const uint TILE_BITMAP_RANGE_HI_OFFSET = (TILE_BITMAP_RANGE_LO_OFFSET + 1);
const uint TILE_BITMAP_L1_OFFSET = (TILE_BITMAP_RANGE_HI_OFFSET + 1);
const uint TILE_BITMAP_L0_OFFSET = (TILE_BITMAP_L1_OFFSET + TILE_BITMAP_L1_WORDS);

bool test_glyph(uint index, vec2 tile_min, vec2 tile_max) {
    const GlyphInstance gi = uniforms.glyph_instances.values[index];
    const Glyph gl = uniforms.glyphs.values[gi.index];
    const vec2 glyph_min = gi.position + gl.offset_min;
    const vec2 glyph_max = gi.position + gl.offset_max;
    return !(any(lessThan(tile_max, glyph_min)) || any(greaterThan(tile_min, glyph_max)));
}
