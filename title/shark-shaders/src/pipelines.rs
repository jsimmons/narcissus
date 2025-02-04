use narcissus_core::default;
use narcissus_font::TouchedGlyphIndex;
use narcissus_gpu::{
    BindDesc, BindGroupLayout, BindingType, BlendMode, BufferAddress, CompareOp,
    ComputePipelineDesc, CullingMode, FrontFace, Gpu, GraphicsPipelineAttachments,
    GraphicsPipelineDesc, ImageFormat, Pipeline, PipelineLayout, PolygonMode, PushConstantRange,
    Sampler, SamplerAddressMode, SamplerDesc, SamplerFilter, ShaderDesc, ShaderStageFlags,
    SpecConstant, Topology,
};
use narcissus_maths::{Mat4, Vec2};

pub const DRAW_2D_TILE_SIZE: u32 = 32;

#[allow(unused)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub texcoord: [f32; 4],
}

#[repr(u8)]
enum Draw2dCmdType {
    Rect,
    Glyph,
}

#[allow(unused)]
#[repr(C)]
pub union Draw2dCmd {
    rect: CmdRect,
    glyph: CmdGlyph,
}

const _: () = assert!(std::mem::size_of::<Draw2dCmd>() == 32);

#[repr(C)]
#[derive(Clone, Copy)]
struct CmdGlyph {
    packed: u32,
    index: u32,
    position: Vec2,
    color: u32,
}

const _: () = assert!(std::mem::size_of::<CmdGlyph>() == 20);

#[repr(C)]
#[derive(Clone, Copy)]
struct CmdRect {
    /// 31       .          .          .          0
    ///  tttt tttt  bbbb bbbb  ssss ssss  ssss ssss
    ///
    /// t: Type
    /// b: Border width
    /// s: Scissor index
    packed: u32,

    position: Vec2,
    bound: Vec2,

    border_radii: u32,
    border_color: u32,

    background_color: u32,
}

const _: () = assert!(std::mem::size_of::<CmdRect>() == 32);

impl Draw2dCmd {
    #[inline(always)]
    pub fn glyph(
        scissor_index: u32,
        touched_glyph_index: TouchedGlyphIndex,
        color: u32,
        position: Vec2,
    ) -> Self {
        let glyph_index = touched_glyph_index.as_u32();
        Self {
            glyph: CmdGlyph {
                packed: ((Draw2dCmdType::Glyph as u32) << 24) | (scissor_index & 0xffff),
                index: glyph_index,
                position,
                color,
            },
        }
    }

    #[inline(always)]
    pub fn rect(
        scissor_index: u32,
        position: Vec2,
        bound: Vec2,
        border_width: u8,
        border_radii: [u8; 4],
        border_color: u32,
        background_color: u32,
    ) -> Self {
        Self {
            rect: CmdRect {
                packed: ((Draw2dCmdType::Rect as u32) << 24)
                    | ((border_width as u32) << 16)
                    | (scissor_index & 0xffff),
                position,
                bound,
                border_radii: u32::from_ne_bytes(border_radii),
                background_color,
                border_color,
            },
        }
    }
}

#[repr(C)]
pub struct Draw2dScissor {
    pub offset_min: Vec2,
    pub offset_max: Vec2,
}

pub struct Samplers {
    pub bilinear: Sampler,
    pub bilinear_unnormalized: Sampler,
}

impl Samplers {
    fn load(gpu: &Gpu) -> Samplers {
        let bilinear = gpu.create_sampler(&SamplerDesc {
            filter: SamplerFilter::Bilinear,
            address_mode: SamplerAddressMode::Clamp,
            compare_op: None,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
            unnormalized_coordinates: false,
        });

        let bilinear_unnormalized = gpu.create_sampler(&SamplerDesc {
            filter: SamplerFilter::Bilinear,
            address_mode: SamplerAddressMode::Clamp,
            compare_op: None,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
            unnormalized_coordinates: true,
        });

        Samplers {
            bilinear,
            bilinear_unnormalized,
        }
    }
}

pub enum GraphicsBinds {
    #[allow(unused)]
    ImmutableSamplers,
    Albedo,
}

pub enum ComputeBinds {
    #[allow(unused)]
    ImmutableSamplers,
    TonyMcMapfaceLut,
    GlyphAtlas,
    UiRenderTarget,
    ColorRenderTarget,
    CompositedRenderTarget,
}

#[repr(C)]
pub struct BasicConstants<'a> {
    pub clip_from_model: Mat4,
    pub vertex_buffer_address: BufferAddress<'a>,
    pub transform_buffer_address: BufferAddress<'a>,
}

#[repr(C)]
pub struct Draw2dClearConstants<'a> {
    pub finished_buffer_address: BufferAddress<'a>,
    pub coarse_buffer_address: BufferAddress<'a>,
}

