use narcissus_core::{cstr, default, include_bytes_align};
use narcissus_gpu::{
    Bind, BindGroupLayout, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType, BlendMode,
    Buffer, CmdBuffer, CompareOp, CullingMode, Device, Frame, FrontFace, GraphicsPipelineDesc,
    GraphicsPipelineLayout, Image, ImageFormat, ImageLayout, IndexType, Pipeline, PolygonMode,
    Sampler, SamplerAddressMode, SamplerDesc, SamplerFilter, ShaderDesc, ShaderStageFlags,
    ThreadToken, Topology, TypedBind,
};
use narcissus_maths::Mat4;

use crate::Blittable;

const VERT_SPV: &[u8] = include_bytes_align!(4, "../shaders/basic.vert.spv");
const FRAG_SPV: &[u8] = include_bytes_align!(4, "../shaders/basic.frag.spv");

#[allow(unused)]
#[repr(C)]
#[repr(align(16))]
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

unsafe impl Blittable for BasicUniforms {}
unsafe impl Blittable for Vertex {}

pub struct BasicPipeline {
    pub uniforms_bind_group_layout: BindGroupLayout,
    pub storage_bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline: Pipeline,
}

impl BasicPipeline {
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

        let sampler = device.create_sampler(&SamplerDesc {
            filter: SamplerFilter::Point,
            address_mode: SamplerAddressMode::Clamp,
            compare_op: None,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 1000.0,
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
            topology: Topology::Triangles,
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
            sampler,
            pipeline,
        }
    }

    pub fn bind(
        &self,
        device: &dyn Device,
        frame: &Frame,
        thread_token: &ThreadToken,
        cmd_buffer: &mut CmdBuffer,
        basic_uniforms: &BasicUniforms,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
        transform_buffer: Buffer,
        texture: Image,
    ) {
        let mut uniform_buffer = device.request_transient_uniform_buffer(
            frame,
            thread_token,
            std::mem::size_of::<BasicUniforms>(),
            std::mem::align_of::<BasicUniforms>(),
        );

        uniform_buffer.copy_from_slice(basic_uniforms.as_bytes());

        device.cmd_set_pipeline(cmd_buffer, self.pipeline);

        device.cmd_set_bind_group(
            frame,
            cmd_buffer,
            self.uniforms_bind_group_layout,
            0,
            &[Bind {
                binding: 0,
                array_element: 0,
                typed: TypedBind::UniformBuffer(&[uniform_buffer.into()]),
            }],
        );

        device.cmd_set_bind_group(
            frame,
            cmd_buffer,
            self.storage_bind_group_layout,
            1,
            &[
                Bind {
                    binding: 0,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[vertex_buffer.into()]),
                },
                Bind {
                    binding: 1,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[transform_buffer.into()]),
                },
                Bind {
                    binding: 2,
                    array_element: 0,
                    typed: TypedBind::Sampler(&[self.sampler]),
                },
                Bind {
                    binding: 3,
                    array_element: 0,
                    typed: TypedBind::Image(&[(ImageLayout::Optimal, texture)]),
                },
            ],
        );

        device.cmd_set_index_buffer(cmd_buffer, index_buffer, 0, IndexType::U16);
    }
}
