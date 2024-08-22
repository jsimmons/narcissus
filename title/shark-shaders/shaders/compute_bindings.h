#ifndef COMPUTE_BINDINGS_INCLUDE
#define COMPUTE_BINDINGS_INCLUDE

#include "primitive_2d.h"

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer PrimitiveInstances
{
    PrimitiveInstance values[];
};

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer Rects
{
    Rect values[];
};

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer Glyphs
{
    Glyph values[];
};

layout(buffer_reference, std430, buffer_reference_align = 4) readonly buffer TilesRead
{
    uint values[];
};

layout(buffer_reference, std430, buffer_reference_align = 4) writeonly buffer TilesWrite
{
    uint values[];
};

struct ComputeUniforms {
    uvec2 screen_resolution;
    uvec2 atlas_resolution;
    uvec2 tile_resolution;

    uint num_primitives;
    uint num_primitives_32;
    uint num_primitives_1024;
    uint tile_stride;

    PrimitiveInstances primitive_instances;
    Rects rects;
    Glyphs glyphs;
    TilesWrite tiles;
};

layout(std430, push_constant) uniform UniformBuffer {
    ComputeUniforms uniforms;
};

layout (set = 0, binding = 0) uniform sampler bilinear_sampler;
layout (set = 0, binding = 1) uniform texture3D tony_mc_mapface_lut;
layout (set = 0, binding = 2) uniform texture2D glyph_atlas;
layout (set = 0, binding = 3, rgba16f) uniform writeonly image2D ui_layer_write;
layout (set = 0, binding = 3, rgba16f) uniform readonly image2D ui_layer_read;
layout (set = 0, binding = 4, rgba16f) uniform readonly image2D color_layer;
layout (set = 0, binding = 5, rgba16f) uniform writeonly image2D composited_output;

#endif
