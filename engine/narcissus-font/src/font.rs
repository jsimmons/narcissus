use narcissus_core::zeroed_box;
use stb_truetype_sys::{
    stbtt_FindGlyphIndex, stbtt_GetFontOffsetForIndex, stbtt_GetFontVMetrics,
    stbtt_GetGlyphBitmapBoxSubpixel, stbtt_GetGlyphHMetrics, stbtt_GetGlyphKernAdvance,
    stbtt_InitFont, stbtt_MakeGlyphBitmapSubpixelPrefilter, truetype,
};
use std::{marker::PhantomData, mem::MaybeUninit, num::NonZeroI32};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Oversample {
    None,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
}

impl Oversample {
    pub fn as_i32(self) -> i32 {
        self as i32 + 1
    }

    pub fn as_f32(self) -> f32 {
        (self as i32) as f32 + 1.0
    }
}

/// Font vertical metrics in unscaled coordinates.
///
/// You should advance the vertical position by
/// `(ascent - descent + line_gap) * scale`
#[derive(Clone, Copy, Debug)]
pub struct VerticalMetrics {
    /// Coordinate above the baseline the font extends.
    pub ascent: f32,
    /// Coordinate below the baseline the font extends.
    pub descent: f32,
    /// The spacing between one row's descent and the next row's ascent.
    pub line_gap: f32,
}

/// Glyph horizontal metrics in unscaled coordinates.
///
/// You should advance the horizontal position by `advance_width * scale`
#[derive(Clone, Copy, Debug)]
pub struct HorizontalMetrics {
    /// The offset from the current horizontal position to the next horizontal
    /// position.
    pub advance_width: f32,
    /// The offset from the current horizontal position to the left edge of the
    /// character.
    pub left_side_bearing: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(C)]
pub struct GlyphIndex(NonZeroI32);

impl GlyphIndex {
    pub fn as_u32(self) -> u32 {
        self.0.get() as u32
    }
}

/// Coordinates:
///  +x right
///  +y down
///
/// ```text
/// (x0,y0)
///        +-----+
///        |     |
///        |     |
///        |     |
///        +-----+
///               (x1,y1)
/// ```
pub struct GlyphBitmapBox {
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
}

const ASCII_PRINTABLE_FIRST: i32 = 32;
const ASCII_PRINTABLE_LAST: i32 = 127;
const ASCII_PRINTABLE_COUNT: usize = (ASCII_PRINTABLE_LAST - ASCII_PRINTABLE_FIRST + 1) as usize;

const KERNING_CACHE_FIRST: i32 = 0;
const KERNING_CACHE_LAST: i32 = 127;
const KERNING_CACHE_COUNT: usize = (KERNING_CACHE_LAST - KERNING_CACHE_FIRST + 1) as usize;

pub struct Font<'a> {
    info: truetype::FontInfo,
    phantom: PhantomData<&'a [u8]>,
    vertical_metrics: VerticalMetrics,
    glyph_index_cache: [Option<GlyphIndex>; ASCII_PRINTABLE_COUNT],
    kerning_cache: Box<[[f32; KERNING_CACHE_COUNT]; KERNING_CACHE_COUNT]>,
}

impl<'a> Font<'a> {
    /// Create a new `Font` from ttf data.
    ///
    /// # Safety
    ///
    /// Must be a valid ttf font from a trusted source. Invalid data is not
    /// safely handled.
    pub unsafe fn from_bytes(data: &'a [u8]) -> Self {
        let info = unsafe {
            let mut info = MaybeUninit::uninit();
            let ret = stbtt_InitFont(
                info.as_mut_ptr(),
                data.as_ptr(),
                stbtt_GetFontOffsetForIndex(data.as_ptr(), 0),
            );
            assert!(ret != 0, "failed to load ttf font");
            info.assume_init()
        };

        let vertical_metrics = {
            let mut ascent = 0;
            let mut descent = 0;
            let mut line_gap = 0;
            // SAFETY: We've just initialized the font info above.
            unsafe { stbtt_GetFontVMetrics(&info, &mut ascent, &mut descent, &mut line_gap) };
            VerticalMetrics {
                ascent: ascent as f32,
                descent: descent as f32,
                line_gap: line_gap as f32,
            }
        };

        let mut glyph_index_cache = [None; ASCII_PRINTABLE_COUNT];
        for (i, printable) in (ASCII_PRINTABLE_FIRST..=ASCII_PRINTABLE_LAST).enumerate() {
            glyph_index_cache[i] =
                NonZeroI32::new(unsafe { stbtt_FindGlyphIndex(&info, printable as i32) })
                    .map(GlyphIndex);
        }

        // SAFETY: Safe to zero as all zero bytes f32 is 0.0
        let mut kerning_cache: Box<[[f32; KERNING_CACHE_COUNT]; KERNING_CACHE_COUNT]> =
            unsafe { zeroed_box().assume_init() };
        for (i, second) in (KERNING_CACHE_FIRST..=KERNING_CACHE_LAST).enumerate() {
            for (j, first) in (KERNING_CACHE_FIRST..=KERNING_CACHE_LAST).enumerate() {
                kerning_cache[i][j] =
                    unsafe { stbtt_GetGlyphKernAdvance(&info, first, second) as f32 };
            }
        }

        Self {
            info,
            phantom: PhantomData,
            vertical_metrics,
            glyph_index_cache,
            kerning_cache,
        }
    }

