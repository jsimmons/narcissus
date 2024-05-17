#version 460

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_control_flow_attributes : require

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
    uint num_primitives;
};

layout (set = 0, binding = 1) uniform sampler bilinear_sampler;

layout (set = 0, binding = 2, rgba16f) uniform readonly image2D render_target;
layout (set = 0, binding = 3, rgba16f) uniform writeonly image2D swapchain_image;

layout (set = 0, binding = 4) uniform texture2D glyph_atlas;
layout (set = 0, binding = 5) uniform texture3D tony_mc_mapface_lut;

layout(std430, set = 0, binding = 6) readonly buffer glyphBuffer {
    CachedGlyph cached_glyphs[];
};

layout(std430, set = 0, binding = 7) readonly buffer glyphInstanceBuffer {
    GlyphInstance glyph_instances[];
};

float srgb_oetf(float a) {
    return (.0031308f >= a) ? 12.92f * a : 1.055f * pow(a, .4166666666666667f) - .055f;
}

vec3 srgb_oetf(vec3 a) {
	return vec3(srgb_oetf(a.r), srgb_oetf(a.g), srgb_oetf(a.b));
}

vec3 tony_mc_mapface(vec3 stimulus) {
    const vec3 encoded = stimulus / (stimulus + 1.0);
    const float LUT_DIMS = 48.0;
    const vec3 uv = (encoded * ((LUT_DIMS - 1.0) / LUT_DIMS) + 0.5 / LUT_DIMS);
    return textureLod(sampler3D(tony_mc_mapface_lut, bilinear_sampler), uv, 0.0).rgb;
}

layout (local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

void main() {
    vec4 accum = vec4(0.0);

    for (int i = 0; i < num_primitives; i++) {
        const GlyphInstance gi = glyph_instances[i];
        const CachedGlyph cg = cached_glyphs[gi.index];
        const vec4 color = unpackUnorm4x8(gi.color).bgra;
        const vec2 glyph_top_left = vec2(gi.x + cg.offset_x0, gi.y + cg.offset_y0);
        const vec2 glyph_bottom_right = vec2(gi.x + cg.offset_x1, gi.y + cg.offset_y1);
        const vec2 glyph_size = vec2(cg.offset_x1 - cg.offset_x0, cg.offset_y1 - cg.offset_y0);
        const vec2 sample_center = gl_GlobalInvocationID.xy; // half pixel offset goes here?
        if (sample_center.x >= glyph_top_left.x &&
            sample_center.x <= glyph_bottom_right.x &&
            sample_center.y >= glyph_top_left.y &&
            sample_center.y <= glyph_bottom_right.y) {
            const vec2 uv = mix(vec2(cg.x0, cg.y0), vec2(cg.x1, cg.y1), (sample_center - glyph_top_left) / glyph_size) / vec2(atlas_width, atlas_height);
            const float coverage = textureLod(sampler2D(glyph_atlas, bilinear_sampler), uv, 0.0).r;
            accum = coverage * color;
            accum.a = coverage;
            break;
        }
    }

    const vec3 stimulus = imageLoad(render_target, ivec2(gl_GlobalInvocationID.xy)).rgb;
    const vec3 transformed = tony_mc_mapface(stimulus);
    const vec3 srgb = srgb_oetf(transformed);
    const vec3 composited = accum.rgb + (srgb * (1.0 - accum.a));
    imageStore(swapchain_image, ivec2(gl_GlobalInvocationID.xy), vec4(composited, 1.0));
}
