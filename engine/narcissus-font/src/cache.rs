use std::collections::hash_map::Entry;

use crate::{font::GlyphBitmapBox, FontCollection, GlyphIndex, Oversample, Packer};
pub use narcissus_core::FiniteF32;
use narcissus_core::{default, Widen};
use rustc_hash::FxHashMap;
use stb_truetype_sys::rectpack::Rect;

/// A key that uniquely identifies a given glyph within the glyph cache.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct GlyphKey<Family> {
    glyph_index: GlyphIndex,
    size_px: FiniteF32,
    family: Family,
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
    pub atlas_min_x: i32,
    pub atlas_min_y: i32,
    pub atlas_max_x: i32,
    pub atlas_max_y: i32,

    // Glyph bounding box relative to glyph origin.
    pub offset_min_x: f32,
    pub offset_min_y: f32,
    pub offset_max_x: f32,
    pub offset_max_y: f32,
}

struct CachedGlyph<F> {
    glyph_key: GlyphKey<F>,
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
    touched_glyph_lookup: FxHashMap<GlyphKey<F::Family>, TouchedGlyphIndex>,
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
    /// Oversampling renders the glyphs pre-filtered so rendering can use
    /// bilinear filtering to avoid blurriness when glyphs are not placed on
    /// exact pixel boundaries. Since it scales the size of the rendered
    /// glyph by a fixed multipler, it can be very costly in terms of atlas
    /// space for larger font sizes. Additionally the positive impact of
    /// oversampling is less pronounced at large font sizes.
    ///
    /// This function chooses an arbitrary threshold above which to disable
    /// oversampling in an attempt to balance atlas space usage and quality.
    fn oversample_for_size(size_px: f32) -> Oversample {
        if size_px <= 25.0 {
            Oversample::X4
        } else {
            Oversample::X2
        }
    }

    pub fn touch_glyph(
        &mut self,
        family: F::Family,
        glyph_index: GlyphIndex,
        size_px: f32,
    ) -> TouchedGlyphIndex {
        let key = GlyphKey {
            family,
            glyph_index,
            size_px: FiniteF32::new(size_px).unwrap(),
        };

        match self.touched_glyph_lookup.entry(key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let touched_glyph_index = TouchedGlyphIndex(self.next_touched_glyph_index);
                self.next_touched_glyph_index += 1;
                *entry.insert(touched_glyph_index)
            }
        }
    }

    pub fn update_atlas(&mut self) -> (&[TouchedGlyph], Option<&[u8]>) {
        // We might have touched more, or fewer, glyphs this iteration, so
        // update the touched glyphs array.
        self.touched_glyphs
            .resize(self.touched_glyph_lookup.len(), TouchedGlyph::default());

        // We can only repack once.
        let mut is_emergency_repack = false;

        let updated_atlas = 'emergency_repack: loop {
            let cached_glyphs_len = self.cached_glyphs.len();
            let sorted_indices = self.cached_glyph_indices_sorted.as_slice();

            // For each touched glyph, try and find it in our cached glyph list.
            for (glyph_key, touched_glyph_index) in self.touched_glyph_lookup.iter() {
                match sorted_indices
                    .binary_search_by_key(glyph_key, |&index| self.cached_glyphs[index].glyph_key)
                {
                    // We've already cached this glyph. So we just need to write
                    // into `touched_glyphs`.
                    Ok(index) => {
                        let cached_glyph_index = sorted_indices[index];
                        let cached_glyph = &self.cached_glyphs[cached_glyph_index];
                        let rect = &self.rects[cached_glyph_index];

                        let touched_glyph = &mut self.touched_glyphs[touched_glyph_index.0.widen()];

                        touched_glyph.atlas_min_x = rect.x;
                        touched_glyph.atlas_min_y = rect.y;
                        touched_glyph.atlas_max_x = rect.x + rect.w;
                        touched_glyph.atlas_max_y = rect.y + rect.h;

                        touched_glyph.offset_min_x = cached_glyph.offset_x0;
                        touched_glyph.offset_min_y = cached_glyph.offset_y0;
                        touched_glyph.offset_max_x = cached_glyph.offset_x1;
                        touched_glyph.offset_max_y = cached_glyph.offset_y1;
                    }
                    // This glyph isn't cached, so we must prepare to pack and
                    // render it.
                    Err(_) => {
                        let font = self.fonts.font(glyph_key.family);
                        let size_px = glyph_key.size_px.get();
                        let oversample = Self::oversample_for_size(size_px);
                        let scale = font.scale_for_size_px(size_px) * oversample.as_f32();

                        let GlyphBitmapBox { x0, x1, y0, y1 } =
                            font.glyph_bitmap_box(glyph_key.glyph_index, scale, scale, 0.0, 0.0);

                        let w = x1 - x0 + self.padding as i32 + oversample.as_i32() - 1;
                        let h = y1 - y0 + self.padding as i32 + oversample.as_i32() - 1;

                        self.cached_glyphs.push(CachedGlyph {
                            glyph_key: *glyph_key,
                            x0,
                            y0,
                            // These zeroed fields will be filled out in the
                            // render step.
                            offset_x0: 0.0,
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

            // New glyphs are now stored in the range `cached_glyphs_len..` in
            // both the `rects` and `cached_glyphs` structures.
            let new_rects = &mut self.rects[cached_glyphs_len..];
            let new_cached_glyphs = &mut self.cached_glyphs[cached_glyphs_len..];

            // We add the new rects to the existing packer state. This can be
            // less than optimal, but allows us to avoid invalidating previous
            // entries in the cache.
            if !self.packer.pack(new_rects) {
                assert!(
                    !is_emergency_repack,
                    "emergency repack failed, texture atlas exhausted"
                );

                // If packing fails, wipe the cache and try again with a full
                // repack, dropping any glyphs that aren't required in this
                // update.
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
                    cached_glyph.glyph_key.glyph_index,
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

                touched_glyph.atlas_min_x = rect.x;
                touched_glyph.atlas_min_y = rect.y;
                touched_glyph.atlas_max_x = rect.x + rect.w;
                touched_glyph.atlas_max_y = rect.y + rect.h;

                touched_glyph.offset_min_x = offset_x0;
                touched_glyph.offset_min_y = offset_y0;
                touched_glyph.offset_max_x = offset_x1;
                touched_glyph.offset_max_y = offset_y1;
            }

            // The `cached_glyphs` and `rects` arrays need to be sorted for the
            // lookup binary search, but instead of sorting them directly, we
            // sort a small indirection table since that's a bit simpler to
            // execute.
            self.cached_glyph_indices_sorted.clear();
            self.cached_glyph_indices_sorted
                .extend(0..self.cached_glyphs.len());
            self.cached_glyph_indices_sorted
                .sort_unstable_by_key(|&index| self.cached_glyphs[index].glyph_key);

            break true;
        };

        // Each update gets new touched glyphs, so we need to clear the hashmap.
        // However this cannot happen until the function exit as the touched
        // glyphs are needed for the emergency repack.
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
