#version 460

#extension GL_GOOGLE_include_directive : require

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

#include "primitive_2d_types.h"

layout(std430, set = 0, binding = 0) uniform uniformBuffer {
    uint screen_width;
    uint screen_height;
    uint atlas_width;
    uint atlas_height;
    uint num_primitives;
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

layout(std430, set = 0, binding = 6) readonly buffer tileBuffer {
    Tile tiles[];
};

layout (set = 0, binding = 7, rgba16f) uniform writeonly image2D ui_image;

layout (local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

void main() {
    vec4 accum = vec4(0.0);

    for (int i = 0; i < num_primitives; i++) {
        const GlyphInstance gi = glyph_instances[i];
        const Glyph gl = glyphs[gi.index];
        const vec4 color = unpackUnorm4x8(gi.color).bgra;
        const vec2 glyph_top_left = vec2(gi.x + gl.offset_x0, gi.y + gl.offset_y0);
        const vec2 glyph_bottom_right = vec2(gi.x + gl.offset_x1, gi.y + gl.offset_y1);
        const vec2 glyph_size = vec2(gl.offset_x1 - gl.offset_x0, gl.offset_y1 - gl.offset_y0);
        const vec2 sample_center = gl_GlobalInvocationID.xy; // half pixel offset goes here?
        if (sample_center.x >= glyph_top_left.x &&
            sample_center.x <= glyph_bottom_right.x &&
            sample_center.y >= glyph_top_left.y &&
            sample_center.y <= glyph_bottom_right.y) {
            const vec2 uv = mix(vec2(gl.x0, gl.y0), vec2(gl.x1, gl.y1), (sample_center - glyph_top_left) / glyph_size) / vec2(atlas_width, atlas_height);
            const float coverage = textureLod(sampler2D(glyph_atlas, bilinear_sampler), uv, 0.0).r;
            accum = coverage * color;
            accum.a = coverage;
            break;
        }
    }

    imageStore(ui_image, ivec2(gl_GlobalInvocationID.xy), accum);
}
