#ifndef GRAPHICS_BINDINGS_INCLUDE
#define GRAPHICS_BINDINGS_INCLUDE

const uint SAMPLER_BILINEAR = 0;
const uint SAMPLER_BILINEAR_UNNORMALIZED = 1;
const uint SAMPLER_COUNT = 2;

layout (set = 0, binding = 0) uniform sampler samplers[SAMPLER_COUNT];
layout (set = 0, binding = 1) uniform texture2D albedo;

#endif
