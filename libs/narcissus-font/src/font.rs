use std::{marker::PhantomData, mem::MaybeUninit, num::NonZeroI32};

use stb_truetype_sys::{
    stbtt_FindGlyphIndex, stbtt_GetFontOffsetForIndex, stbtt_GetFontVMetrics,
    stbtt_GetGlyphBitmapBoxSubpixel, stbtt_GetGlyphHMetrics, stbtt_GetGlyphKernAdvance,
    stbtt_InitFont, stbtt_MakeGlyphBitmapSubpixelPrefilter, stbtt_ScaleForPixelHeight, truetype,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Oversample {
    X1,
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
}

/// Font vertical metrics in unscaled coordinates.
///
/// You should advance the vertical position by `ascent * scale - descent * scale + line_gap * scale`
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
    /// The offset from the current horizontal position to the next horizontal position.
    pub advance_width: f32,
    /// The offset from the current horizontal position to the left edge of the character.
    pub left_side_bearing: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(C)]
pub struct GlyphIndex(NonZeroI32);

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

pub struct Font<'a> {
    info: truetype::FontInfo,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> Font<'a> {
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

        Self {
            info,
            phantom: PhantomData,
        }
    }

    pub fn scale_for_pixel_height(&self, height: f32) -> f32 {
        unsafe { stbtt_ScaleForPixelHeight(&self.info, height) }
    }

    pub fn vertical_metrics(&self) -> VerticalMetrics {
        let mut ascent = 0;
        let mut descent = 0;
        let mut line_gap = 0;
        unsafe {
            stbtt_GetFontVMetrics(&self.info, &mut ascent, &mut descent, &mut line_gap);
        }
        VerticalMetrics {
            ascent: ascent as f32,
            descent: descent as f32,
            line_gap: line_gap as f32,
        }
    }

    pub fn glyph_id(&self, c: char) -> Option<GlyphIndex> {
        let glyph_id = unsafe { stbtt_FindGlyphIndex(&self.info, c as i32) };
        NonZeroI32::new(glyph_id).map(|glyph_id| GlyphIndex(glyph_id))
    }

    pub fn glyph_bitmap_box(
        &self,
        glyph: GlyphIndex,
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
                glyph.0.get(),
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

    pub fn kerning_advance(&self, glyph_1: GlyphIndex, glyph_2: GlyphIndex) -> f32 {
        unsafe { stbtt_GetGlyphKernAdvance(&self.info, glyph_1.0.get(), glyph_2.0.get()) as f32 }
    }
}

pub trait FontCollection<'a> {
    type Family: Copy + Eq + Ord + std::hash::Hash;
    fn font(&self, font_family: Self::Family) -> &Font<'a>;
}
