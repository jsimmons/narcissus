use std::ffi::c_void;

#[repr(C)]
pub struct stbtt_pack_context {
    user_allocator_context: *mut c_void,
    pack_info: *mut c_void,
    width: i32,
    height: i32,
    stride_in_bytes: i32,
    padding: i32,
    skip_missing: i32,
    h_oversample: u32,
    v_oversample: u32,
    pixels: *mut u8,
    nodes: *mut c_void,
}

#[repr(C)]
pub struct stbtt_packedchar {
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16, // coordinates of bbox in bitmap
    xoff: f32,
    yoff: f32,
    xadvance: f32,
    xoff2: f32,
    yoff2: f32,
}

#[repr(C)]
pub struct stbtt_pack_range {
    font_size: f32,
    first_unicode_codepoint_in_range: i32, // if non-zero, then the chars are continuous, and this is the first codepoint
    array_of_unicode_codepoints: *const i32, // if non-zero, then this is an array of unicode codepoints
    num_chars: i32,
    chardata_for_range: *mut stbtt_packedchar, // output
    h_oversample: u32,
    v_oversample: u32, // don't set these, they're used internally
}

#[repr(C)]
pub struct stbtt_aligned_quad {
    x0: f32, // top-left
    y0: f32,
    s0: f32,
    t0: f32,
    x1: f32, // bottom-right
    y1: f32,
    s1: f32,
    t1: f32,
}

#[repr(C)]
struct stbtt__buf {
    data: *mut u8,
    cursor: i32,
    size: i32,
}

#[repr(C)]
pub struct stbtt_fontinfo {
    userdata: *mut c_void,
    data: *mut u8,  // pointer to .ttf file
    fontstart: i32, // offset of start of font

    num_glyphs: i32, // number of glyphs, needed for range checking

    loca: i32,
    head: i32,
    glyf: i32,
    hhea: i32,
    hmtx: i32,
    kern: i32,
    gpos: i32,
    svg: i32,                 // table locations as offset from start of .ttf
    index_map: i32,           // a cmap mapping for our chosen character encoding
    index_to_loc_format: i32, // format needed to map from glyph index to glyph

    cff: stbtt__buf,         // cff font data
    charstrings: stbtt__buf, // the charstring index
    gsubrs: stbtt__buf,      // global charstring subroutines index
    subrs: stbtt__buf,       // private charstring subroutines index
    fontdicts: stbtt__buf,   // array of font dicts
    fdselect: stbtt__buf,    // map from glyph to fontdict
}

extern "C" {
    pub fn stbtt_PackBegin(
        spc: *mut stbtt_pack_context,
        pixels: *const u8,
        width: i32,
        height: i32,
        stride_in_bytes: i32,
        padding: i32,
        alloc_context: *mut c_void,
    ) -> i32;

    pub fn stbtt_PackSetOversampling(
        spc: *mut stbtt_pack_context,
        h_oversample: u32,
        v_oversample: u32,
    );

    pub fn stbtt_PackFontRanges(
        spc: *mut stbtt_pack_context,
        fontdata: *const u8,
        font_index: i32,
        ranges: *const stbtt_pack_range,
        num_ranges: i32,
    );

    pub fn stbtt_PackEnd(spc: *mut stbtt_pack_context);

    pub fn stbtt_GetPackedQuad(
        chardata: *const stbtt_packedchar,
        pw: i32,
        ph: i32,
        char_index: i32, // character to display
        xpos: *mut f32,
        ypos: *mut f32, // pointers to current position in screen pixel space
        q: *mut stbtt_aligned_quad, // output: quad to draw
        align_to_integer: i32,
    );

    pub fn stbtt_InitFont(info: *mut stbtt_fontinfo, data: *const u8, offset: i32) -> i32;

    pub fn stbtt_GetNumberOfFonts(data: *const u8) -> i32;

    pub fn stbtt_GetFontOffsetForIndex(data: *const u8, index: i32) -> i32;

    pub fn stbtt_GetCodepointBitmap(
        info: &stbtt_fontinfo,
        scale_x: f32,
        scale_y: f32,
        codepoint: i32,
        width: &mut i32,
        height: &mut i32,
        xoff: &mut i32,
        yoff: &mut i32,
    ) -> *mut u8;

    pub fn stbtt_MakeCodepointBitmap(
        info: &stbtt_fontinfo,
        output: *mut u8,
        out_w: i32,
        out_h: i32,
        out_stride: i32,
        scale_x: f32,
        scale_y: f32,
        codepoint: i32,
    );

    pub fn stbtt_GetCodepointBitmapBox(
        info: &stbtt_fontinfo,
        codepoint: i32,
        scale_x: f32,
        scale_y: f32,
        ix0: &mut i32,
        iy0: &mut i32,
        ix1: &mut i32,
        iy1: &mut i32,
    );

    pub fn stbtt_GetCodepointBitmapBoxSubpixel(
        info: &stbtt_fontinfo,
        codepoint: i32,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
        ix0: &mut i32,
        iy0: &mut i32,
        ix1: &mut i32,
        iy1: &mut i32,
    );

    pub fn stbtt_GetCodepointHMetrics(
        info: &stbtt_fontinfo,
        codepoint: i32,
        advance_width: &mut i32,
        left_side_bearing: &mut i32,
    );

    pub fn stbtt_GetFontVMetrics(
        info: &stbtt_fontinfo,
        ascent: &mut i32,
        descent: &mut i32,
        line_gap: &mut i32,
    );

    pub fn stbtt_GetFontVMetricsOS2(
        info: &stbtt_fontinfo,
        typo_ascent: &mut i32,
        typo_descent: &mut i32,
        typo_line_gap: &mut i32,
    ) -> i32;

    pub fn stbtt_GetCodepointKernAdvance(info: &stbtt_fontinfo, ch1: i32, ch2: i32) -> i32;
}
