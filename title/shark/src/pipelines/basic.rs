use narcissus_core::default;
use narcissus_gpu::{
    BindGroupLayout, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType, BlendMode,
    CompareOp, CullingMode, FrontFace, GraphicsPipelineDesc, GraphicsPipelineLayout, ImageFormat,
    Pipeline, PolygonMode, ShaderDesc, ShaderStageFlags, Topology,
};
use narcissus_maths::Mat4;

use crate::Gpu;

#[allow(unused)]
#[repr(C)]
pub struct BasicUniforms {
    pub clip_from_model: Mat4,
}

#[allow(unused)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub texcoord: [f32; 4],
}

pub struct BasicPipeline {
    pub uniforms_bind_group_layout: BindGroupLayout,
    pub storage_bind_group_layout: BindGroupLayout,
    pub pipeline: Pipeline,
}

impl BasicPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let uniforms_bind_group_layout = gpu.create_bind_group_layout(&BindGroupLayoutDesc {
            entries: &[BindGroupLayoutEntryDesc {
                slot: 0,
                stages: ShaderStageFlags::ALL,
                binding_type: BindingType::UniformBuffer,
                count: 1,
            }],
        });

        let storage_bind_group_layout = gpu.create_bind_group_layout(&BindGroupLayoutDesc {
            entries: &[
                BindGroupLayoutEntryDesc {
                    slot: 0,
                    stages: ShaderStageFlags::ALL,
                    binding_type: BindingType::StorageBuffer,
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
                    binding_type: BindingType::Sampler,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 3,
                    stages: ShaderStageFlags::ALL,
                    binding_type: BindingType::SampledImage,
                    count: 1,
                },
            ],
        });

        let pipeline = gpu.create_graphics_pipeline(&GraphicsPipelineDesc {
            vertex_shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::BASIC_VERT_SPV,
            },
            fragment_shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::BASIC_FRAG_SPV,
            },
            bind_group_layouts: &[uniforms_bind_group_layout, storage_bind_group_layout],
            layout: GraphicsPipelineLayout {
                color_attachment_formats: &[ImageFormat::RGBA16_FLOAT],
                depth_attachment_format: Some(ImageFormat::DEPTH_F32),
                stencil_attachment_format: None,
            },
            topology: Topology::Triangles,
            primitive_restart: false,
            polygon_mode: PolygonMode::Fill,
            culling_mode: CullingMode::Back,
            front_face: FrontFace::CounterClockwise,
            blend_mode: BlendMode::Opaque,
            depth_bias: None,
            depth_compare_op: CompareOp::GreaterOrEqual,
            depth_test_enable: true,
            depth_write_enable: true,
            stencil_test_enable: false,
            stencil_back: default(),
            stencil_front: default(),
        });

        Self {
            uniforms_bind_group_layout,
            storage_bind_group_layout,
            pipeline,
        }
    }
}
