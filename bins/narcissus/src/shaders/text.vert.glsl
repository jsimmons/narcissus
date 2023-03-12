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
    uint screenWidth;
    uint screenHeight;
    uint atlasWidth;
    uint atlasHeight;
};

layout(std430, set = 0, binding = 1) readonly buffer glyphBuffer {
    CachedGlyph cachedGlyphs[];
};

layout(std430, set = 0, binding = 2) readonly buffer glyphInstanceBuffer {
    GlyphInstance glyphInstances[];
};

layout(location = 0) out vec2 outTexcoord;
layout(location = 1) out flat vec4 outColor;

void main() {
    GlyphInstance gi = glyphInstances[gl_InstanceIndex];
    CachedGlyph cg = cachedGlyphs[gi.index];

    vec2 positions[4] = {
        vec2(cg.offset_x0, cg.offset_y0),
        vec2(cg.offset_x0, cg.offset_y1),
        vec2(cg.offset_x1, cg.offset_y0),
        vec2(cg.offset_x1, cg.offset_y1)
    };

    vec2 position = positions[gl_VertexIndex];
    vec2 halfScreenSize = vec2(screenWidth, screenHeight) / 2.0;
    vec2 glyphPosition = vec2(gi.x, gi.y);
    vec2 vertexPosition = (position + glyphPosition) / halfScreenSize - 1.0;
    gl_Position = vec4(vertexPosition, 0.0, 1.0);

    vec2 texcoords[4] = {
        vec2(cg.x0, cg.y0),
        vec2(cg.x0, cg.y1),
        vec2(cg.x1, cg.y0),
        vec2(cg.x1, cg.y1)
    };

    vec2 texcoord = texcoords[gl_VertexIndex];
    outTexcoord = texcoord / vec2(atlasWidth, atlasHeight);

    vec4 color = unpackUnorm4x8(gi.color).bgra;
    outColor = color;
}
