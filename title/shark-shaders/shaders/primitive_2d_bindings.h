

layout(std430, push_constant) uniform uniformBuffer {
    PrimitiveUniforms primitive_uniforms; 
};

layout (set = 0, binding = 0) uniform sampler bilinear_sampler;

layout (set = 0, binding = 1) uniform texture2D glyph_atlas;

layout(std430, set = 0, binding = 2) readonly buffer glyphBuffer {
    Glyph glyphs[];
};

layout(std430, set = 0, binding = 3) readonly buffer glyphInstanceBuffer {
    GlyphInstance glyph_instances[];
};

layout(std430, set = 0, binding = 4) readonly buffer tileBufferRead {
    uint tile_bitmap_ro[];
};

layout(std430, set = 0, binding = 4) writeonly buffer tileBufferWrite {
    uint tile_bitmap_wo[];
};

layout (set = 0, binding = 5, rgba16f) uniform writeonly image2D ui_image;
