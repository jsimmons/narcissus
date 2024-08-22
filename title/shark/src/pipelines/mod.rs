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

    pub primitives_instances_buffer: u64,
    pub rects_buffer: u64,
    pub glyphs_buffer: u64,
    pub tiles_buffer: u64,
}

#[repr(u32)]
pub enum PrimitiveType {
    Rect,
    Glyph,
}

#[allow(unused)]
#[repr(C)]
pub struct PrimitiveInstance {
    pub packed: u32,
    pub color: u32,
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct Rect {
    pub half_extent_x: f32,
    pub half_extent_y: f32,
    pub border_width: f32,
    pub border_radius: f32,
}

impl PrimitiveInstance {
    #[inline(always)]
    pub fn glyph(glyph_index: TouchedGlyphIndex, color: u32, x: f32, y: f32) -> Self {
        let packed = glyph_index.as_u32() | ((PrimitiveType::Glyph as u32) << 30);
        Self {
            packed,
            color,
            x,
            y,
        }
    }

    #[inline(always)]
    pub fn rect(rect_index: u32, color: u32, x: f32, y: f32) -> Self {
        let packed = rect_index | ((PrimitiveType::Rect as u32) << 30);
        Self {
            packed,
            color,
            x,
            y,
        }
    }
}
