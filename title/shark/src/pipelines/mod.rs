use narcissus_font::TouchedGlyphIndex;

pub mod basic;

pub const TILE_SIZE: u32 = 32;
pub const MAX_PRIMS: u32 = 1 << 18;
pub const TILE_BITMAP_WORDS_L1: u32 = MAX_PRIMS / 32 / 32;
pub const TILE_BITMAP_WORDS_L0: u32 = MAX_PRIMS / 32;
pub const TILE_STRIDE: u32 = TILE_BITMAP_WORDS_L0 + TILE_BITMAP_WORDS_L1 + 2;

#[allow(unused)]
#[repr(C)]
pub struct PrimitiveUniforms {
    pub screen_resolution_x: u32,
    pub screen_resolution_y: u32,

    pub atlas_resolution_x: u32,
    pub atlas_resolution_y: u32,

    pub tile_resolution_x: u32,
    pub tile_resolution_y: u32,

    pub num_primitives: u32,
    pub num_primitives_32: u32,
    pub num_primitives_1024: u32,
    pub tile_stride: u32,

    pub glyphs_buffer: u64,
    pub glyph_instances_buffer: u64,
    pub tiles_buffer: u64,
}

#[allow(unused)]
#[repr(C)]
pub struct GlyphInstance {
    pub x: f32,
    pub y: f32,
    pub touched_glyph_index: TouchedGlyphIndex,
    pub color: u32,
}
