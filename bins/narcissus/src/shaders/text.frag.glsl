#version 460

layout(set = 0, binding = 3) uniform sampler texSampler;
layout(set = 0, binding = 4) uniform texture2D tex;

layout(location = 0) in vec2 texcoord;
layout(location = 1) in vec4 color;
layout(location = 0) out vec4 outColor;

void main() {
    float coverage = texture(sampler2D(tex, texSampler), texcoord).r;
    outColor = color * coverage;
}
