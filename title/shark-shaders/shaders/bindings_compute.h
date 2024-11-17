#ifndef COMPUTE_BINDINGS_INCLUDE
#define COMPUTE_BINDINGS_INCLUDE

const uint SAMPLER_BILINEAR = 0;
const uint SAMPLER_BILINEAR_UNNORMALIZED = 1;
const uint SAMPLER_COUNT = 2;

layout (set = 0, binding = 0) uniform sampler samplers[SAMPLER_COUNT];
layout (set = 0, binding = 1) uniform texture3D tony_mc_mapface_lut;
layout (set = 0, binding = 2) uniform texture2D glyph_atlas;
layout (set = 0, binding = 3, rgba16f) uniform writeonly image2D ui_layer_write;
layout (set = 0, binding = 3, rgba16f) uniform readonly image2D ui_layer_read;
layout (set = 0, binding = 4, rgba16f) uniform readonly image2D color_layer;
layout (set = 0, binding = 5, rgba16f) uniform writeonly image2D composited_output;

#endif
