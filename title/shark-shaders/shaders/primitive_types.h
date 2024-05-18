
struct CachedGlyph {
    uint x0;
    uint x1;
    uint y0;
    uint y1;

    float offset_x0;
    float offset_x1;
    float offset_y0;
    float offset_y1;
};

struct GlyphInstance {
    float x;
    float y;
    uint index;
    uint color;
};
