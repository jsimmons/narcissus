use narcissus_core::{cstr, default, include_bytes_align};
use narcissus_font::CachedGlyphIndex;
use narcissus_gpu::{
    Bind, BindGroupLayout, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType, BlendMode,
    Buffer, CmdBuffer, CompareOp, CullingMode, Device, Frame, FrontFace, GraphicsPipelineDesc,
    GraphicsPipelineLayout, Image, ImageFormat, ImageLayout, Pipeline, PolygonMode, Sampler,
    SamplerAddressMode, SamplerDesc, SamplerFilter, ShaderDesc, ShaderStageFlags, Topology,
    TypedBind,
};

use crate::Blittable;

const VERT_SPV: &'static [u8] = include_bytes_align!(4, "../shaders/text.vert.spv");
const FRAG_SPV: &'static [u8] = include_bytes_align!(4, "../shaders/text.frag.spv");

#[allow(unused)]
#[repr(C)]
pub struct TextUniforms {
    pub screen_width: u32,
    pub screen_height: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
}

#[allow(unused)]
#[repr(C)]
pub struct GlyphInstance {
    pub cached_glyph_index: CachedGlyphIndex,
    pub x: f32,
    pub y: f32,
    pub color: u32,
}

unsafe impl Blittable for TextUniforms {}
unsafe impl Blittable for GlyphInstance {}

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
                    binding_type: BindingType::Sampler,
                    count: 1,
                },
                BindGroupLayoutEntryDesc {
                    slot: 4,
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
                entry: cstr!("main"),
                code: VERT_SPV,
            },
            fragment_shader: ShaderDesc {
                entry: cstr!("main"),
                code: FRAG_SPV,
            },
            bind_group_layouts: &[bind_group_layout],
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
            bind_group_layout,
            sampler,
            pipeline,
        }
    }

    pub fn bind(
        &self,
        device: &dyn Device,
        frame: &Frame,
        cmd_buffer: &mut CmdBuffer,
        uniforms: Buffer,
        cached_glyphs: Buffer,
        glyph_instances: Buffer,
        atlas: Image,
    ) {
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
                    typed: TypedBind::UniformBuffer(&[uniforms]),
                },
                Bind {
                    binding: 1,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[cached_glyphs]),
                },
                Bind {
                    binding: 2,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[glyph_instances]),
                },
                Bind {
                    binding: 3,
                    array_element: 0,
                    typed: TypedBind::Sampler(&[self.sampler]),
                },
                Bind {
                    binding: 4,
                    array_element: 0,
                    typed: TypedBind::Image(&[(ImageLayout::Optimal, atlas)]),
                },
            ],
        );
    }
}
