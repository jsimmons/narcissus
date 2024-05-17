#version 460

layout(set = 0, binding = 1) uniform sampler bilinear_sampler;
layout(set = 1, binding = 2) uniform texture2D albedo;

layout(location = 0) in vec2 tex_coord;
layout(location = 1) in vec3 normal;
layout(location = 0) out vec4 out_color;

void main() {
    const float n_dot_l = max(dot(normal, vec3(0.0, 1.0, 0.0)), 0.1);
    const vec3 rgb = texture(sampler2D(albedo, bilinear_sampler), vec2(tex_coord.x, tex_coord.y)).rgb;
    out_color = vec4(rgb * n_dot_l, 1.0);
}
