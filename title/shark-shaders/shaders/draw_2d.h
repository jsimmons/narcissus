#ifndef DRAW_2D_H
#define DRAW_2D_H

const uint TILE_SIZE = 32;

const uint DRAW_2D_CMD_RECT = 0;
const uint DRAW_2D_CMD_GLYPH = 1;

struct Tile {
    uint min_index;
    uint max_index;
};

struct Glyph {
    ivec2 atlas_min;
    ivec2 atlas_max;

    vec2 offset_min;
    vec2 offset_max;
};

struct Draw2dCmd {
    uint packed_type;
    uint words[7];
};

struct Draw2dCmdRect {
    vec2 bounds_min;
    vec2 bounds_max;

    uint border_radii;
    uint border_color;

    uint background_color;
};

struct Draw2dCmdGlyph {
    vec2 position;
    uint color;
};

Draw2dCmdRect decode_rect(Draw2dCmd cmd) {
    Draw2dCmdRect rect = {
        { uintBitsToFloat(cmd.words[0]), uintBitsToFloat(cmd.words[1]) }, // bounds_min
        { uintBitsToFloat(cmd.words[2]), uintBitsToFloat(cmd.words[3]) }, // bounds_max
        cmd.words[4], // border_radii
        cmd.words[5], // border_color
        cmd.words[6], // background_color
    };
    return rect;
}

Draw2dCmdGlyph decode_glyph(in Draw2dCmd cmd) {
    return Draw2dCmdGlyph(vec2(uintBitsToFloat(cmd.words[0]), uintBitsToFloat(cmd.words[1])), cmd.words[2]);
}

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer Draw2dCommandRef
{
    Draw2dCmd values[];
};

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer GlyphRef
{
    Glyph values[];
};

layout(buffer_reference, std430, buffer_reference_align = 4) buffer CoarseRef
{
    uint values[];
};

layout(buffer_reference, std430, buffer_reference_align = 4) buffer FineRef {
    uint values[];
};

layout(buffer_reference, std430, buffer_reference_align = 4) buffer TileRef {
    Tile values[];
};

#endif