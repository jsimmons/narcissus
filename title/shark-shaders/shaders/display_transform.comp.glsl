#version 460

layout (local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout (set = 0, binding = 0) uniform sampler linear_sampler;

layout (set = 0, binding = 1) uniform texture3D tony_mc_mapface_lut;
layout (set = 0, binding = 2, rgba16f) uniform readonly image2D render_target;
layout (set = 0, binding = 3, rgba16f) uniform writeonly image2D swapchain_image;

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
    return textureLod(sampler3D(tony_mc_mapface_lut, linear_sampler), uv, 0.0).rgb;
}

void main() {
    vec3 stimulus = imageLoad(render_target, ivec2(gl_GlobalInvocationID.xy)).rgb;
    vec3 srgb = srgb_oetf(tony_mc_mapface(stimulus));
    imageStore(swapchain_image, ivec2(gl_GlobalInvocationID.xy), vec4(srgb, 1.0));
}
