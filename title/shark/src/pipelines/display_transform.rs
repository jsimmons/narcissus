use narcissus_font::TouchedGlyphIndex;
use narcissus_gpu::{
    BindGroupLayout, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType,
    ComputePipelineDesc, Pipeline, ShaderDesc, ShaderStageFlags,
};

use crate::Gpu;

#[allow(unused)]
#[repr(C)]
pub struct DisplayTransformUniforms {
    pub screen_width: u32,
    pub screen_height: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub num_primitives: u32,
}

#[allow(unused)]
#[repr(C)]
pub struct PrimitiveInstance {
    pub x: f32,
    pub y: f32,
    pub touched_glyph_index: TouchedGlyphIndex,
    pub color: u32,
}

pub struct DisplayTransformPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline: Pipeline,
}

impl DisplayTransformPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let bind_group_layout = gpu.create_bind_group_layout(&BindGroupLayoutDesc {
            entries: &[
                BindGroupLayoutEntryDesc {
                    // uniforms
                    slot: 0,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::UniformBuffer,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    // rt
                    slot: 1,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::StorageImage,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    // swapchain
                    slot: 2,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::StorageImage,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    // sampler
                    slot: 3,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::Sampler,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    // glyph atlas
                    slot: 4,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::SampledImage,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    // lut
                    slot: 5,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::SampledImage,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    // glyphs
                    slot: 6,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::StorageBuffer,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    // glyph instances
                    slot: 7,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::StorageBuffer,
                    count: 1,
                },
            ],
        });

        let pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::DISPLAY_TRANSFORM_COMP_SPV,
            },
            bind_group_layouts: &[bind_group_layout],
        });

        Self {
            bind_group_layout,
            pipeline,
        }
    }
}