#[repr(C)]
pub struct Draw2dScatterConstants<'a> {
    pub tile_resolution_x: u32,
    pub tile_resolution_y: u32,

    pub draw_buffer_len: u32,
    pub coarse_buffer_len: u32,

    pub draw_buffer_address: BufferAddress<'a>,
    pub scissor_buffer_address: BufferAddress<'a>,
    pub glyph_buffer_address: BufferAddress<'a>,
    pub coarse_buffer_address: BufferAddress<'a>,
}

#[repr(C)]
pub struct Draw2dSortConstants<'a> {
    pub coarse_buffer_len: u32,
    pub _pad: u32,
    pub indirect_dispatch_buffer_address: BufferAddress<'a>,
    pub coarse_buffer_address: BufferAddress<'a>,
}

#[repr(C)]
pub struct Draw2dResolveConstants<'a> {
    pub tile_stride: u32,
    pub draw_buffer_len: u32,

    pub draw_buffer_address: BufferAddress<'a>,
    pub scissor_buffer_address: BufferAddress<'a>,
    pub glyph_buffer_address: BufferAddress<'a>,
    pub coarse_buffer_address: BufferAddress<'a>,
    pub fine_buffer_address: BufferAddress<'a>,
    pub tile_buffer_address: BufferAddress<'a>,
}

#[repr(C)]
pub struct Draw2dRasterizeConstants<'a> {
    pub tile_stride: u32,
    pub _pad: u32,

    pub draw_buffer_address: BufferAddress<'a>,
    pub scissor_buffer_address: BufferAddress<'a>,
    pub glyph_buffer_address: BufferAddress<'a>,
    pub coarse_buffer_address: BufferAddress<'a>,
    pub fine_buffer_address: BufferAddress<'a>,
    pub tile_buffer_address: BufferAddress<'a>,
}

#[repr(C)]
pub struct CompositeConstants<'a> {
    pub tile_resolution_x: u32,
    pub tile_resolution_y: u32,
    pub tile_buffer_address: BufferAddress<'a>,
}

#[repr(C)]
pub struct RadixSortUpsweepConstants<'a> {
    pub shift: u32,
    pub _pad: u32,
    pub finished_buffer_address: BufferAddress<'a>,
    pub count_buffer_address: BufferAddress<'a>,
    pub src_buffer_address: BufferAddress<'a>,
    pub spine_buffer_address: BufferAddress<'a>,
}

#[repr(C)]
pub struct RadixSortDownsweepConstants<'a> {
    pub shift: u32,
    pub _pad: u32,
    pub count_buffer_address: BufferAddress<'a>,
    pub spine_buffer_address: BufferAddress<'a>,
    pub src_buffer_address: BufferAddress<'a>,
    pub dst_buffer_address: BufferAddress<'a>,
}

pub const RADIX_ITEMS_PER_WGP: usize = 4096;
pub const RADIX_DIGITS: usize = 256;

pub fn calcuate_workgroup_count(count: usize) -> usize {
    count.div_ceil(RADIX_ITEMS_PER_WGP)
}

/// Returns the size of the spine required to sort the given count in units of u32 words.
pub fn calculate_spine_size(count: usize) -> usize {
    calcuate_workgroup_count(count) * RADIX_DIGITS
}

pub struct Pipelines {
    _samplers: Samplers,

    pub graphics_bind_group_layout: BindGroupLayout,
    pub compute_bind_group_layout: BindGroupLayout,

    pub basic_pipeline: Pipeline,

    pub draw_2d_bin_0_clear_pipeline: Pipeline,
    pub draw_2d_bin_1_scatter_pipeline_workgroup_size: u32,
    pub draw_2d_bin_1_scatter_pipeline: Pipeline,
    pub draw_2d_bin_2_sort_pipeline: Pipeline,
    pub draw_2d_bin_3_resolve_pipeline: Pipeline,
    pub draw_2d_rasterize_pipeline: Pipeline,

    pub radix_sort_0_upsweep_pipeline: Pipeline,
    pub radix_sort_1_downsweep_pipeline: Pipeline,

    pub composite_pipeline: Pipeline,
}