    /// Returns a scale factor to produce a font whose "height" is `size_px`
    /// pixels tall.
    ///
    /// Height is measured as the distance from the highest ascender to the
    /// lowest descender.
    pub fn scale_for_size_px(&self, size_px: f32) -> f32 {
        size_px / (self.vertical_metrics.ascent - self.vertical_metrics.descent)
    }

    /// Return the font's vertical ascent in unscaled coordinates.
    pub fn ascent(&self) -> f32 {
        self.vertical_metrics.ascent
    }

    /// Return the font's vertical descent in unscaled coordinates.
    pub fn descent(&self) -> f32 {
        self.vertical_metrics.descent
    }

    /// Return the font's line gap in unscaled coordinates.
    pub fn line_gap(&self) -> f32 {
        self.vertical_metrics.line_gap
    }

    /// Return the `GlyphIndex` for the character, or `None` if the font has no
    /// matching glyph.
    pub fn glyph_index(&self, c: char) -> Option<GlyphIndex> {
        if (c as i32) >= ASCII_PRINTABLE_FIRST && (c as i32) <= ASCII_PRINTABLE_LAST {
            self.glyph_index_cache[(c as i32 - ASCII_PRINTABLE_FIRST) as usize]
        } else {
            let glyph_index = unsafe { stbtt_FindGlyphIndex(&self.info, c as i32) };
            NonZeroI32::new(glyph_index).map(GlyphIndex)
        }
    }

    pub fn glyph_bitmap_box(
        &self,
        glyph_index: GlyphIndex,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
    ) -> GlyphBitmapBox {
        let mut x0 = 0;
        let mut x1 = 0;
        let mut y0 = 0;
        let mut y1 = 0;
        unsafe {
            stbtt_GetGlyphBitmapBoxSubpixel(
                &self.info,
                glyph_index.0.get(),
                scale_x,
                scale_y,
                shift_x,
                shift_y,
                &mut x0,
                &mut y0,
                &mut x1,
                &mut y1,
            );
        }
        GlyphBitmapBox { x0, x1, y0, y1 }
    }

    pub fn render_glyph_bitmap(
        &self,
        out: &mut [u8],
        out_x: i32,
        out_y: i32,
        out_w: i32,
        out_h: i32,
        out_stride: i32,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
        oversample_h: Oversample,
        oversample_v: Oversample,
        glyph: GlyphIndex,
    ) -> (f32, f32) {
        let mut sub_x = 0.0;
        let mut sub_y = 0.0;

        unsafe {
            stbtt_MakeGlyphBitmapSubpixelPrefilter(
                &self.info,
                out.as_mut_ptr()
                    .offset(out_y as isize * out_stride as isize + out_x as isize),
                out_w,
                out_h,
                out_stride,
                scale_x,
                scale_y,
                shift_x,
                shift_y,
                oversample_h.as_i32(),
                oversample_v.as_i32(),
                &mut sub_x,
                &mut sub_y,
                glyph.0.get(),
            );
        }

        (sub_x, sub_y)
    }

    pub fn horizontal_metrics(&self, glyph: GlyphIndex) -> HorizontalMetrics {
        let mut advance_width = 0;
        let mut left_side_bearing = 0;
        unsafe {
            stbtt_GetGlyphHMetrics(
                &self.info,
                glyph.0.get(),
                &mut advance_width,
                &mut left_side_bearing,
            )
        };
        HorizontalMetrics {
            advance_width: advance_width as f32,
            left_side_bearing: left_side_bearing as f32,
        }
    }

    pub fn kerning_advance(&self, glyph1: GlyphIndex, glyph2: GlyphIndex) -> f32 {
        let glyph1 = glyph1.0.get();
        let glyph2 = glyph2.0.get();
        if glyph1 >= KERNING_CACHE_FIRST
            && glyph1 <= KERNING_CACHE_LAST
            && glyph2 >= KERNING_CACHE_FIRST
            && glyph2 <= KERNING_CACHE_LAST
        {
            self.kerning_cache[(glyph2 - KERNING_CACHE_FIRST) as usize]
                [(glyph1 - KERNING_CACHE_FIRST) as usize]
        } else {
            unsafe { stbtt_GetGlyphKernAdvance(&self.info, glyph1, glyph2) as f32 }
        }
    }
}

pub trait FontCollection<'a> {
    type Family: Copy + Eq + Ord + std::hash::Hash;
    fn font(&self, family: Self::Family) -> &Font<'a>;
}
