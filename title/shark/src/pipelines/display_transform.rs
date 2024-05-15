use narcissus_gpu::{
    BindGroupLayout, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType,
    ComputePipelineDesc, Pipeline, ShaderDesc, ShaderStageFlags,
};

use crate::Gpu;

#[allow(unused)]
#[repr(C)]
pub struct DisplayTransformUniforms {
    pub width: u32,
    pub height: u32,
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
                    slot: 0,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::Sampler,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 1,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::SampledImage,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 2,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::StorageImage,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 3,
                    stages: ShaderStageFlags::COMPUTE,
                    binding_type: BindingType::StorageImage,
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
