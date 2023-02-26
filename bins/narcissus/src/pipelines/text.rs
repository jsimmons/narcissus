use narcissus_core::{cstr, default, include_bytes_align};
use narcissus_gpu::{
    BindGroupLayout, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType, BlendMode,
    CompareOp, CullingMode, Device, FrontFace, GraphicsPipelineDesc, GraphicsPipelineLayout,
    ImageFormat, Pipeline, PolygonMode, ShaderDesc, ShaderStageFlags, Topology,
};

const VERT_SPV: &'static [u8] = include_bytes_align!(4, "../shaders/text.vert.spv");
const FRAG_SPV: &'static [u8] = include_bytes_align!(4, "../shaders/text.frag.spv");

pub struct TextPipeline {
    pub uniforms_bind_group_layout: BindGroupLayout,
    pub storage_bind_group_layout: BindGroupLayout,
    pub pipeline: Pipeline,
}

impl TextPipeline {
    pub fn new(device: &dyn Device) -> Self {
        let uniforms_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDesc {
            entries: &[BindGroupLayoutEntryDesc {
                slot: 0,
                stages: ShaderStageFlags::ALL,
                binding_type: BindingType::UniformBuffer,
                count: 1,
            }],
        });

        let storage_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDesc {
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
                    binding_type: BindingType::Image,
                    count: 1,
                },
            ],
        });

        let pipeline = device.create_graphics_pipeline(&GraphicsPipelineDesc {
            vertex_shader: ShaderDesc {
                entry: cstr!("main"),
                code: VERT_SPV,
            },
            fragment_shader: ShaderDesc {
                entry: cstr!("main"),
                code: FRAG_SPV,
            },
            bind_group_layouts: &[uniforms_bind_group_layout, storage_bind_group_layout],
            layout: GraphicsPipelineLayout {
                color_attachment_formats: &[ImageFormat::BGRA8_SRGB],
                depth_attachment_format: Some(ImageFormat::DEPTH_F32),
                stencil_attachment_format: None,
            },
            topology: Topology::TriangleStrip,
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
            uniforms_bind_group_layout,
            storage_bind_group_layout,
            pipeline,
        }
    }
}
