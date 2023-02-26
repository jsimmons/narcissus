use rustc_hash::FxHashMap;
use stb_truetype_sys::rectpack::Rect;

use crate::{font::GlyphBitmapBox, FontCollection, GlyphIndex, Oversample, Packer};

pub use narcissus_core::FiniteF32;

/// An index into the CachedGlyph slice.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CachedGlyphIndex(u32);

/// Holds data required to draw a glyph from the glyph atlas.
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
    size_px: FiniteF32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Glyph<F> {
    family: F,
    glyph_index: GlyphIndex,
    size_px: FiniteF32,
    cached_glyph_index: CachedGlyphIndex,
}

pub struct GlyphCache<'a, F>
where
    F: FontCollection<'a>,
{
    fonts: &'a F,

    padding: usize,

    next_cached_glyph_index: u32,
    cached_glyph_lookup: FxHashMap<GlyphKey<F::Family>, CachedGlyphIndex>,

    glyphs: Vec<Glyph<F::Family>>,

    packer: Packer,
    rects: Vec<Rect>,

    cached_glyphs: Vec<CachedGlyph>,

    width: usize,
    height: usize,
    texture: Box<[u8]>,
}

impl<'a, F> GlyphCache<'a, F>
where
    F: FontCollection<'a>,
{
    pub fn new(fonts: &'a F, width: usize, height: usize, padding: usize) -> Self {
        Self {
            fonts,

            padding,

            glyphs: Vec::new(),

            next_cached_glyph_index: 0,
            cached_glyph_lookup: Default::default(),

            packer: Packer::new(width - padding, height - padding),
            rects: Vec::new(),

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

    fn oversample_for_size(size_px: f32) -> Oversample {
        if size_px <= 25.0 {
            Oversample::X2
        } else {
            Oversample::None
        }
    }

    pub fn cache_glyph(
        &mut self,
        family: F::Family,
        glyph_index: GlyphIndex,
        size_px: f32,
    ) -> CachedGlyphIndex {
        let key = GlyphKey {
            family,
            glyph_index,
            size_px: FiniteF32::new(size_px).unwrap(),
        };

        *self.cached_glyph_lookup.entry(key).or_insert_with(|| {
            let cached_glyph_index = CachedGlyphIndex(self.next_cached_glyph_index);
            self.next_cached_glyph_index += 1;
            cached_glyph_index
        })
    }

    pub fn update_atlas(&mut self) -> (&[CachedGlyph], &[u8]) {
        // We recreate the CachedGlyphs structure completely every update, so reset the index here.
        self.next_cached_glyph_index = 0;

        self.glyphs.clear();
        self.glyphs.extend(self.cached_glyph_lookup.iter().map(
            |(glyph_key, &cached_glyph_index)| Glyph {
                family: glyph_key.family,
                glyph_index: glyph_key.glyph_index,
                size_px: glyph_key.size_px,
                cached_glyph_index,
            },
        ));

        // Sort just so we avoid ping-ponging between fonts during rendering.
        self.glyphs.sort_unstable();

        let padding = self.padding as i32;

        self.rects.clear();
        self.rects.extend(self.glyphs.iter().map(|glyph| {
            let font = self.fonts.font(glyph.family);
            let size_px = glyph.size_px.get();
            let scale = font.scale_for_size_px(size_px);
            let oversample = Self::oversample_for_size(size_px);

            let bitmap_box = font.glyph_bitmap_box(
                glyph.glyph_index,
                scale * oversample.as_f32(),
                scale * oversample.as_f32(),
                0.0,
                0.0,
            );

            let w = bitmap_box.x1 - bitmap_box.x0 + padding + oversample.as_i32() - 1;
            let h = bitmap_box.y1 - bitmap_box.y0 + padding + oversample.as_i32() - 1;

            Rect {
                id: glyph.cached_glyph_index.0 as i32,
                w,
                h,
                x: 0,
                y: 0,
                was_packed: 0,
            }
        }));

        self.packer.clear();
        assert!(self.packer.pack(&mut self.rects));

        self.texture.fill(0);
        self.cached_glyphs
            .resize(self.glyphs.len(), CachedGlyph::default());

        let padding = self.padding as i32;

        for (glyph, rect) in self.glyphs.iter().zip(self.rects.iter_mut()) {
            let font = self.fonts.font(glyph.family);

            // Pad on left and top.
            rect.x += padding;
            rect.y += padding;
            rect.w -= padding;
            rect.h -= padding;

            let size_px = glyph.size_px.get();
            let scale = font.scale_for_size_px(size_px);
            let oversample = Self::oversample_for_size(size_px);

            let scale_x = scale * oversample.as_f32();
            let scale_y = scale * oversample.as_f32();

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
                oversample,
                oversample,
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
                scale * oversample.as_f32(),
                scale * oversample.as_f32(),
                0.0,
                0.0,
            );

            cached_glyph.offset_x0 = x0 as f32 / oversample.as_f32() + sub_x;
            cached_glyph.offset_y0 = y0 as f32 / oversample.as_f32() + sub_y;
            cached_glyph.offset_x1 = (x0 + rect.w) as f32 / oversample.as_f32() + sub_x;
            cached_glyph.offset_y1 = (y0 + rect.h) as f32 / oversample.as_f32() + sub_y;
        }

        (&self.cached_glyphs, &self.texture)
    }
}
