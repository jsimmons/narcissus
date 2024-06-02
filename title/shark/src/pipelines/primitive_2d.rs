use narcissus_font::TouchedGlyphIndex;
use narcissus_gpu::{
    BindDesc, BindGroupLayout, BindingType, ComputePipelineDesc, Pipeline, PipelineLayout,
    PushConstantRange, ShaderDesc, ShaderStageFlags,
};

use crate::Gpu;

pub const TILE_SIZE: u32 = 32;
pub const MAX_PRIMS: u32 = 1 << 18;
pub const TILE_BITMAP_WORDS_L1: u32 = MAX_PRIMS / 32 / 32;
pub const TILE_BITMAP_WORDS_L0: u32 = MAX_PRIMS / 32;
pub const TILE_STRIDE: u32 = TILE_BITMAP_WORDS_L0 + TILE_BITMAP_WORDS_L1 + 2;

#[allow(unused)]
#[repr(C)]
pub struct PrimitiveUniforms {
    pub screen_resolution_x: u32,
    pub screen_resolution_y: u32,
    pub atlas_resolution_x: u32,
    pub atlas_resolution_y: u32,

    pub num_primitives: u32,
    pub num_primitives_32: u32,
    pub num_primitives_1024: u32,

    pub tile_stride: u32,
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
    pub bin_clear_pipeline: Pipeline,
    pub bin_pipeline: Pipeline,
    pub rasterize_pipeline: Pipeline,
}

impl Primitive2dPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let bind_group_layout = gpu.create_bind_group_layout(&[
            // Sampler
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::Sampler),
            // Glyph Atlas
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::SampledImage),
            // Glyphs
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // Glyph Instances
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // Tiles
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // UI Image Output
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
        ]);

        let layout = &PipelineLayout {
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[PushConstantRange {
                stage_flags: ShaderStageFlags::COMPUTE,
                offset: 0,
                size: std::mem::size_of::<PrimitiveUniforms>() as u32,
            }],
        };

        let bin_clear_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_BIN_CLEAR_COMP_SPV,
            },
            layout,
        });

        let bin_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_BIN_COMP_SPV,
            },
            layout,
        });

        let rasterize_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_RASTERIZE_COMP_SPV,
            },
            layout,
        });

        Self {
            bind_group_layout,
            bin_clear_pipeline,
            bin_pipeline,
            rasterize_pipeline,
        }
    }
}
