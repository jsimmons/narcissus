use narcissus_gpu::{
    BindDesc, BindGroupLayout, BindingType, ComputePipelineDesc, Pipeline, PipelineLayout,
    PushConstantRange, ShaderDesc, ShaderStageFlags,
};

use crate::Gpu;

use super::primitive_2d::PrimitiveUniforms;

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
            // Tile color buffer
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
        ]);

        let layout = &PipelineLayout {
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[PushConstantRange {
                stage_flags: ShaderStageFlags::COMPUTE,
                offset: 0,
                size: std::mem::size_of::<PrimitiveUniforms>() as u32,
            }],
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
