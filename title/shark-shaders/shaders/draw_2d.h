#ifndef DRAW_2D_H
#define DRAW_2D_H

const uint TILE_SIZE = 32;

const uint DRAW_2D_CMD_RECT = 0;
const uint DRAW_2D_CMD_GLYPH = 1;

struct Glyph {
    ivec2 atlas_min;
    ivec2 atlas_max;

    vec2 offset_min;
    vec2 offset_max;
};

struct Draw2dCmd {
    uint type;
    uint words[7];
};

struct Draw2dCmdRect {
    uint border_width;
    vec2 position;
    vec2 half_extent;
    uint background_color;
    uint border_color;
};

struct Draw2dCmdGlyph {
    uint index;
    vec2 position;
    uint color;
};

Draw2dCmdRect decode_rect(Draw2dCmd cmd) {
    return Draw2dCmdRect(
        cmd.words[0],
        vec2(uintBitsToFloat(cmd.words[1]), uintBitsToFloat(cmd.words[2])),
        vec2(uintBitsToFloat(cmd.words[3]), uintBitsToFloat(cmd.words[4])),
        cmd.words[5],
        cmd.words[6]
    );
}

Draw2dCmdGlyph decode_glyph(Draw2dCmd cmd) {
    return Draw2dCmdGlyph(
        cmd.words[0],
        vec2(uintBitsToFloat(cmd.words[1]), uintBitsToFloat(cmd.words[2])),
        cmd.words[3]
    );
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

#endif