#!/bin/bash

pushd "${0%/*}"
glslc --target-env=vulkan1.3 -O -fshader-stage=vert -o basic.vert.spv basic.vert.glsl
glslc --target-env=vulkan1.3 -O -fshader-stage=frag -o basic.frag.spv basic.frag.glsl
echo "built shaders"
popd
