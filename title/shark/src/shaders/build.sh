#!/bin/bash

pushd "${0%/*}"
glslc --target-env=vulkan1.3 -O -fshader-stage=vert -o basic.vert.spv basic.vert.glsl
glslc --target-env=vulkan1.3 -O -fshader-stage=frag -o basic.frag.spv basic.frag.glsl
glslc --target-env=vulkan1.3 -O -fshader-stage=vert -o text.vert.spv text.vert.glsl
glslc --target-env=vulkan1.3 -O -fshader-stage=frag -o text.frag.spv text.frag.glsl
echo "built shaders"
popd
