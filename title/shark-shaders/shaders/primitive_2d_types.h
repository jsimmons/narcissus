
struct Glyph {
    ivec2 atlas_min;
    ivec2 atlas_max;

    vec2 offset_min;
    vec2 offset_max;
};

struct GlyphInstance {
    vec2 position;
    uint index;
    uint color;
};

struct PrimitiveInstance {
    uint type;
    uint index;
};
