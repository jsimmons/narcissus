use narcissus_gpu::{
    BindDesc, BindGroupLayout, BindingType, ComputePipelineDesc, Pipeline, PipelineLayout,
    ShaderDesc, ShaderStageFlags,
};

use crate::Gpu;

pub struct DisplayTransformPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline: Pipeline,
}

impl DisplayTransformPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let bind_group_layout = gpu.create_bind_group_layout(&[
            // Sampler
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::Sampler),
            // Tony Mc'mapface LUT
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::SampledImage),
            // Layer RT
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
            // Layer UI
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
            // Composited Output
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
        ]);

        let layout = &PipelineLayout {
            bind_group_layouts: &[bind_group_layout],
        };

        let pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::DISPLAY_TRANSFORM_COMP_SPV,
            },
            layout,
        });

        Self {
            bind_group_layout,
            pipeline,
        }
    }
}
