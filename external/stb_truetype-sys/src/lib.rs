pub mod rectpack {
    use std::mem::MaybeUninit;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[repr(C)]
    pub struct Rect {
        pub id: i32,
        pub w: i32,
        pub h: i32,
        pub x: i32,
        pub y: i32,
        pub was_packed: i32,
    }

    impl Default for Rect {
        fn default() -> Self {
            unsafe { MaybeUninit::zeroed().assume_init() }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Node {
        x: i32,
        y: i32,
        next: *mut Node,
    }

    impl Default for Node {
        fn default() -> Self {
            Self {
                x: 0,
                y: 0,
                next: std::ptr::null_mut(),
            }
        }
    }

    #[repr(C)]
    pub struct Context {
        width: i32,
        height: i32,
        align: i32,
        init_mode: i32,
        heuristic: i32,
        num_nodes: i32,
        active_head: *mut Node,
        free_head: *mut Node,
        extra: [Node; 2], // we allocate two extra nodes so optimal user-node-count is 'width' not 'width+2'
    }

    #[repr(i32)]
    #[derive(Default)]
    pub enum Heuristic {
        #[default]
        SkylineBLSortHeight,
        SkylineBFSortHeight,
    }
}

pub mod truetype {
    use std::ffi::c_void;

    #[repr(C)]
    pub struct PackContext {
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
    pub struct PackedChar {
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16, // coordinates of bbox in bitmap
        x_offset: f32,
        y_offset: f32,
        x_advance: f32,
        x_off2: f32,
        y_off2: f32,
    }

    #[repr(C)]
    pub struct PackRange {
        font_size: f32,
        first_unicode_codepoint_in_range: i32, // if non-zero, then the chars are continuous, and this is the first codepoint
        array_of_unicode_codepoints: *const i32, // if non-zero, then this is an array of unicode codepoints
        num_chars: i32,
        chardata_for_range: *mut PackedChar, // output
        h_oversample: u32,
        v_oversample: u32, // don't set these, they're used internally
    }

    #[repr(C)]
    pub struct AlignedQuad {
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
    struct Buf {
        data: *mut u8,
        cursor: i32,
        size: i32,
    }

    #[repr(C)]
    pub struct FontInfo {
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

        cff: Buf,         // cff font data
        charstrings: Buf, // the charstring index
        gsubrs: Buf,      // global charstring subroutines index
        subrs: Buf,       // private charstring subroutines index
        fontdicts: Buf,   // array of font dicts
        fdselect: Buf,    // map from glyph to fontdict
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct KerningEntry {
        pub glyph1: i32, // use stbtt_FindGlyphIndex
        pub glyph2: i32,
        pub advance: i32,
    }
}

unsafe extern "C" {
    pub fn stbrp_pack_rects(
        context: *mut rectpack::Context,
        rects: *mut rectpack::Rect,
        num_rects: i32,
    ) -> i32;

    pub fn stbrp_init_target(
        context: *mut rectpack::Context,
        width: i32,
        height: i32,
        nodes: *mut rectpack::Node,
        num_nodes: i32,
    );

    pub fn stbrp_setup_allow_out_of_mem(context: *mut rectpack::Context, allow_out_of_mem: i32);

    pub fn stbrp_setup_heuristic(context: *mut rectpack::Context, heuristic: rectpack::Heuristic);

    pub fn stbtt_PackBegin(
        spc: *mut truetype::PackContext,
        pixels: *const u8,
        width: i32,
        height: i32,
        stride_in_bytes: i32,
        padding: i32,
        alloc_context: *mut std::ffi::c_void,
    ) -> i32;

    pub fn stbtt_PackSetOversampling(
        spc: &mut truetype::PackContext,
        h_oversample: u32,
        v_oversample: u32,
    );

    pub fn stbtt_PackFontRanges(
        spc: &mut truetype::PackContext,
        fontdata: *const u8,
        font_index: i32,
        ranges: *const truetype::PackRange,
        num_ranges: i32,
    );

    pub fn stbtt_PackEnd(spc: &mut truetype::PackContext);

    pub fn stbtt_GetPackedQuad(
        chardata: *const truetype::PackedChar,
        pw: i32,
        ph: i32,
        char_index: i32, // character to display
        x: &mut f32,     // current position in screen pixel space
        y: &mut f32,
        quad: &mut truetype::AlignedQuad, // output: quad to draw
        align_to_integer: i32,
    );

    pub fn stbtt_InitFont(info: *mut truetype::FontInfo, data: *const u8, offset: i32) -> i32;

    pub fn stbtt_FindGlyphIndex(info: &truetype::FontInfo, unicode_codepoint: i32) -> i32;

    pub fn stbtt_ScaleForPixelHeight(info: &truetype::FontInfo, height: f32) -> f32;

    pub fn stbtt_GetNumberOfFonts(data: *const u8) -> i32;

    pub fn stbtt_GetFontOffsetForIndex(data: *const u8, index: i32) -> i32;

    pub fn stbtt_GetCodepointBitmap(
        info: &truetype::FontInfo,
        scale_x: f32,
        scale_y: f32,
        codepoint: i32,
        width: &mut i32,
        height: &mut i32,
        x_offset: &mut i32,
        y_offset: &mut i32,
    ) -> *mut u8;

    pub fn stbtt_MakeCodepointBitmap(
        info: &truetype::FontInfo,
        output: *mut u8,
        out_w: i32,
        out_h: i32,
        out_stride: i32,
        scale_x: f32,
        scale_y: f32,
        codepoint: i32,
    );

    pub fn stbtt_MakeCodepointBitmapSubpixel(
        info: &truetype::FontInfo,
        output: *mut u8,
        out_w: i32,
        out_h: i32,
        out_stride: i32,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
        codepoint: i32,
    );

    pub fn stbtt_MakeCodepointBitmapSubpixelPrefilter(
        info: &truetype::FontInfo,
        output: *mut u8,
        out_w: i32,
        out_h: i32,
        out_stride: i32,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
        oversample_x: i32,
        oversample_y: i32,
        sub_x: &mut f32,
        sub_y: &mut f32,
        codepoint: i32,
    );

    pub fn stbtt_GetCodepointBitmapBox(
        info: &truetype::FontInfo,
        codepoint: i32,
        scale_x: f32,
        scale_y: f32,
        x0: &mut i32,
        y0: &mut i32,
        x1: &mut i32,
        y1: &mut i32,
    );

    pub fn stbtt_GetCodepointBitmapBoxSubpixel(
        info: &truetype::FontInfo,
        codepoint: i32,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
        x0: &mut i32,
        y0: &mut i32,
        x1: &mut i32,
        y1: &mut i32,
    );

    pub fn stbtt_GetGlyphBitmap(
        info: &truetype::FontInfo,
        scale_x: f32,
        scale_y: f32,
        glyph: i32,
        width: &mut i32,
        height: &mut i32,
        x_offset: &mut i32,
        y_offset: &mut i32,
    ) -> *mut u8;

    pub fn stbtt_GetGlyphBitmapSubpixel(
        info: &truetype::FontInfo,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
        glyph: i32,
        width: &mut i32,
        height: &mut i32,
        x_offset: &mut i32,
        y_offset: &mut i32,
    ) -> *mut u8;

    pub fn stbtt_MakeGlyphBitmap(
        info: &truetype::FontInfo,
        output: *mut u8,
        out_w: i32,
        out_h: i32,
        out_stride: i32,
        scale_x: f32,
        scale_y: f32,
        glyph: i32,
    );

    pub fn stbtt_MakeGlyphBitmapSubpixel(
        info: &truetype::FontInfo,
        output: *mut u8,
        out_w: i32,
        out_h: i32,
        out_stride: i32,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
        glyph: i32,
    );

    pub fn stbtt_MakeGlyphBitmapSubpixelPrefilter(
        info: &truetype::FontInfo,
        output: *mut u8,
        out_w: i32,
        out_h: i32,
        out_stride: i32,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
        oversample_x: i32,
        oversample_y: i32,
        sub_x: &mut f32,
        sub_y: &mut f32,
        glyph: i32,
    );

    pub fn stbtt_GetGlyphBitmapBox(
        info: &truetype::FontInfo,
        glyph: i32,
        scale_x: f32,
        scale_y: f32,
        x0: &mut i32,
        y0: &mut i32,
        x1: &mut i32,
        y1: &mut i32,
    );

    pub fn stbtt_GetGlyphBitmapBoxSubpixel(
        info: &truetype::FontInfo,
        glyph: i32,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
        x0: &mut i32,
        y0: &mut i32,
        x1: &mut i32,
        y1: &mut i32,
    );

    pub fn stbtt_GetCodepointHMetrics(
        info: &truetype::FontInfo,
        codepoint: i32,
        advance_width: &mut i32,
        left_side_bearing: &mut i32,
    );

    pub fn stbtt_GetFontVMetrics(
        info: &truetype::FontInfo,
        ascent: &mut i32,
        descent: &mut i32,
        line_gap: &mut i32,
    );

    pub fn stbtt_GetFontVMetricsOS2(
        info: &truetype::FontInfo,
        typo_ascent: &mut i32,
        typo_descent: &mut i32,
        typo_line_gap: &mut i32,
    ) -> i32;

    pub fn stbtt_GetCodepointKernAdvance(info: &truetype::FontInfo, ch1: i32, ch2: i32) -> i32;

    pub fn stbtt_GetGlyphHMetrics(
        info: &truetype::FontInfo,
        glyph_index: i32,
        advance_width: &mut i32,
        left_side_bearing: &mut i32,
    );

    pub fn stbtt_GetGlyphKernAdvance(info: &truetype::FontInfo, glyph1: i32, glyph2: i32) -> i32;

    pub fn stbtt_GetGlyphBox(
        info: &truetype::FontInfo,
        glyph_index: i32,
        x0: &mut i32,
        y0: &mut i32,
        x1: &mut i32,
        y1: &mut i32,
    ) -> i32;
}
