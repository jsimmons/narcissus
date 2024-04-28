use narcissus_core::default;
use narcissus_font::{TouchedGlyph, TouchedGlyphIndex};
use narcissus_gpu::{
    Bind, BindGroupLayout, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType, BlendMode,
    BufferUsageFlags, CmdEncoder, CompareOp, CullingMode, Device, DeviceExt, Frame, FrontFace,
    GraphicsPipelineDesc, GraphicsPipelineLayout, Image, ImageFormat, ImageLayout, Pipeline,
    PolygonMode, Sampler, SamplerAddressMode, SamplerDesc, SamplerFilter, ShaderDesc,
    ShaderStageFlags, ThreadToken, Topology, TypedBind,
};
use shark_shaders;

#[allow(unused)]
#[repr(C)]
pub struct TextUniforms {
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

pub struct TextPipeline {
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,
    pipeline: Pipeline,
}

impl TextPipeline {
    pub fn new(device: &dyn Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDesc {
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

        let sampler = device.create_sampler(&SamplerDesc {
            filter: SamplerFilter::Bilinear,
            address_mode: SamplerAddressMode::Clamp,
            compare_op: None,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
        });

        let pipeline = device.create_graphics_pipeline(&GraphicsPipelineDesc {
            vertex_shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::TEXT_VERT_SPV,
            },
            fragment_shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::TEXT_FRAG_SPV,
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

    pub fn bind(
        &self,
        device: &(dyn Device + 'static),
        frame: &Frame,
        thread_token: &ThreadToken,
        cmd_buffer: &mut CmdEncoder,
        text_uniforms: &TextUniforms,
        primitive_vertices: &[PrimitiveVertex],
        touched_glyphs: &[TouchedGlyph],
        primitive_instances: &[PrimitiveInstance],
        atlas: Image,
    ) {
        let uniforms_buffer = device.request_transient_buffer_with_data(
            frame,
            thread_token,
            BufferUsageFlags::UNIFORM,
            text_uniforms,
        );

        let primitive_vertex_buffer = device.request_transient_buffer_with_data(
            frame,
            thread_token,
            BufferUsageFlags::STORAGE,
            primitive_vertices,
        );

        let cached_glyphs_buffer = device.request_transient_buffer_with_data(
            frame,
            thread_token,
            BufferUsageFlags::STORAGE,
            touched_glyphs,
        );
        let glyph_instance_buffer = device.request_transient_buffer_with_data(
            frame,
            thread_token,
            BufferUsageFlags::STORAGE,
            primitive_instances,
        );

        device.cmd_set_pipeline(cmd_buffer, self.pipeline);
        device.cmd_set_bind_group(
            frame,
            cmd_buffer,
            self.bind_group_layout,
            0,
            &[
                Bind {
                    binding: 0,
                    array_element: 0,
                    typed: TypedBind::UniformBuffer(&[uniforms_buffer.to_arg()]),
                },
                Bind {
                    binding: 1,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[primitive_vertex_buffer.to_arg()]),
                },
                Bind {
                    binding: 2,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[cached_glyphs_buffer.to_arg()]),
                },
                Bind {
                    binding: 3,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[glyph_instance_buffer.to_arg()]),
                },
                Bind {
                    binding: 4,
                    array_element: 0,
                    typed: TypedBind::Sampler(&[self.sampler]),
                },
                Bind {
                    binding: 5,
                    array_element: 0,
                    typed: TypedBind::Image(&[(ImageLayout::Optimal, atlas)]),
                },
            ],
        );
    }
}
