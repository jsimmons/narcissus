use std::collections::hash_map::Entry;

use crate::{
    font::{GlyphBitmapBox, HorizontalMetrics},
    FontCollection, GlyphIndex, Oversample, Packer,
};
use narcissus_core::default;
use rustc_hash::FxHashMap;
use stb_truetype_sys::rectpack::Rect;

pub use narcissus_core::FiniteF32;

/// A key that uniquely identifies a given glyph within the glyph cache.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct GlyphKey<Family> {
    family: Family,
    c: char,
    size_px: FiniteF32,
}

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

/// Information stored for each touched glyph to avoid re-computing for each instance of that glyph.
#[derive(Clone, Copy)]
pub struct TouchedGlyphInfo {
    pub touched_glyph_index: TouchedGlyphIndex,
    pub glyph_index: GlyphIndex,
    pub advance_width: f32,
}

struct CachedGlyph<F> {
    glyph_key: GlyphKey<F>,
    glyph_index: GlyphIndex,
    x0: i32,
    y0: i32,
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

    /// Calculate oversampling factor for a given font size in pixels.
    ///
    /// Oversampling renders the glyphs pre-filtered at a higher resolution, so rendering can use bilinear filtering to
    /// avoid blurriness on small fonts that aren't placed at pixel boundaries. Since it scales the size of the rendered
    /// glyph by some fixed multipler, it's very costly in terms of atlas space for larger fonts. At the same time,
    /// larger fonts don't benefit significantly from the filtering.
    ///
    /// This function chooses an arbitrary threshold above which to disable oversampling to avoid wasting atlas space.
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
        // We might have touched more, or fewer, glyphs this iteration, so update the touched glyphs array.
        self.touched_glyphs
            .resize(self.touched_glyph_lookup.len(), TouchedGlyph::default());

        // We can only repack once.
        let mut is_emergency_repack = false;

        let updated_atlas = 'emergency_repack: loop {
            let cached_glyphs_len = self.cached_glyphs.len();
            let sorted_indices = &self.cached_glyph_indices_sorted;

            // For each touched glyph, try and find it in our cached glyph list.
            for (glyph_key, touched_glyph_info) in self.touched_glyph_lookup.iter() {
                let touched_glyph_index = touched_glyph_info.touched_glyph_index.0;
                let glyph_index = touched_glyph_info.glyph_index;

                match sorted_indices
                    .binary_search_by_key(glyph_key, |&index| self.cached_glyphs[index].glyph_key)
                {
                    // We've already cached this glyph. So we just need to write into `touched_glyphs`.
                    Ok(index) => {
                        let cached_glyph_index = sorted_indices[index];
                        let cached_glyph = &self.cached_glyphs[cached_glyph_index];
                        let rect = &self.rects[cached_glyph_index];

                        let touched_glyph = &mut self.touched_glyphs[touched_glyph_index as usize];

                        touched_glyph.x0 = rect.x;
                        touched_glyph.x1 = rect.x + rect.w;
                        touched_glyph.y0 = rect.y;
                        touched_glyph.y1 = rect.y + rect.h;

                        touched_glyph.offset_x0 = cached_glyph.offset_x0;
                        touched_glyph.offset_y0 = cached_glyph.offset_y0;
                        touched_glyph.offset_x1 = cached_glyph.offset_x1;
                        touched_glyph.offset_y1 = cached_glyph.offset_y1;
                    }
                    // This glyph isn't cached, so we must prepare to pack and render it.
                    Err(_) => {
                        let font = self.fonts.font(glyph_key.family);
                        let size_px = glyph_key.size_px.get();
                        let oversample = Self::oversample_for_size(size_px);
                        let scale = font.scale_for_size_px(size_px) * oversample.as_f32();

                        let GlyphBitmapBox { x0, x1, y0, y1 } =
                            font.glyph_bitmap_box(glyph_index, scale, scale, 0.0, 0.0);

                        let w = x1 - x0 + self.padding as i32 + oversample.as_i32() - 1;
                        let h = y1 - y0 + self.padding as i32 + oversample.as_i32() - 1;

                        self.cached_glyphs.push(CachedGlyph {
                            glyph_key: *glyph_key,
                            glyph_index,
                            x0,
                            y0,
                            offset_x0: 0.0, // These zeroed fields will be filled out in the render step.
                            offset_y0: 0.0,
                            offset_x1: 0.0,
                            offset_y1: 0.0,
                        });

                        self.rects.push(Rect {
                            id: 0,
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
                break false;
            }

            // New glyphs are now stored in the range `cached_glyphs_len..` in both the `rects` and `cached_glyphs`
            // structures.
            let new_rects = &mut self.rects[cached_glyphs_len..];
            let new_cached_glyphs = &mut self.cached_glyphs[cached_glyphs_len..];

            // First we must pack the new glyph rects so we know where to render them.
            if !self.packer.pack(new_rects) {
                assert!(
                    !is_emergency_repack,
                    "emergency repack failed, texture atlas exhausted"
                );

                // If packing fails, wipe the cache and try again with a full repack and render of touched_glyphs.
                self.cached_glyph_indices_sorted.clear();
                self.cached_glyphs.clear();
                self.rects.clear();
                self.packer.clear();
                self.texture.fill(0);

                is_emergency_repack = true;
                continue 'emergency_repack;
            }

            // Render the new glyphs we've just packed.
            for (cached_glyph, rect) in new_cached_glyphs.iter_mut().zip(new_rects.iter_mut()) {
                let font = self.fonts.font(cached_glyph.glyph_key.family);
                let size_px = cached_glyph.glyph_key.size_px.get();
                let oversample = Self::oversample_for_size(size_px);
                let scale = font.scale_for_size_px(size_px) * oversample.as_f32();

                // Pad on left and top.
                let padding = self.padding as i32;
                rect.x += padding;
                rect.y += padding;
                rect.w -= padding;
                rect.h -= padding;

                let (sub_x, sub_y) = font.render_glyph_bitmap(
                    &mut self.texture,
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    self.width as i32,
                    scale,
                    scale,
                    0.0,
                    0.0,
                    oversample,
                    oversample,
                    cached_glyph.glyph_index,
                );

                let offset_x0 = cached_glyph.x0 as f32 / oversample.as_f32() + sub_x;
                let offset_y0 = cached_glyph.y0 as f32 / oversample.as_f32() + sub_y;
                let offset_x1 = (cached_glyph.x0 + rect.w) as f32 / oversample.as_f32() + sub_x;
                let offset_y1 = (cached_glyph.y0 + rect.h) as f32 / oversample.as_f32() + sub_y;

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

            // Instead of sorting the `cached_glyphs` and `rects` arrays directly, we sort an indirection array. Since
            // we've changed the cached_glyphs array, this needs to be updated now.
            self.cached_glyph_indices_sorted.clear();
            self.cached_glyph_indices_sorted
                .extend(0..self.cached_glyphs.len());
            self.cached_glyph_indices_sorted
                .sort_unstable_by_key(|&index| self.cached_glyphs[index].glyph_key);

            break true;
        };

        // Each update gets new touched glyphs, so we need to clear the hashmap. However this cannot happen until the
        // function exit as the touched glyphs are needed for the emergency repack.
        self.touched_glyph_lookup.clear();
        self.next_touched_glyph_index = 0;

        (
            &self.touched_glyphs,
            if updated_atlas {
                Some(&self.texture)
            } else {
                None
            },
        )
    }
}
