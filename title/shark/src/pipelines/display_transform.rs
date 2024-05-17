use narcissus_font::TouchedGlyphIndex;
use narcissus_gpu::{
    BindDesc, BindGroupLayout, BindingType, ComputePipelineDesc, Pipeline, ShaderDesc,
    ShaderStageFlags,
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
        let bind_group_layout = gpu.create_bind_group_layout(&[
            // Uniforms
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::UniformBuffer),
            // Sampler
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::Sampler),
            // RT
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
            // Swapchain
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
            // Glyph Atlas
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::SampledImage),
            // Tony Mc'mapface LUT
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::SampledImage),
            // Glyphs
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // Glyph Instances
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
        ]);

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
