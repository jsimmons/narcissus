use narcissus_core::default;
use narcissus_font::TouchedGlyphIndex;
use narcissus_gpu::{
    BindGroupLayout, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType, BlendMode,
    CompareOp, CullingMode, FrontFace, GraphicsPipelineDesc, GraphicsPipelineLayout, ImageFormat,
    Pipeline, PolygonMode, Sampler, SamplerAddressMode, SamplerDesc, SamplerFilter, ShaderDesc,
    ShaderStageFlags, Topology,
};

use crate::Gpu;

#[allow(unused)]
#[repr(C)]
pub struct UiUniforms {
    pub screen_width: u32,
    pub screen_height: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
}

#[repr(u32)]
pub enum PrimitiveKind {
    Glyph,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PrimitiveVertex(u32);

impl PrimitiveVertex {
    #[inline(always)]
    pub fn glyph(corner: u32, index: u32) -> Self {
        let kind = PrimitiveKind::Glyph as u32;
        Self(kind << 26 | corner << 24 | index)
    }
}

#[allow(unused)]
#[repr(C)]
pub struct PrimitiveInstance {
    pub x: f32,
    pub y: f32,
    pub touched_glyph_index: TouchedGlyphIndex,
    pub color: u32,
}

pub struct UiPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline: Pipeline,
}

impl UiPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let bind_group_layout = gpu.create_bind_group_layout(&BindGroupLayoutDesc {
            entries: &[
                BindGroupLayoutEntryDesc {
                    slot: 0,
                    stages: ShaderStageFlags::ALL,
                    binding_type: BindingType::UniformBuffer,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 1,
                    stages: ShaderStageFlags::ALL,
                    binding_type: BindingType::StorageBuffer,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 2,
                    stages: ShaderStageFlags::ALL,
                    binding_type: BindingType::StorageBuffer,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 3,
                    stages: ShaderStageFlags::ALL,
                    binding_type: BindingType::StorageBuffer,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 4,
                    stages: ShaderStageFlags::ALL,
                    binding_type: BindingType::Sampler,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 5,
                    stages: ShaderStageFlags::ALL,
                    binding_type: BindingType::Image,
                    count: 1,
                },
            ],
        });

        let sampler = gpu.create_sampler(&SamplerDesc {
            filter: SamplerFilter::Bilinear,
            address_mode: SamplerAddressMode::Clamp,
            compare_op: None,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
        });

        let pipeline = gpu.create_graphics_pipeline(&GraphicsPipelineDesc {
            vertex_shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::UI_VERT_SPV,
            },
            fragment_shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::UI_FRAG_SPV,
            },
            bind_group_layouts: &[bind_group_layout],
            layout: GraphicsPipelineLayout {
                color_attachment_formats: &[ImageFormat::BGRA8_SRGB],
                depth_attachment_format: Some(ImageFormat::DEPTH_F32),
                stencil_attachment_format: None,
            },
            topology: Topology::Triangles,
            primitive_restart: false,
            polygon_mode: PolygonMode::Fill,
            culling_mode: CullingMode::None,
            front_face: FrontFace::CounterClockwise,
            blend_mode: BlendMode::Premultiplied,
            depth_bias: None,
            depth_compare_op: CompareOp::Always,
            depth_test_enable: false,
            depth_write_enable: false,
            stencil_test_enable: false,
            stencil_back: default(),
            stencil_front: default(),
        });

        Self {
            bind_group_layout,
            sampler,
            pipeline,
        }
    }
}
