use rustc_hash::FxHashMap;
use stb_truetype_sys::rectpack::Rect;

use crate::{font::GlyphBitmapBox, FontCollection, GlyphIndex, Oversample, Packer};

pub use narcissus_core::FiniteF32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CachedGlyphIndex(u32);

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct CachedGlyph {
    // Bitmap coordinates in texture atlas.
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,

    // Glyph bounding box relative to glyph origin.
    pub offset_x0: f32,
    pub offset_x1: f32,
    pub offset_y0: f32,
    pub offset_y1: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey<Family> {
    family: Family,
    glyph_index: GlyphIndex,
    scale: FiniteF32,
}

pub struct GlyphCache<'a, F>
where
    F: FontCollection<'a>,
{
    fonts: &'a F,

    oversample_h: Oversample,
    oversample_v: Oversample,

    next_cached_glyph_index: u32,
    glyph_lookup: FxHashMap<GlyphKey<F::Family>, CachedGlyphIndex>,

    cached_glyphs: Vec<CachedGlyph>,

    width: usize,
    height: usize,
    texture: Box<[u8]>,
}

const GLYPH_CACHE_PADDING: usize = 1;

impl<'a, F> GlyphCache<'a, F>
where
    F: FontCollection<'a>,
{
    pub fn new(
        fonts: &'a F,
        width: usize,
        height: usize,
        oversample_h: Oversample,
        oversample_v: Oversample,
    ) -> Self {
        Self {
            fonts,
            oversample_h,
            oversample_v,

            next_cached_glyph_index: 0,
            glyph_lookup: Default::default(),

            cached_glyphs: Vec::new(),

            width,
            height,
            texture: vec![0; width * height].into_boxed_slice(),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cache_glyph(
        &mut self,
        family: F::Family,
        glyph_index: GlyphIndex,
        scale: f32,
    ) -> CachedGlyphIndex {
        let key = GlyphKey {
            family,
            glyph_index,
            scale: FiniteF32::new(scale).unwrap(),
        };

        *self.glyph_lookup.entry(key).or_insert_with(|| {
            let cached_glyph_index = CachedGlyphIndex(self.next_cached_glyph_index);
            self.next_cached_glyph_index += 1;
            cached_glyph_index
        })
    }

    pub fn update_atlas(&mut self) -> (&[CachedGlyph], &[u8]) {
        self.next_cached_glyph_index = 0;

        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct GlyphToRender<F> {
            family: F,
            glyph_index: GlyphIndex,
            scale: FiniteF32,
            cached_glyph_index: CachedGlyphIndex,
        }

        let mut glyphs_to_render = self
            .glyph_lookup
            .iter()
            .map(|(glyph_key, &cached_glyph_index)| GlyphToRender {
                family: glyph_key.family,
                glyph_index: glyph_key.glyph_index,
                scale: glyph_key.scale,
                cached_glyph_index,
            })
            .collect::<Vec<_>>();

        glyphs_to_render.sort_unstable();

        let padding = GLYPH_CACHE_PADDING as i32;
        let oversample_h = self.oversample_h.as_i32();
        let oversample_v = self.oversample_v.as_i32();

        let mut rects = glyphs_to_render
            .iter()
            .map(|glyph| {
                let scale = glyph.scale.get();

                let bitmap_box = self.fonts.font(glyph.family).glyph_bitmap_box(
                    glyph.glyph_index,
                    scale * oversample_h as f32,
                    scale * oversample_v as f32,
                    0.0,
                    0.0,
                );

                let w = bitmap_box.x1 - bitmap_box.x0 + padding + oversample_h - 1;
                let h = bitmap_box.y1 - bitmap_box.y0 + padding + oversample_v - 1;

                Rect {
                    id: glyph.cached_glyph_index.0 as i32,
                    w,
                    h,
                    x: 0,
                    y: 0,
                    was_packed: 0,
                }
            })
            .collect::<Vec<_>>();

        let mut packer = Packer::new(
            self.width - GLYPH_CACHE_PADDING,
            self.height - GLYPH_CACHE_PADDING,
        );

        packer.pack(rects.as_mut_slice());

        self.texture.fill(0);
        self.cached_glyphs
            .resize(glyphs_to_render.len(), CachedGlyph::default());

        let oversample_h = oversample_h as f32;
        let oversample_v = oversample_v as f32;

        for (glyph, rect) in glyphs_to_render.iter().zip(rects.iter_mut()) {
            let font = self.fonts.font(glyph.family);

            // Pad on left and top.
            rect.x += padding;
            rect.y += padding;
            rect.w -= padding;
            rect.h -= padding;

            let scale = glyph.scale.get();
            let scale_x = scale * oversample_h;
            let scale_y = scale * oversample_v;

            let (sub_x, sub_y) = font.render_glyph_bitmap(
                &mut self.texture,
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                self.width as i32,
                scale_x,
                scale_y,
                0.0,
                0.0,
                self.oversample_h,
                self.oversample_v,
                glyph.glyph_index,
            );

            let cached_glyph = &mut self.cached_glyphs[rect.id as usize];

            cached_glyph.x0 = rect.x;
            cached_glyph.x1 = rect.x + rect.w;
            cached_glyph.y0 = rect.y;
            cached_glyph.y1 = rect.y + rect.h;

            let GlyphBitmapBox {
                x0,
                x1: _,
                y0,
                y1: _,
            } = font.glyph_bitmap_box(
                glyph.glyph_index,
                scale * oversample_h,
                scale * oversample_v,
                0.0,
                0.0,
            );

            cached_glyph.offset_x0 = x0 as f32 / oversample_h + sub_x;
            cached_glyph.offset_y0 = y0 as f32 / oversample_v + sub_y;
            cached_glyph.offset_x1 = (x0 + rect.w) as f32 / oversample_h + sub_x;
            cached_glyph.offset_y1 = (y0 + rect.h) as f32 / oversample_v + sub_y;
        }

        (&self.cached_glyphs, &self.texture)
    }
}
