use narcissus_font::TouchedGlyphIndex;
use narcissus_gpu::{
    BindDesc, BindGroupLayout, BindingType, ComputePipelineDesc, Pipeline, ShaderDesc,
    ShaderStageFlags,
};

use crate::Gpu;

#[allow(unused)]
#[repr(C)]
pub struct PrimitiveUniforms {
    pub screen_width: u32,
    pub screen_height: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub num_primitives: u32,
}

#[allow(unused)]
#[repr(C)]
pub struct GlyphInstance {
    pub x: f32,
    pub y: f32,
    pub touched_glyph_index: TouchedGlyphIndex,
    pub color: u32,
}

pub struct Primitive2dPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline: Pipeline,
}

impl Primitive2dPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let bind_group_layout = gpu.create_bind_group_layout(&[
            // Uniforms
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::UniformBuffer),
            // Sampler
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::Sampler),
            // Glyph Atlas
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::SampledImage),
            // Glyphs
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // Glyph Instances
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // Primitive Instances
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // Tiles
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // UI
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
        ]);

        let pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_TILED_COMP_SPV,
            },
            bind_group_layouts: &[bind_group_layout],
        });

        Self {
            bind_group_layout,
            pipeline,
        }
    }
}
