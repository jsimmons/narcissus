use std::collections::hash_map::Entry;

use crate::{
    font::{GlyphBitmapBox, HorizontalMetrics},
    FontCollection, GlyphIndex, Oversample, Packer,
};
use narcissus_core::default;
use rustc_hash::FxHashMap;
use stb_truetype_sys::rectpack::Rect;

pub use narcissus_core::FiniteF32;

/// An index into the CachedGlyph slice.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct TouchedGlyphIndex(u32);

/// Holds data required to draw a glyph from the glyph atlas.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct TouchedGlyph {
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

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct GlyphKey<Family> {
    family: Family,
    c: char,
    size_px: FiniteF32,
}

#[derive(Clone, Copy)]
pub struct TouchedGlyphInfo {
    pub touched_glyph_index: TouchedGlyphIndex,
    pub glyph_index: GlyphIndex,
    pub advance_width: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CachedGlyphIndex(u32);

struct CachedGlyph<F> {
    glyph_key: GlyphKey<F>,
    glyph_index: GlyphIndex,
    offset_x0: f32,
    offset_y0: f32,
    offset_x1: f32,
    offset_y1: f32,
}

pub struct GlyphCache<'a, F>
where
    F: FontCollection<'a>,
{
    fonts: &'a F,

    padding: usize,
    width: usize,
    height: usize,
    texture: Box<[u8]>,

    next_touched_glyph_index: u32,
    touched_glyph_lookup: FxHashMap<GlyphKey<F::Family>, TouchedGlyphInfo>,
    touched_glyphs: Vec<TouchedGlyph>,

    cached_glyph_indices_sorted: Vec<usize>,
    cached_glyphs: Vec<CachedGlyph<F::Family>>,
    rects: Vec<Rect>,

    packer: Packer,
}

impl<'a, F> GlyphCache<'a, F>
where
    F: FontCollection<'a>,
{
    pub fn new(fonts: &'a F, width: usize, height: usize, padding: usize) -> Self {
        Self {
            fonts,

            padding,
            width,
            height,
            texture: vec![0; width * height].into_boxed_slice(),

            next_touched_glyph_index: 0,
            touched_glyph_lookup: default(),
            touched_glyphs: default(),

            cached_glyph_indices_sorted: default(),
            cached_glyphs: default(),
            rects: default(),

            packer: Packer::new(width - padding, height - padding),
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

    pub fn touch_glyph(&mut self, family: F::Family, c: char, size_px: f32) -> TouchedGlyphInfo {
        let key = GlyphKey {
            family,
            c,
            size_px: FiniteF32::new(size_px).unwrap(),
        };

        match self.touched_glyph_lookup.entry(key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let touched_glyph_index = TouchedGlyphIndex(self.next_touched_glyph_index);
                self.next_touched_glyph_index += 1;

                let font = self.fonts.font(family);
                let glyph_index = font
                    .glyph_index(c)
                    .unwrap_or_else(|| font.glyph_index('□').unwrap());

                let HorizontalMetrics {
                    advance_width,
                    left_side_bearing: _,
                } = font.horizontal_metrics(glyph_index);

                *entry.insert(TouchedGlyphInfo {
                    touched_glyph_index,
                    glyph_index,
                    advance_width,
                })
            }
        }
    }

    pub fn update_atlas(&mut self) -> (&[TouchedGlyph], Option<&[u8]>) {
        debug_assert_eq!(
            self.touched_glyph_lookup.len(),
            self.next_touched_glyph_index as usize
        );
        self.touched_glyphs
            .resize(self.touched_glyph_lookup.len(), TouchedGlyph::default());
        self.next_touched_glyph_index = 0;

        let mut is_emergency_repack = false;

        'emergency_repack: loop {
            let cached_glyphs_len = self.cached_glyphs.len();

            // For each touched glyph, try and find it in our cached glyph list.
            for (&glyph_key, touched_glyph_info) in self.touched_glyph_lookup.iter() {
                match self
                    .cached_glyph_indices_sorted
                    .binary_search_by_key(&glyph_key, |&cached_glyph_index| {
                        self.cached_glyphs[cached_glyph_index].glyph_key
                    }) {
                    // We've already cached this glyph. So we just need to write the `touched_glyphs`
                    // information.
                    Ok(index) => {
                        let cached_glyph_index = self.cached_glyph_indices_sorted[index];

                        let touched_glyph = &mut self.touched_glyphs
                            [touched_glyph_info.touched_glyph_index.0 as usize];

                        let rect = &self.rects[cached_glyph_index];
                        touched_glyph.x0 = rect.x;
                        touched_glyph.x1 = rect.x + rect.w;
                        touched_glyph.y0 = rect.y;
                        touched_glyph.y1 = rect.y + rect.h;

                        let cached_glyph = &self.cached_glyphs[cached_glyph_index];
                        touched_glyph.offset_x0 = cached_glyph.offset_x0;
                        touched_glyph.offset_y0 = cached_glyph.offset_y0;
                        touched_glyph.offset_x1 = cached_glyph.offset_x1;
                        touched_glyph.offset_y1 = cached_glyph.offset_y1;
                    }
                    // This glyph isn't cached, so we must prepare to pack and render it.
                    Err(_) => {
                        let font = self.fonts.font(glyph_key.family);
                        let size_px = glyph_key.size_px.get();
                        let scale = font.scale_for_size_px(size_px);
                        let oversample = Self::oversample_for_size(size_px);

                        let bitmap_box = font.glyph_bitmap_box(
                            touched_glyph_info.glyph_index,
                            scale * oversample.as_f32(),
                            scale * oversample.as_f32(),
                            0.0,
                            0.0,
                        );

                        let w = bitmap_box.x1 - bitmap_box.x0
                            + self.padding as i32
                            + oversample.as_i32()
                            - 1;
                        let h = bitmap_box.y1 - bitmap_box.y0
                            + self.padding as i32
                            + oversample.as_i32()
                            - 1;

                        self.cached_glyphs.push(CachedGlyph {
                            glyph_key,
                            glyph_index: touched_glyph_info.glyph_index,
                            offset_x0: 0.0,
                            offset_y0: 0.0,
                            offset_x1: 0.0,
                            offset_y1: 0.0,
                        });

                        self.rects.push(Rect {
                            id: touched_glyph_info.touched_glyph_index.0 as i32,
                            w,
                            h,
                            x: 0,
                            y: 0,
                            was_packed: 0,
                        });
                    }
                }
            }

            // If we haven't added anything new, we're done here.
            if self.cached_glyphs.len() == cached_glyphs_len {
                self.touched_glyph_lookup.clear();
                return (&self.touched_glyphs, None);
            }

            // Pack any new glyphs we might have.
            //
            // If packing fails, wipe the cache and try again with a full repack and render of the
            // touched_glyphs this iteration.
            if !self.packer.pack(&mut self.rects[cached_glyphs_len..]) {
                assert!(
                    !is_emergency_repack,
                    "emergency repack failed, texture atlas exhausted"
                );

                self.cached_glyph_indices_sorted.clear();
                self.cached_glyphs.clear();
                self.rects.clear();
                self.packer.clear();
                self.texture.fill(0);

                is_emergency_repack = true;
                continue 'emergency_repack;
            }

            // Render any new glyphs we might have.
            for (cached_glyph, rect) in self.cached_glyphs[cached_glyphs_len..]
                .iter_mut()
                .zip(self.rects[cached_glyphs_len..].iter_mut())
            {
                let font = self.fonts.font(cached_glyph.glyph_key.family);

                // Pad on left and top.
                let padding = self.padding as i32;
                rect.x += padding;
                rect.y += padding;
                rect.w -= padding;
                rect.h -= padding;

                let size_px = cached_glyph.glyph_key.size_px.get();
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
                    cached_glyph.glyph_index,
                );

                let GlyphBitmapBox {
                    x0,
                    x1: _,
                    y0,
                    y1: _,
                } = font.glyph_bitmap_box(
                    cached_glyph.glyph_index,
                    scale * oversample.as_f32(),
                    scale * oversample.as_f32(),
                    0.0,
                    0.0,
                );

                let offset_x0 = x0 as f32 / oversample.as_f32() + sub_x;
                let offset_y0 = y0 as f32 / oversample.as_f32() + sub_y;
                let offset_x1 = (x0 + rect.w) as f32 / oversample.as_f32() + sub_x;
                let offset_y1 = (y0 + rect.h) as f32 / oversample.as_f32() + sub_y;

                cached_glyph.offset_x0 = offset_x0;
                cached_glyph.offset_y0 = offset_y0;
                cached_glyph.offset_x1 = offset_x1;
                cached_glyph.offset_y1 = offset_y1;

                let touched_glyph = &mut self.touched_glyphs[rect.id as usize];

                touched_glyph.x0 = rect.x;
                touched_glyph.x1 = rect.x + rect.w;
                touched_glyph.y0 = rect.y;
                touched_glyph.y1 = rect.y + rect.h;

                touched_glyph.offset_x0 = offset_x0;
                touched_glyph.offset_y0 = offset_y0;
                touched_glyph.offset_x1 = offset_x1;
                touched_glyph.offset_y1 = offset_y1;
            }

            self.cached_glyph_indices_sorted.clear();
            self.cached_glyph_indices_sorted
                .extend(0..self.cached_glyphs.len());
            self.cached_glyph_indices_sorted
                .sort_unstable_by_key(|&index| self.cached_glyphs[index].glyph_key);

            self.touched_glyph_lookup.clear();
            return (&self.touched_glyphs, Some(&self.texture));
        }
    }
}
