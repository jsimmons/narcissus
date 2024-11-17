#ifndef DRAW_2D_H
#define DRAW_2D_H

const uint TILE_SIZE = 32;

const uint DRAW_2D_CMD_RECT = 0;
const uint DRAW_2D_CMD_GLYPH = 1;

struct Tile {
    uint index_min;
    uint index_max;
};

struct Glyph {
    ivec2 atlas_min;
    ivec2 atlas_max;

    vec2 offset_min;
    vec2 offset_max;
};

struct Scissor {
    vec2 offset_min;
    vec2 offset_max;
};

struct Cmd {
    uint packed_type;
    uint words[7];
};

struct CmdRect {
    vec2 position;
    vec2 bound;

    uint border_radii;
    uint border_color;

    uint background_color;
};

struct CmdGlyph {
    uint index;
    vec2 position;
    uint color;
};

CmdRect decode_rect(Cmd cmd) {
    CmdRect rect = {
        { uintBitsToFloat(cmd.words[0]), uintBitsToFloat(cmd.words[1]) }, // position
        { uintBitsToFloat(cmd.words[2]), uintBitsToFloat(cmd.words[3]) }, // bound
        cmd.words[4], // border_radii
        cmd.words[5], // border_color
        cmd.words[6], // background_color
    };
    return rect;
}

CmdGlyph decode_glyph(Cmd cmd) {
    CmdGlyph glyph = {
        cmd.words[0], // index
        { uintBitsToFloat(cmd.words[1]), uintBitsToFloat(cmd.words[2]) }, // position
        cmd.words[3], // color
    };
    return glyph;
}

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer CommandRef {
    Cmd values[];
};

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer ScissorRef {
    Scissor values[];
};

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer GlyphRef {
    Glyph values[];
};

layout(buffer_reference, std430, buffer_reference_align = 4) buffer CoarseRef {
    uint values[];
};

layout(buffer_reference, std430, buffer_reference_align = 4) readonly buffer CoarseReadRef {
    uint values[];
};

layout(buffer_reference, std430, buffer_reference_align = 4) buffer FineRef {
    uint values[];
};

layout(buffer_reference, std430, buffer_reference_align = 4) readonly buffer FineReadRef {
    uint values[];
};

layout(buffer_reference, std430, buffer_reference_align = 16) buffer TileRef {
    Tile values[];
};

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer TileReadRef {
    Tile values[];
};

#endif