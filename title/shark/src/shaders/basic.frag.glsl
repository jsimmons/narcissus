#version 460

layout(set = 1, binding = 2) uniform sampler texSampler;
layout(set = 1, binding = 3) uniform texture2D tex;

layout(location = 0) in vec2 texcoord;
layout(location = 1) in vec3 normal;
layout(location = 0) out vec4 outColor;

void main() {
    float NdotL = max(dot(normal, vec3(0.0, 1.0, 0.0)), 0.1f);
    vec3 rgb = texture(sampler2D(tex, texSampler), vec2(texcoord.x, texcoord.y)).rgb;
    outColor = vec4(rgb * NdotL, 1.0);
}
