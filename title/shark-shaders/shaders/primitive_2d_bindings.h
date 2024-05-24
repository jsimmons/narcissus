

layout(std430, set = 0, binding = 0) uniform uniformBuffer {
    PrimitiveUniforms primitive_uniforms; 
};

layout (set = 0, binding = 1) uniform sampler bilinear_sampler;

layout (set = 0, binding = 2) uniform texture2D glyph_atlas;

layout(std430, set = 0, binding = 3) readonly buffer glyphBuffer {
    Glyph glyphs[];
};

layout(std430, set = 0, binding = 4) readonly buffer glyphInstanceBuffer {
    GlyphInstance glyph_instances[];
};

layout(std430, set = 0, binding = 5) readonly buffer primitiveInstanceBuffer {
    PrimitiveInstance primitive_instances[];
};

layout(std430, set = 0, binding = 6) readonly buffer coarseTileBufferRead {
    uint coarse_bitmap_ro[];
};

layout(std430, set = 0, binding = 6) writeonly buffer coarseTileBufferWrite {
    uint coarse_bitmap_wo[];
};

layout(std430, set = 0, binding = 7) readonly buffer fineTileBufferRead {
    uint fine_bitmap_ro[];
};

layout(std430, set = 0, binding = 7) writeonly buffer fineTileBufferWrite {
    uint fine_bitmap_wo[];
};

layout (set = 0, binding = 8, rgba16f) uniform writeonly image2D ui_image;
