use narcissus_core::default;
use narcissus_gpu::{
    Bind, BindGroupLayout, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType, BlendMode,
    BufferUsageFlags, CmdEncoder, CompareOp, CullingMode, Device, DeviceExt, Frame, FrontFace,
    GraphicsPipelineDesc, GraphicsPipelineLayout, Image, ImageFormat, ImageLayout, IndexType,
    PersistentBuffer, Pipeline, PolygonMode, Sampler, SamplerAddressMode, SamplerDesc,
    SamplerFilter, ShaderDesc, ShaderStageFlags, ThreadToken, Topology, TypedBind,
};
use narcissus_maths::{Affine3, Mat4};

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
                entry: c"main",
                code: shark_shaders::BASIC_VERT_SPV,
            },
            fragment_shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::BASIC_FRAG_SPV,
            },
            bind_group_layouts: &[uniforms_bind_group_layout, storage_bind_group_layout],
            layout: GraphicsPipelineLayout {
                color_attachment_formats: &[ImageFormat::BGRA8_SRGB],
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
            sampler,
            pipeline,
        }
    }

    pub fn bind(
        &self,
        device: &(dyn Device + 'static),
        frame: &Frame,
        thread_token: &ThreadToken,
        cmd_buffer: &mut CmdEncoder,
        basic_uniforms: &BasicUniforms,
        vertex_buffer: &PersistentBuffer,
        index_buffer: &PersistentBuffer,
        transforms: &[Affine3],
        texture: Image,
    ) {
        let uniform_buffer = device.request_transient_buffer_with_data(
            frame,
            thread_token,
            BufferUsageFlags::UNIFORM,
            basic_uniforms,
        );

        let transform_buffer = device.request_transient_buffer_with_data(
            frame,
            thread_token,
            BufferUsageFlags::STORAGE,
            transforms,
        );

        device.cmd_set_pipeline(cmd_buffer, self.pipeline);

        device.cmd_set_bind_group(
            frame,
            cmd_buffer,
            self.uniforms_bind_group_layout,
            0,
            &[Bind {
                binding: 0,
                array_element: 0,
                typed: TypedBind::UniformBuffer(&[uniform_buffer.to_arg()]),
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
                    typed: TypedBind::StorageBuffer(&[vertex_buffer.to_arg()]),
                },
                Bind {
                    binding: 1,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[transform_buffer.to_arg()]),
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

        device.cmd_set_index_buffer(cmd_buffer, index_buffer.to_arg(), 0, IndexType::U16);
    }
}
