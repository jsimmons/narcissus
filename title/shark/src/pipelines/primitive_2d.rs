use narcissus_font::TouchedGlyphIndex;
use narcissus_gpu::{
    BindDesc, BindGroupLayout, BindingType, ComputePipelineDesc, Pipeline, ShaderDesc,
    ShaderStageFlags,
};

use crate::Gpu;

pub const MAX_PRIMS: u32 = 0x20000;
pub const TILE_SIZE_COARSE: u32 = 128;
pub const TILE_SIZE_FINE: u32 = 16;
pub const TILE_BITMAP_WORDS_L1: u32 = MAX_PRIMS / 32 / 32;
pub const TILE_BITMAP_WORDS_L0: u32 = MAX_PRIMS / 32;
pub const TILE_STRIDE_COARSE: u32 = TILE_BITMAP_WORDS_L0;
pub const TILE_STRIDE_FINE: u32 = TILE_BITMAP_WORDS_L0 + TILE_BITMAP_WORDS_L1;

#[allow(unused)]
#[repr(C)]
pub struct PrimitiveUniforms {
    pub screen_resolution_x: u32,
    pub screen_resolution_y: u32,
    pub tile_resolution_coarse_x: u32,
    pub tile_resolution_coarse_y: u32,
    pub tile_resolution_fine_x: u32,
    pub tile_resolution_fine_y: u32,
    pub atlas_resolution_x: u32,
    pub atlas_resolution_y: u32,

    pub num_primitives: u32,
    pub num_primitives_32: u32,
    pub num_primitives_1024: u32,

    pub _pad0: u32,
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
    pub coarse_bin_pipeline: Pipeline,
    pub fine_bin_pipeline: Pipeline,
    pub rasterize_pipeline: Pipeline,
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
            // Coarse Tiles
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // Fine Tiles
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageBuffer),
            // UI Image Output
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
        ]);

        let coarse_bin_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_BIN_COARSE_COMP_SPV,
            },
            bind_group_layouts: &[bind_group_layout],
        });

        let fine_bin_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_BIN_FINE_COMP_SPV,
            },
            bind_group_layouts: &[bind_group_layout],
        });

        let rasterize_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_RASTERIZE_COMP_SPV,
            },
            bind_group_layouts: &[bind_group_layout],
        });

        Self {
            bind_group_layout,
            coarse_bin_pipeline,
            fine_bin_pipeline,
            rasterize_pipeline,
        }
    }
}