impl Pipelines {
    pub fn load(gpu: &Gpu) -> Self {
        let samplers = Samplers::load(gpu);
        let immutable_samplers = &[samplers.bilinear, samplers.bilinear_unnormalized];

        let graphics_bind_group_layout = gpu.create_bind_group_layout(&[
            // Samplers
            BindDesc::with_immutable_samplers(ShaderStageFlags::FRAGMENT, immutable_samplers),
            // Albedo
            BindDesc::new(ShaderStageFlags::FRAGMENT, BindingType::SampledImage),
        ]);

        gpu.debug_name_bind_group_layout(graphics_bind_group_layout, "graphics");

        let compute_bind_group_layout = gpu.create_bind_group_layout(&[
            // Samplers
            BindDesc::with_immutable_samplers(ShaderStageFlags::COMPUTE, immutable_samplers),
            // Tony mc mapface LUT
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::SampledImage),
            // Glyph Atlas
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::SampledImage),
            // UI Render Target
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
            // Color Render Target
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
            // Composited Render Target
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
        ]);

        gpu.debug_name_bind_group_layout(compute_bind_group_layout, "compute");

        let basic_pipeline = gpu.create_graphics_pipeline(&GraphicsPipelineDesc {
            vertex_shader: ShaderDesc {
                code: crate::BASIC_VERT_SPV,
                ..default()
            },
            fragment_shader: ShaderDesc {
                code: crate::BASIC_FRAG_SPV,
                ..default()
            },
            layout: PipelineLayout {
                bind_group_layouts: &[graphics_bind_group_layout],
                push_constant_ranges: &[PushConstantRange {
                    stage_flags: ShaderStageFlags::VERTEX,
                    offset: 0,
                    size: std::mem::size_of::<BasicConstants>() as u32,
                }],
            },
            attachments: GraphicsPipelineAttachments {
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

        gpu.debug_name_pipeline(basic_pipeline, "basic");

        let create_compute_pipeline =
            |code, name, workgroup_size, require_full_subgroups, push_constant_size| {
                let push_constant_range = PushConstantRange {
                    stage_flags: ShaderStageFlags::COMPUTE,
                    offset: 0,
                    size: push_constant_size as u32,
                };

                let pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
                    shader: ShaderDesc {
                        code,
                        require_full_subgroups,
                        required_subgroup_size: if workgroup_size != 0 {
                            Some(workgroup_size)
                        } else {
                            None
                        },
                        spec_constants: &[SpecConstant::U32 {
                            id: 0,
                            value: workgroup_size,
                        }],
                        ..default()
                    },
                    layout: PipelineLayout {
                        bind_group_layouts: &[compute_bind_group_layout],
                        // Validation cries about push constant ranges with zero size.
                        push_constant_ranges: if push_constant_range.size != 0 {
                            std::slice::from_ref(&push_constant_range)
                        } else {
                            &[]
                        },
                    },
                });

                gpu.debug_name_pipeline(pipeline, name);

                pipeline
            };

        let draw_2d_bin_0_clear_pipeline = create_compute_pipeline(
            crate::DRAW_2D_BIN_0_CLEAR_COMP_SPV,
            "draw2d_bin_clear",
            0,
            false,
            std::mem::size_of::<Draw2dClearConstants>(),
        );

        let draw_2d_bin_1_scatter_pipeline_workgroup_size = 32;
        let draw_2d_bin_1_scatter_pipeline = create_compute_pipeline(
            crate::DRAW_2D_BIN_1_SCATTER_COMP_SPV,
            "draw2d_bin_scatter",
            draw_2d_bin_1_scatter_pipeline_workgroup_size,
            true,
            std::mem::size_of::<Draw2dScatterConstants>(),
        );

        let draw_2d_bin_2_sort_pipeline = create_compute_pipeline(
            crate::DRAW_2D_BIN_2_SORT_COMP_SPV,
            "draw2d_bin_sort",
            0,
            false,
            std::mem::size_of::<Draw2dSortConstants>(),
        );

        let draw_2d_bin_3_resolve_pipeline = create_compute_pipeline(
            crate::DRAW_2D_BIN_3_RESOLVE_COMP_SPV,
            "draw2d_bin_resolve",
            32,
            true,
            std::mem::size_of::<Draw2dResolveConstants>(),
        );

        let draw_2d_rasterize_pipeline = create_compute_pipeline(
            crate::DRAW_2D_RASTERIZE_COMP_SPV,
            "draw2d_rasterize",
            0,
            false,
            std::mem::size_of::<Draw2dRasterizeConstants>(),
        );

        let radix_sort_0_upsweep_pipeline = create_compute_pipeline(
            crate::RADIX_SORT_0_UPSWEEP_COMP_SPV,
            "radix_sort_upsweep",
            32,
            true,
            std::mem::size_of::<RadixSortUpsweepConstants>(),
        );

        let radix_sort_1_downsweep_pipeline = create_compute_pipeline(
            crate::RADIX_SORT_1_DOWNSWEEP_COMP_SPV,
            "radix_sort_downsweep",
            32,
            true,
            std::mem::size_of::<RadixSortDownsweepConstants>(),
        );

        let composite_pipeline = create_compute_pipeline(
            crate::COMPOSITE_COMP_SPV,
            "composite",
            0,
            false,
            std::mem::size_of::<CompositeConstants>(),
        );

        Self {
            _samplers: samplers,

            graphics_bind_group_layout,
            compute_bind_group_layout,

            basic_pipeline,

            draw_2d_bin_0_clear_pipeline,
            draw_2d_bin_1_scatter_pipeline_workgroup_size,
            draw_2d_bin_1_scatter_pipeline,
            draw_2d_bin_2_sort_pipeline,
            draw_2d_bin_3_resolve_pipeline,
            draw_2d_rasterize_pipeline,

            radix_sort_0_upsweep_pipeline,
            radix_sort_1_downsweep_pipeline,

            composite_pipeline,
        }
    }
}
