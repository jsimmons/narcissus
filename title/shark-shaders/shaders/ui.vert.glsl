#version 460

#extension GL_EXT_scalar_block_layout : require

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

layout(std430, set = 0, binding = 0) uniform uniformBuffer {
    uint screen_width;
    uint screen_height;
    uint atlas_width;
    uint atlas_height;
};

layout(std430, set = 0, binding = 1) readonly buffer primitiveBuffer {
    uint primitive_vertices[];
};

layout(std430, set = 0, binding = 2) readonly buffer glyphBuffer {
    CachedGlyph cached_glyphs[];
};

layout(std430, set = 0, binding = 3) readonly buffer glyphInstanceBuffer {
    GlyphInstance glyph_instances[];
};

layout(location = 0) out vec2 out_texcoord;
layout(location = 1) out flat vec4 out_color;

void main() {
    uint primitive_packed = primitive_vertices[gl_VertexIndex];
    uint primitive_kind = bitfieldExtract(primitive_packed, 26, 6);
    uint primitive_data = bitfieldExtract(primitive_packed, 24, 2);
    uint instance_index = bitfieldExtract(primitive_packed, 0, 24);

    GlyphInstance gi = glyph_instances[instance_index];
    CachedGlyph cg = cached_glyphs[gi.index];

    vec2 positions[4] = {
        vec2(cg.offset_x0, cg.offset_y0),
        vec2(cg.offset_x0, cg.offset_y1),
        vec2(cg.offset_x1, cg.offset_y0),
        vec2(cg.offset_x1, cg.offset_y1)
    };

    vec2 position = positions[primitive_data];
    vec2 half_screen_size = vec2(screen_width, screen_height) / 2.0;
    vec2 glyph_position = vec2(gi.x, gi.y);
    vec2 vertex_position = (position + glyph_position) / half_screen_size - 1.0;
    gl_Position = vec4(vertex_position, 0.0, 1.0);

    vec2 texcoords[4] = {
        vec2(cg.x0, cg.y0),
        vec2(cg.x0, cg.y1),
        vec2(cg.x1, cg.y0),
        vec2(cg.x1, cg.y1)
    };

    vec2 texcoord = texcoords[primitive_data];
    vec4 color = unpackUnorm4x8(gi.color).bgra;

    out_texcoord = texcoord / vec2(atlas_width, atlas_height);
    out_color = color;
}
