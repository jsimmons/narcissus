use std::{
    cell::UnsafeCell,
    collections::{hash_map::Entry, HashMap, VecDeque},
    marker::PhantomData,
    os::raw::{c_char, c_void},
    ptr::NonNull,
    sync::atomic::{AtomicU64, Ordering},
};

use narcissus_core::{
    cstr, cstr_from_bytes_until_nul, default, manual_arc,
    manual_arc::ManualArc,
    raw_window::{AsRawWindow, RawWindow},
    Arena, HybridArena, Mutex, PhantomUnsend, Pool,
};

use vulkan_sys as vk;

use crate::{
    delay_queue::DelayQueue, frame_counter::FrameCounter, Access, Bind, BindGroupLayout,
    BindGroupLayoutDesc, BindingType, BlendMode, Buffer, BufferDesc, BufferImageCopy,
    BufferUsageFlags, ClearValue, CmdBuffer, CompareOp, ComputePipelineDesc, CullingMode, Device,
    Extent2d, Extent3d, Frame, FrontFace, GlobalBarrier, GpuConcurrent, GraphicsPipelineDesc,
    Image, ImageAspectFlags, ImageBarrier, ImageBlit, ImageDesc, ImageDimension, ImageFormat,
    ImageLayout, ImageSubresourceLayers, ImageSubresourceRange, ImageUsageFlags, ImageViewDesc,
    IndexType, LoadOp, MemoryLocation, Offset2d, Offset3d, Pipeline, PolygonMode, Sampler,
    SamplerAddressMode, SamplerCompareOp, SamplerDesc, SamplerFilter, ShaderStageFlags, StencilOp,
    StencilOpState, StoreOp, SwapchainOutOfDateError, ThreadToken, Topology, TypedBind,
};

const NUM_FRAMES: usize = 2;

/// How many frames to delay swapchain destruction.
///
/// There's no correct answer here (spec bug) we're just picking a big number and hoping for the best.
const SWAPCHAIN_DESTROY_DELAY_FRAMES: usize = 8;

mod libc {
    use std::os::raw::{c_char, c_int, c_void};

    pub const RTLD_NOW: c_int = 0x2;
    pub const RTLD_LOCAL: c_int = 0;

    extern "C" {
        pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }
}

macro_rules! vk_check {
    ($e:expr) => ({
        #[allow(unused_unsafe)]
        let e = unsafe { $e };
        if e != vulkan_sys::Result::Success {
            panic!("assertion failed: `result == vk::Result::Success`: \n value: `{:?}`", e);
        }
    });
    ($e:expr, $($msg_args:tt)+) => ({
        #[allow(unused_unsafe)]
        let e = unsafe { $e };
        if e != vulkan_sys::::Result::Success {
            panic!("assertion failed: `result == vk::Result::Success`: \n value: `{:?}: {}`", e, format_args!($($msg_args)+));
        }
    })
}

#[must_use]
fn vk_vec<T, F: FnMut(&mut u32, *mut T) -> vulkan_sys::Result>(mut f: F) -> Vec<T> {
    let mut count = 0;
    vk_check!(f(&mut count, std::ptr::null_mut()));
    let mut v = Vec::with_capacity(count as usize);
    vk_check!(f(&mut count, v.as_mut_ptr()));
    unsafe { v.set_len(count as usize) };
    v
}

impl From<Extent2d> for vk::Extent2d {
    fn from(extent: Extent2d) -> Self {
        vk::Extent2d {
            width: extent.width,
            height: extent.height,
        }
    }
}

impl From<Extent3d> for vk::Extent3d {
    fn from(extent: Extent3d) -> Self {
        vk::Extent3d {
            width: extent.width,
            height: extent.height,
            depth: extent.depth,
        }
    }
}

impl From<Offset2d> for vk::Offset2d {
    fn from(extent: Offset2d) -> Self {
        vk::Offset2d {
            x: extent.x,
            y: extent.y,
        }
    }
}

impl From<Offset3d> for vk::Offset3d {
    fn from(extent: Offset3d) -> Self {
        vk::Offset3d {
            x: extent.x,
            y: extent.y,
            z: extent.z,
        }
    }
}

#[must_use]
fn vulkan_bool32(b: bool) -> vk::Bool32 {
    const VALUES: [vk::Bool32; 2] = [vk::Bool32::False, vk::Bool32::True];
    VALUES[b as usize]
}

#[must_use]
fn vulkan_format(format: ImageFormat) -> vk::Format {
    match format {
        ImageFormat::RGBA8_SRGB => vk::Format::R8G8B8A8_SRGB,
        ImageFormat::RGBA8_UNORM => vk::Format::R8G8B8A8_UNORM,
        ImageFormat::BGRA8_SRGB => vk::Format::B8G8R8A8_SRGB,
        ImageFormat::BGRA8_UNORM => vk::Format::B8G8R8A8_UNORM,
        ImageFormat::DEPTH_F32 => vk::Format::D32_SFLOAT,
    }
}

fn vulkan_aspect_for_format(format: ImageFormat) -> vk::ImageAspectFlags {
    match format {
        ImageFormat::BGRA8_SRGB
        | ImageFormat::BGRA8_UNORM
        | ImageFormat::RGBA8_SRGB
        | ImageFormat::RGBA8_UNORM => vk::ImageAspectFlags::COLOR,
        ImageFormat::DEPTH_F32 => vk::ImageAspectFlags::DEPTH,
    }
}

fn vulkan_aspect(aspect: ImageAspectFlags) -> vk::ImageAspectFlags {
    let mut aspect_flags = default();
    if aspect.contains(ImageAspectFlags::COLOR) {
        aspect_flags |= vk::ImageAspectFlags::COLOR;
    }
    if aspect.contains(ImageAspectFlags::DEPTH) {
        aspect_flags |= vk::ImageAspectFlags::DEPTH;
    }
    if aspect.contains(ImageAspectFlags::STENCIL) {
        aspect_flags |= vk::ImageAspectFlags::STENCIL;
    }
    aspect_flags
}

#[must_use]
fn vulkan_clear_value(clear_value: ClearValue) -> vk::ClearValue {
    match clear_value {
        ClearValue::ColorF32(value) => vk::ClearValue {
            color: vk::ClearColorValue { f32: value },
        },
        ClearValue::ColorU32(value) => vk::ClearValue {
            color: vk::ClearColorValue { u32: value },
        },
        ClearValue::ColorI32(value) => vk::ClearValue {
            color: vk::ClearColorValue { i32: value },
        },
        ClearValue::DepthStencil { depth, stencil } => vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { depth, stencil },
        },
    }
}

#[must_use]
fn vulkan_load_op(load_op: LoadOp) -> (vk::AttachmentLoadOp, vk::ClearValue) {
    match load_op {
        LoadOp::Load => (vk::AttachmentLoadOp::Load, vk::ClearValue::default()),
        LoadOp::Clear(clear_value) => {
            (vk::AttachmentLoadOp::Clear, vulkan_clear_value(clear_value))
        }
        LoadOp::DontCare => (vk::AttachmentLoadOp::DontCare, vk::ClearValue::default()),
    }
}

#[must_use]
fn vulkan_store_op(store_op: StoreOp) -> vk::AttachmentStoreOp {
    match store_op {
        StoreOp::Store => vk::AttachmentStoreOp::Store,
        StoreOp::DontCare => vk::AttachmentStoreOp::DontCare,
    }
}

#[must_use]
fn vulkan_shader_stage_flags(stage_flags: ShaderStageFlags) -> vk::ShaderStageFlags {
    let mut flags = vk::ShaderStageFlags::default();
    if stage_flags.contains(ShaderStageFlags::COMPUTE) {
        flags |= vk::ShaderStageFlags::COMPUTE;
    }
    if stage_flags.contains(ShaderStageFlags::FRAGMENT) {
        flags |= vk::ShaderStageFlags::FRAGMENT;
    }
    if stage_flags.contains(ShaderStageFlags::VERTEX) {
        flags |= vk::ShaderStageFlags::VERTEX;
    }
    flags
}

#[must_use]
fn vulkan_descriptor_type(binding_type: BindingType) -> vk::DescriptorType {
    match binding_type {
        BindingType::Sampler => vk::DescriptorType::Sampler,
        BindingType::Image => vk::DescriptorType::SampledImage,
        BindingType::UniformBuffer => vk::DescriptorType::UniformBuffer,
        BindingType::StorageBuffer => vk::DescriptorType::StorageBuffer,
        BindingType::DynamicUniformBuffer => vk::DescriptorType::UniformBufferDynamic,
        BindingType::DynamicStorageBuffer => vk::DescriptorType::StorageBufferDynamic,
    }
}

#[must_use]
fn vulkan_index_type(index_type: IndexType) -> vk::IndexType {
    match index_type {
        IndexType::U16 => vk::IndexType::Uint16,
        IndexType::U32 => vk::IndexType::Uint32,
    }
}

#[must_use]
fn vulkan_primitive_topology(primitive_topology: Topology) -> vk::PrimitiveTopology {
    match primitive_topology {
        Topology::Points => vk::PrimitiveTopology::PointList,
        Topology::Lines => vk::PrimitiveTopology::LineList,
        Topology::LineStrip => vk::PrimitiveTopology::LineStrip,
        Topology::Triangles => vk::PrimitiveTopology::TriangleList,
        Topology::TriangleStrip => vk::PrimitiveTopology::TriangleStrip,
    }
}

#[must_use]
fn vulkan_polygon_mode(polygon_mode: PolygonMode) -> vk::PolygonMode {
    match polygon_mode {
        PolygonMode::Fill => vk::PolygonMode::Fill,
        PolygonMode::Line => vk::PolygonMode::Line,
        PolygonMode::Point => vk::PolygonMode::Point,
    }
}

#[must_use]
fn vulkan_cull_mode(culling_mode: CullingMode) -> vk::CullModeFlags {
    match culling_mode {
        CullingMode::None => vk::CullModeFlags::NONE,
        CullingMode::Front => vk::CullModeFlags::FRONT,
        CullingMode::Back => vk::CullModeFlags::BACK,
    }
}

#[must_use]
fn vulkan_front_face(front_face: FrontFace) -> vk::FrontFace {
    match front_face {
        FrontFace::Clockwise => vk::FrontFace::Clockwise,
        FrontFace::CounterClockwise => vk::FrontFace::CounterClockwise,
    }
}

#[must_use]
fn vulkan_compare_op(compare_op: CompareOp) -> vk::CompareOp {
    match compare_op {
        CompareOp::Never => vk::CompareOp::Never,
        CompareOp::Less => vk::CompareOp::Less,
        CompareOp::Equal => vk::CompareOp::Equal,
        CompareOp::LessOrEqual => vk::CompareOp::LessOrEqual,
        CompareOp::Greater => vk::CompareOp::Greater,
        CompareOp::NotEqual => vk::CompareOp::NotEqual,
        CompareOp::GreaterOrEqual => vk::CompareOp::GreaterOrEqual,
        CompareOp::Always => vk::CompareOp::Always,
    }
}

#[must_use]
fn vulkan_stencil_op(stencil_op: StencilOp) -> vk::StencilOp {
    match stencil_op {
        StencilOp::Keep => vk::StencilOp::Keep,
        StencilOp::Zero => vk::StencilOp::Zero,
        StencilOp::Replace => vk::StencilOp::Replace,
        StencilOp::IncrementAndClamp => vk::StencilOp::IncrementAndClamp,
        StencilOp::DecrementAndClamp => vk::StencilOp::DecrementAndClamp,
        StencilOp::Invert => vk::StencilOp::Invert,
        StencilOp::IncrementAndWrap => vk::StencilOp::IncrementAndWrap,
        StencilOp::DecrementAndWrap => vk::StencilOp::DecrementAndWrap,
    }
}

#[must_use]
fn vulkan_stencil_op_state(stencil_op_state: StencilOpState) -> vk::StencilOpState {
    vk::StencilOpState {
        fail_op: vulkan_stencil_op(stencil_op_state.fail_op),
        pass_op: vulkan_stencil_op(stencil_op_state.pass_op),
        depth_fail_op: vulkan_stencil_op(stencil_op_state.depth_fail_op),
        compare_op: vulkan_compare_op(stencil_op_state.compare_op),
        compare_mask: stencil_op_state.compare_mask,
        write_mask: stencil_op_state.write_mask,
        reference: stencil_op_state.reference,
    }
}

#[must_use]
fn vulkan_blend_mode(blend_mode: BlendMode) -> vk::PipelineColorBlendAttachmentState {
    match blend_mode {
        BlendMode::Opaque => vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
            ..default()
        },
        BlendMode::Mask => todo!(),
        BlendMode::Translucent => vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::Bool32::True,
            src_color_blend_factor: vk::BlendFactor::SrcAlpha,
            dst_color_blend_factor: vk::BlendFactor::OneMinusSrcAlpha,
            color_blend_op: vk::BlendOp::Add,
            src_alpha_blend_factor: vk::BlendFactor::One,
            dst_alpha_blend_factor: vk::BlendFactor::Zero,
            alpha_blend_op: vk::BlendOp::Add,
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        },
        BlendMode::Premultiplied => vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::Bool32::True,
            src_color_blend_factor: vk::BlendFactor::One,
            dst_color_blend_factor: vk::BlendFactor::OneMinusSrcAlpha,
            color_blend_op: vk::BlendOp::Add,
            src_alpha_blend_factor: vk::BlendFactor::One,
            dst_alpha_blend_factor: vk::BlendFactor::Zero,
            alpha_blend_op: vk::BlendOp::Add,
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        },
        BlendMode::Additive => todo!(),
        BlendMode::Modulate => todo!(),
    }
}

#[must_use]
fn vulkan_image_view_type(layer_count: u32, image_dimension: ImageDimension) -> vk::ImageViewType {
    match (layer_count, image_dimension) {
        (1, ImageDimension::Type1d) => vk::ImageViewType::Type1d,
        (1, ImageDimension::Type2d) => vk::ImageViewType::Type2d,
        (1, ImageDimension::Type3d) => vk::ImageViewType::Type3d,
        (6, ImageDimension::TypeCube) => vk::ImageViewType::TypeCube,
        (_, ImageDimension::Type1d) => vk::ImageViewType::Type1dArray,
        (_, ImageDimension::Type2d) => vk::ImageViewType::Type2dArray,
        (_, ImageDimension::TypeCube) => vk::ImageViewType::TypeCubeArray,
        _ => panic!("unsupported view type"),
    }
}

fn vulkan_subresource_layers(
    subresource_layers: &ImageSubresourceLayers,
) -> vk::ImageSubresourceLayers {
    vk::ImageSubresourceLayers {
        aspect_mask: vulkan_aspect(subresource_layers.aspect),
        mip_level: subresource_layers.mip_level,
        base_array_layer: subresource_layers.base_array_layer,
        layer_count: subresource_layers.array_layer_count,
    }
}

fn vulkan_subresource_range(subresource: &ImageSubresourceRange) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange {
        aspect_mask: vulkan_aspect(subresource.aspect),
        base_mip_level: subresource.base_mip_level,
        level_count: subresource.mip_level_count,
        base_array_layer: subresource.base_array_layer,
        layer_count: subresource.array_layer_count,
    }
}

struct VulkanAccessInfo {
    stages: vk::PipelineStageFlags2,
    access: vk::AccessFlags2,
    layout: vk::ImageLayout,
}

#[must_use]
fn vulkan_access_info(access: Access) -> VulkanAccessInfo {
    match access {
        Access::None => VulkanAccessInfo {
            stages: default(),
            access: default(),
            layout: vk::ImageLayout::Undefined,
        },

        Access::IndirectBuffer => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::DRAW_INDIRECT,
            access: vk::AccessFlags2::INDIRECT_COMMAND_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::IndexBuffer => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_INPUT,
            access: vk::AccessFlags2::INDEX_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::VertexBuffer => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_INPUT,
            access: vk::AccessFlags2::VERTEX_ATTRIBUTE_READ,
            layout: vk::ImageLayout::Undefined,
        },

        Access::VertexShaderUniformBufferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_SHADER,
            access: vk::AccessFlags2::UNIFORM_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::VertexShaderSampledImageRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_SHADER,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::ReadOnlyOptimal,
        },
        Access::VertexShaderOtherRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_SHADER,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::General,
        },

        Access::FragmentShaderUniformBufferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            access: vk::AccessFlags2::UNIFORM_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::FragmentShaderSampledImageRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::ReadOnlyOptimal,
        },
        Access::FragmentShaderOtherRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::General,
        },

        Access::ColorAttachmentRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            access: vk::AccessFlags2::COLOR_ATTACHMENT_READ,
            layout: vk::ImageLayout::AttachmentOptimal,
        },
        Access::DepthStencilAttachmentRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
            layout: vk::ImageLayout::AttachmentOptimal,
        },

        Access::ShaderUniformBufferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::UNIFORM_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::ShaderUniformBufferOrVertexBufferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::UNIFORM_READ | vk::AccessFlags2::VERTEX_ATTRIBUTE_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::ShaderSampledImageRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::ReadOnlyOptimal,
        },
        Access::ShaderOtherRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::General,
        },

        Access::TransferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::TRANSFER,
            access: vk::AccessFlags2::TRANSFER_READ,
            layout: vk::ImageLayout::TransferSrcOptimal,
        },
        Access::HostRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::HOST,
            access: vk::AccessFlags2::HOST_READ,
            layout: vk::ImageLayout::General,
        },

        Access::PresentRead => VulkanAccessInfo {
            stages: default(),
            access: default(),
            layout: vk::ImageLayout::PresentSrcKhr,
        },

        Access::VertexShaderWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_SHADER,
            access: vk::AccessFlags2::SHADER_WRITE,
            layout: vk::ImageLayout::General,
        },
        Access::FragmentShaderWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            access: vk::AccessFlags2::SHADER_WRITE,
            layout: vk::ImageLayout::General,
        },
        Access::ColorAttachmentWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            layout: vk::ImageLayout::ColorAttachmentOptimal,
        },
        Access::DepthStencilAttachmentWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
            access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
            layout: vk::ImageLayout::DepthAttachmentOptimal,
        },
        Access::ShaderWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::SHADER_WRITE,
            layout: vk::ImageLayout::General,
        },
        Access::TransferWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::TRANSFER,
            access: vk::AccessFlags2::TRANSFER_WRITE,
            layout: vk::ImageLayout::TransferDstOptimal,
        },
        Access::HostPreInitializedWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::HOST,
            access: vk::AccessFlags2::HOST_WRITE,
            layout: vk::ImageLayout::Preinitialized,
        },
        Access::HostWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::HOST,
            access: vk::AccessFlags2::HOST_WRITE,
            layout: vk::ImageLayout::General,
        },
        Access::ColorAttachmentReadWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            access: vk::AccessFlags2::COLOR_ATTACHMENT_READ
                | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            layout: vk::ImageLayout::AttachmentOptimal,
        },
        Access::General => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::COLOR_ATTACHMENT_READ
                | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            layout: vk::ImageLayout::General,
        },
    }
}

fn vulkan_memory_barrier(barrier: &GlobalBarrier) -> vk::MemoryBarrier2 {
    let mut src_stage_mask = default();
    let mut src_access_mask = default();
    let mut dst_stage_mask = default();
    let mut dst_access_mask = default();

    for &access in barrier.prev_access {
        debug_assert!(
            access.is_read() || barrier.prev_access.len() == 1,
            "write access types must be on their own"
        );

        let info = vulkan_access_info(access);
        src_stage_mask |= info.stages;

        // For writes, add availability operations.
        if access.is_write() {
            src_access_mask |= info.access;
        }
    }

    for &access in barrier.next_access {
        debug_assert!(
            access.is_read() || barrier.prev_access.len() == 1,
            "write access types must be on their own"
        );

        let info = vulkan_access_info(access);
        dst_stage_mask |= info.stages;

        // Add visibility operations if necessary.
        //
        // If the src access mask is zero, this is a Write-After-Read hazard (or for some reason, a
        // Read-After-Read), so the dst access mask can be safely zeroed as these don't need
        // visibility.
        if src_access_mask != default() {
            dst_access_mask |= info.access;
        }
    }

    if src_stage_mask == default() {
        src_stage_mask = vk::PipelineStageFlags2::TOP_OF_PIPE;
    }

    if dst_stage_mask == default() {
        dst_stage_mask = vk::PipelineStageFlags2::BOTTOM_OF_PIPE;
    }

    vk::MemoryBarrier2 {
        src_stage_mask,
        src_access_mask,
        dst_stage_mask,
        dst_access_mask,
        ..default()
    }
}

fn vulkan_image_memory_barrier(
    barrier: &ImageBarrier,
    image: vk::Image,
    subresource_range: vk::ImageSubresourceRange,
) -> vk::ImageMemoryBarrier2 {
    let mut src_stage_mask = default();
    let mut src_access_mask = default();
    let mut dst_stage_mask = default();
    let mut dst_access_mask = default();
    let mut old_layout = vk::ImageLayout::Undefined;
    let mut new_layout = vk::ImageLayout::Undefined;

    for &access in barrier.prev_access {
        debug_assert!(
            access.is_read() || barrier.prev_access.len() == 1,
            "write access types must be on their own"
        );

        let info = vulkan_access_info(access);
        src_stage_mask |= info.stages;

        // For writes, add availability operations.
        if access.is_write() {
            src_access_mask |= info.access;
        }

        let layout = match barrier.prev_layout {
            ImageLayout::Optimal => info.layout,
            ImageLayout::General => {
                if access == Access::PresentRead {
                    vk::ImageLayout::PresentSrcKhr
                } else {
                    vk::ImageLayout::General
                }
            }
        };

        debug_assert!(
            old_layout == vk::ImageLayout::Undefined || old_layout == layout,
            "mixed image layout"
        );

        old_layout = layout;
    }

    for &access in barrier.next_access {
        debug_assert!(
            access.is_read() || barrier.prev_access.len() == 1,
            "write access types must be on their own"
        );

        let info = vulkan_access_info(access);
        dst_stage_mask |= info.stages;

        // Add visibility operations if necessary.
        //
        // If the src access mask is zero, this is a Write-After-Read hazard (or for some reason, a
        // Read-After-Read), so the dst access mask can be safely zeroed as these don't need
        // visibility.
        if src_access_mask != default() {
            dst_access_mask |= info.access;
        }

        let layout = match barrier.next_layout {
            ImageLayout::Optimal => info.layout,
            ImageLayout::General => {
                if access == Access::PresentRead {
                    vk::ImageLayout::PresentSrcKhr
                } else {
                    vk::ImageLayout::General
                }
            }
        };

        debug_assert!(
            new_layout == vk::ImageLayout::Undefined || new_layout == layout,
            "mixed image layout"
        );

        new_layout = layout;
    }

    vk::ImageMemoryBarrier2 {
        src_stage_mask,
        src_access_mask,
        dst_stage_mask,
        dst_access_mask,
        old_layout,
        new_layout,
        src_queue_family_index: 0,
        dst_queue_family_index: 0,
        image,
        subresource_range,
        ..default()
    }
}

struct VulkanBuffer {
    memory: VulkanMemory,
    buffer: vk::Buffer,
}

#[derive(Clone)]
struct VulkanImage {
    memory: VulkanMemory,
    image: vk::Image,
}

struct VulkanImageUnique {
    image: VulkanImage,
    view: vk::ImageView,
}

struct VulkanImageShared {
    image: ManualArc<VulkanImage>,
    view: vk::ImageView,
}

struct VulkanImageSwapchain {
    surface: vk::SurfaceKHR,
    image: vk::Image,
    view: vk::ImageView,
}

enum VulkanImageHolder {
    Unique(VulkanImageUnique),
    Shared(VulkanImageShared),
    Swapchain(VulkanImageSwapchain),
}

impl VulkanImageHolder {
    fn image(&self) -> vk::Image {
        match self {
            VulkanImageHolder::Unique(x) => x.image.image,
            VulkanImageHolder::Shared(_) => panic!(),
            VulkanImageHolder::Swapchain(x) => x.image,
        }
    }

    fn image_view(&self) -> vk::ImageView {
        match self {
            VulkanImageHolder::Unique(x) => x.view,
            VulkanImageHolder::Shared(x) => x.view,
            VulkanImageHolder::Swapchain(x) => x.view,
        }
    }
}

struct VulkanSampler(vk::Sampler);

struct VulkanBindGroupLayout(vk::DescriptorSetLayout);

struct VulkanPipeline {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    pipeline_bind_point: vk::PipelineBindPoint,
}

enum VulkanSwapchainState {
    Vacant,
    Occupied {
        width: u32,
        height: u32,
        suboptimal: bool,
        swapchain: vk::SwapchainKHR,
        image_views: Box<[Image]>,
    },
}

struct VulkanSwapchain {
    surface_format: vk::SurfaceFormatKHR,

    state: VulkanSwapchainState,

    _formats: Box<[vk::SurfaceFormatKHR]>,
    _present_modes: Box<[vk::PresentModeKHR]>,
    capabilities: vk::SurfaceCapabilitiesKHR,
}

#[derive(Default)]
struct VulkanPresentInfo {
    acquire: vk::Semaphore,
    release: vk::Semaphore,
    swapchain: vk::SwapchainKHR,
    image_index: u32,
}

struct VulkanMemoryDesc {
    requirements: vk::MemoryRequirements,
    memory_location: MemoryLocation,
    _linear: bool,
}

#[derive(Clone)]
struct VulkanMemory {
    memory: vk::DeviceMemory,
    offset: u64,
    size: u64,
}

#[derive(Clone)]
struct VulkanBoundPipeline {
    pipeline_layout: vk::PipelineLayout,
    pipeline_bind_point: vk::PipelineBindPoint,
}

struct VulkanCmdBuffer {
    command_buffer: vk::CommandBuffer,
    bound_pipeline: Option<VulkanBoundPipeline>,
    swapchains_touched: HashMap<vk::SurfaceKHR, (vk::Image, vk::PipelineStageFlags2)>,
}

struct VulkanCmdBufferPool {
    command_pool: vk::CommandPool,
    next_free_index: usize,
    command_buffers: Vec<vk::CommandBuffer>,
}

struct VulkanPerThread {
    cmd_buffer_pool: VulkanCmdBufferPool,
    descriptor_pool: vk::DescriptorPool,
    arena: Arena,
}

struct VulkanFrame {
    universal_queue_fence: AtomicU64,

    per_thread: GpuConcurrent<VulkanPerThread>,

    present_swapchains: Mutex<HashMap<vk::SurfaceKHR, VulkanPresentInfo>>,

    destroyed_allocations: Mutex<VecDeque<VulkanMemory>>,
    destroyed_buffers: Mutex<VecDeque<vk::Buffer>>,
    destroyed_buffer_views: Mutex<VecDeque<vk::BufferView>>,
    destroyed_images: Mutex<VecDeque<vk::Image>>,
    destroyed_image_views: Mutex<VecDeque<vk::ImageView>>,
    destroyed_samplers: Mutex<VecDeque<vk::Sampler>>,
    destroyed_descriptor_set_layouts: Mutex<VecDeque<vk::DescriptorSetLayout>>,
    destroyed_pipeline_layouts: Mutex<VecDeque<vk::PipelineLayout>>,
    destroyed_pipelines: Mutex<VecDeque<vk::Pipeline>>,

    recycled_semaphores: Mutex<VecDeque<vk::Semaphore>>,
    recycled_descriptor_pools: Mutex<VecDeque<vk::DescriptorPool>>,
}

impl VulkanFrame {
    fn recycle_semaphore(&self, semaphore: vk::Semaphore) {
        self.recycled_semaphores.lock().push_back(semaphore);
    }

    fn recycle_descriptor_pool(&self, descriptor_pool: vk::DescriptorPool) {
        self.recycled_descriptor_pools
            .lock()
            .push_back(descriptor_pool)
    }
}

type SwapchainDestroyQueue = DelayQueue<(vk::SwapchainKHR, vk::SurfaceKHR, Box<[vk::ImageView]>)>;

pub(crate) struct VulkanDevice {
    instance: vk::Instance,
    physical_device: vk::PhysicalDevice,
    physical_device_memory_properties: Box<vk::PhysicalDeviceMemoryProperties>,
    device: vk::Device,

    universal_queue: vk::Queue,
    universal_queue_fence: AtomicU64,
    universal_queue_semaphore: vk::Semaphore,
    universal_queue_family_index: u32,

    frame_counter: FrameCounter,
    frames: Box<[UnsafeCell<VulkanFrame>; NUM_FRAMES]>,

    surfaces: Mutex<HashMap<RawWindow, vk::SurfaceKHR>>,

    swapchains: Mutex<HashMap<vk::SurfaceKHR, VulkanSwapchain>>,
    destroyed_swapchains: Mutex<SwapchainDestroyQueue>,

    image_pool: Mutex<Pool<VulkanImageHolder>>,
    buffer_pool: Mutex<Pool<VulkanBuffer>>,
    sampler_pool: Mutex<Pool<VulkanSampler>>,
    bind_group_layout_pool: Mutex<Pool<VulkanBindGroupLayout>>,
    pipeline_pool: Mutex<Pool<VulkanPipeline>>,

    recycled_semaphores: Mutex<VecDeque<vk::Semaphore>>,
    recycled_descriptor_pools: Mutex<VecDeque<vk::DescriptorPool>>,

    _global_fn: vk::GlobalFunctions,
    instance_fn: vk::InstanceFunctions,
    xcb_surface_fn: Option<vk::XcbSurfaceKHRFunctions>,
    xlib_surface_fn: Option<vk::XlibSurfaceKHRFunctions>,
    wayland_surface_fn: Option<vk::WaylandSurfaceKHRFunctions>,
    surface_fn: vk::SurfaceKHRFunctions,
    swapchain_fn: vk::SwapchainKHRFunctions,
    device_fn: vk::DeviceFunctions,
}

impl VulkanDevice {
    pub(crate) fn new() -> Self {
        let get_proc_addr = unsafe {
            let module = libc::dlopen(
                cstr!("libvulkan.so.1").as_ptr(),
                libc::RTLD_NOW | libc::RTLD_LOCAL,
            );
            libc::dlsym(module, cstr!("vkGetInstanceProcAddr").as_ptr())
        };

        let global_fn = unsafe { vk::GlobalFunctions::new(get_proc_addr) };

        let api_version = {
            let mut api_version = 0;
            vk_check!(global_fn.enumerate_instance_version(&mut api_version));
            api_version
        };

        if api_version < vk::VERSION_1_2 {
            panic!("instance does not support vulkan 1.2")
        }

        #[cfg(debug_assertions)]
        let enabled_layers = &[cstr!("VK_LAYER_KHRONOS_validation").as_ptr()];
        #[cfg(not(debug_assertions))]
        let enabled_layers = &[];

        let extension_properties = vk_vec(|count, ptr| unsafe {
            global_fn.enumerate_instance_extension_properties(std::ptr::null(), count, ptr)
        });

        let mut has_wayland_support = false;
        let mut has_xlib_support = false;
        let mut has_xcb_support = false;

        let mut enabled_extensions = vec![];
        for extension in &extension_properties {
            let extension_name = cstr_from_bytes_until_nul(&extension.extension_name).unwrap();

            match extension_name.to_str().unwrap() {
                "VK_KHR_wayland_surface" => {
                    has_wayland_support = true;
                    enabled_extensions.push(extension_name);
                }
                "VK_KHR_xlib_surface" => {
                    has_xlib_support = true;
                    enabled_extensions.push(extension_name);
                }
                "VK_KHR_xcb_surface" => {
                    has_xcb_support = true;
                    enabled_extensions.push(extension_name);
                }
                _ => {}
            }
        }

        // If we found any surface extensions, we need to additionally enable VK_KHR_surface.
        if !enabled_extensions.is_empty() {
            enabled_extensions.push(cstr!("VK_KHR_surface"));
        }

        let enabled_extensions = enabled_extensions
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<*const c_char>>();

        let instance = {
            let application_info = vk::ApplicationInfo {
                application_name: cstr!("TRIANGLE").as_ptr(),
                application_version: 0,
                engine_name: cstr!("NARCISSUS").as_ptr(),
                engine_version: 0,
                api_version: vk::VERSION_1_3,
                ..default()
            };
            let create_info = vk::InstanceCreateInfo {
                enabled_layers: enabled_layers.into(),
                enabled_extension_names: enabled_extensions.as_slice().into(),
                application_info: Some(&application_info),
                ..default()
            };
            let mut instance = vk::Instance::null();
            vk_check!(global_fn.create_instance(&create_info, None, &mut instance));
            instance
        };

        let instance_fn = vk::InstanceFunctions::new(&global_fn, instance, vk::VERSION_1_2);

        let xcb_surface_fn = if has_xcb_support {
            Some(vk::XcbSurfaceKHRFunctions::new(&global_fn, instance))
        } else {
            None
        };

        let xlib_surface_fn = if has_xlib_support {
            Some(vk::XlibSurfaceKHRFunctions::new(&global_fn, instance))
        } else {
            None
        };

        let wayland_surface_fn = if has_wayland_support {
            Some(vk::WaylandSurfaceKHRFunctions::new(&global_fn, instance))
        } else {
            None
        };

        let surface_fn = vk::SurfaceKHRFunctions::new(&global_fn, instance);
        let swapchain_fn = vk::SwapchainKHRFunctions::new(&global_fn, instance, vk::VERSION_1_1);

        let physical_devices = vk_vec(|count, ptr| unsafe {
            instance_fn.enumerate_physical_devices(instance, count, ptr)
        });

        let physical_device = physical_devices
            .iter()
            .copied()
            .find(|&physical_device| {
                let (
                    physical_device_properties,
                    _physical_device_properties_11,
                    _physical_device_properties_12,
                    _physical_device_properties_13,
                ) = {
                    let mut properties_13 = vk::PhysicalDeviceVulkan13Properties::default();
                    let mut properties_12 = vk::PhysicalDeviceVulkan12Properties {
                        _next: &mut properties_13 as *mut vk::PhysicalDeviceVulkan13Properties
                            as *mut _,
                        ..default()
                    };
                    let mut properties_11 = vk::PhysicalDeviceVulkan11Properties {
                        _next: &mut properties_12 as *mut vk::PhysicalDeviceVulkan12Properties
                            as *mut _,
                        ..default()
                    };
                    let mut properties = vk::PhysicalDeviceProperties2 {
                        _next: &mut properties_11 as *mut vk::PhysicalDeviceVulkan11Properties
                            as *mut _,
                        ..default()
                    };
                    unsafe {
                        instance_fn
                            .get_physical_device_properties2(physical_device, &mut properties);
                    }
                    (properties, properties_11, properties_12, properties_13)
                };

                let (
                    _physical_device_features,
                    _physical_device_features_11,
                    physical_device_features_12,
                    physical_device_features_13,
                ) = {
                    let mut features_13 = vk::PhysicalDeviceVulkan13Features::default();
                    let mut features_12 = vk::PhysicalDeviceVulkan12Features {
                        _next: &mut features_13 as *mut vk::PhysicalDeviceVulkan13Features
                            as *mut _,
                        ..default()
                    };
                    let mut features_11 = vk::PhysicalDeviceVulkan11Features {
                        _next: &mut features_12 as *mut vk::PhysicalDeviceVulkan12Features
                            as *mut _,
                        ..default()
                    };
                    let mut features = vk::PhysicalDeviceFeatures2 {
                        _next: &mut features_11 as *mut vk::PhysicalDeviceVulkan11Features
                            as *mut _,
                        ..default()
                    };

                    unsafe {
                        instance_fn.get_physical_device_features2(physical_device, &mut features);
                    }
                    (features.features, features_11, features_12, features_13)
                };

                physical_device_properties.properties.api_version >= vk::VERSION_1_3
                    && physical_device_features_13.dynamic_rendering == vk::Bool32::True
                    && physical_device_features_12.timeline_semaphore == vk::Bool32::True
                    && physical_device_features_12.descriptor_indexing == vk::Bool32::True
                    && physical_device_features_12.descriptor_binding_partially_bound
                        == vk::Bool32::True
                    && physical_device_features_12.draw_indirect_count == vk::Bool32::True
            })
            .expect("no supported physical devices reported");

        let physical_device_memory_properties = unsafe {
            let mut memory_properties = vk::PhysicalDeviceMemoryProperties::default();
            instance_fn
                .get_physical_device_memory_properties(physical_device, &mut memory_properties);
            memory_properties
        };

        let queue_family_properties = vk_vec(|count, ptr| unsafe {
            instance_fn.get_physical_device_queue_family_properties(physical_device, count, ptr);
            vk::Result::Success
        });

        let (queue_family_index, _) = (0..)
            .zip(queue_family_properties.iter())
            .find(|&(_, queue_family_properties)| {
                queue_family_properties
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)
            })
            .expect("failed to find universal queue for chosen device");

        let device = {
            let queue_priorities: &[_] = &[1.0];
            let device_queue_create_infos: &[_] = &[vk::DeviceQueueCreateInfo {
                queue_family_index,
                queue_priorities: queue_priorities.into(),
                ..default()
            }];
            let enabled_extensions = vec![cstr!("VK_KHR_swapchain")];
            let enabled_extensions = enabled_extensions
                .iter()
                .map(|x| x.as_ptr())
                .collect::<Vec<*const c_char>>();
            let enabled_features_13 = vk::PhysicalDeviceVulkan13Features {
                dynamic_rendering: vk::Bool32::True,
                synchronization2: vk::Bool32::True,
                ..default()
            };
            let enabled_features_12 = vk::PhysicalDeviceVulkan12Features {
                _next: &enabled_features_13 as *const vk::PhysicalDeviceVulkan13Features as *mut _,
                timeline_semaphore: vk::Bool32::True,
                descriptor_indexing: vk::Bool32::True,
                descriptor_binding_partially_bound: vk::Bool32::True,
                draw_indirect_count: vk::Bool32::True,
                ..default()
            };
            let enabled_features_11 = vk::PhysicalDeviceVulkan11Features {
                _next: &enabled_features_12 as *const vk::PhysicalDeviceVulkan12Features as *mut _,
                ..default()
            };
            let enabled_features = vk::PhysicalDeviceFeatures2 {
                _next: &enabled_features_11 as *const vk::PhysicalDeviceVulkan11Features as *mut _,
                ..default()
            };
            let create_info = vk::DeviceCreateInfo {
                _next: &enabled_features as *const vk::PhysicalDeviceFeatures2 as *const _,
                enabled_extension_names: enabled_extensions.as_slice().into(),
                queue_create_infos: device_queue_create_infos.into(),
                ..default()
            };
            let mut device = vk::Device::null();
            vk_check!(instance_fn.create_device(physical_device, &create_info, None, &mut device));
            device
        };

        let device_fn = vk::DeviceFunctions::new(&instance_fn, device, vk::VERSION_1_3);

        let universal_queue = unsafe {
            let mut queue = vk::Queue::default();
            device_fn.get_device_queue(device, queue_family_index, 0, &mut queue);
            queue
        };

        let universal_queue_fence = 0;

        let universal_queue_semaphore = {
            let type_create_info = vk::SemaphoreTypeCreateInfo {
                semaphore_type: vk::SemaphoreType::Timeline,
                initial_value: universal_queue_fence,
                ..default()
            };
            let create_info = vk::SemaphoreCreateInfo {
                _next: &type_create_info as *const vk::SemaphoreTypeCreateInfo as *const _,
                ..default()
            };
            let mut semaphore = vk::Semaphore::null();
            vk_check!(device_fn.create_semaphore(device, &create_info, None, &mut semaphore));
            semaphore
        };

        let frames = Box::new(std::array::from_fn(|_| {
            let per_thread = GpuConcurrent::new(|| {
                let command_pool = {
                    let create_info = vk::CommandPoolCreateInfo {
                        flags: vk::CommandPoolCreateFlags::TRANSIENT,
                        queue_family_index,
                        ..default()
                    };
                    let mut pool = vk::CommandPool::null();
                    vk_check!(device_fn.create_command_pool(device, &create_info, None, &mut pool));
                    pool
                };
                let cmd_buffer_pool = VulkanCmdBufferPool {
                    command_pool,
                    command_buffers: Vec::new(),
                    next_free_index: 0,
                };
                VulkanPerThread {
                    cmd_buffer_pool,
                    descriptor_pool: vk::DescriptorPool::null(),
                    arena: Arena::new(),
                }
            });

            UnsafeCell::new(VulkanFrame {
                per_thread,
                universal_queue_fence: AtomicU64::new(universal_queue_fence),
                present_swapchains: default(),
                destroyed_allocations: default(),
                destroyed_buffers: default(),
                destroyed_buffer_views: default(),
                destroyed_images: default(),
                destroyed_image_views: default(),
                destroyed_samplers: default(),
                destroyed_descriptor_set_layouts: default(),
                destroyed_pipeline_layouts: default(),
                destroyed_pipelines: default(),
                recycled_semaphores: default(),
                recycled_descriptor_pools: default(),
            })
        }));

        Self {
            instance,
            physical_device,
            physical_device_memory_properties: Box::new(physical_device_memory_properties),
            device,

            universal_queue,
            universal_queue_fence: AtomicU64::new(universal_queue_fence),
            universal_queue_semaphore,
            universal_queue_family_index: queue_family_index,

            frame_counter: FrameCounter::new(),
            frames,

            surfaces: default(),
            swapchains: default(),
            destroyed_swapchains: Mutex::new(DelayQueue::new(SWAPCHAIN_DESTROY_DELAY_FRAMES)),

            image_pool: default(),
            buffer_pool: default(),
            sampler_pool: default(),
            bind_group_layout_pool: default(),
            pipeline_pool: default(),

            recycled_semaphores: default(),
            recycled_descriptor_pools: default(),

            _global_fn: global_fn,
            instance_fn,
            xcb_surface_fn,
            xlib_surface_fn,
            wayland_surface_fn,
            surface_fn,
            swapchain_fn,
            device_fn,
        }
    }

    fn frame<'token>(&self, frame: &'token Frame) -> &'token VulkanFrame {
        frame.check_device(self as *const _ as usize);
        frame.check_frame_counter(self.frame_counter.load());
        // Safety: Reference is bound to the frame exposed by the API. only one frame can be valid
        // at a time. The returned VulkanFrame is only valid so long as we have a ref on the frame.
        unsafe { &*self.frames[frame.frame_index % NUM_FRAMES].get() }
    }

    fn frame_mut<'token>(&self, frame: &'token mut Frame) -> &'token mut VulkanFrame {
        frame.check_device(self as *const _ as usize);
        frame.check_frame_counter(self.frame_counter.load());
        // Safety: Reference is bound to the frame exposed by the API. only one frame can be valid
        // at a time. The returned VulkanFrame is only valid so long as we have a ref on the frame.
        unsafe { &mut *self.frames[frame.frame_index % NUM_FRAMES].get() }
    }

    fn cmd_buffer_mut<'a>(&self, cmd_buffer: &'a mut CmdBuffer) -> &'a mut VulkanCmdBuffer {
        // Safety: CmdBuffer's can't outlive a frame, and the memory for a cmd_buffer is reset when
        // the frame ends. So the pointer contained in the cmd_buffer is always valid while the
        // CmdBuffer is valid. They can't cloned, copied or be sent between threads, and we have a
        // mut reference.
        unsafe {
            NonNull::new_unchecked(cmd_buffer.cmd_buffer_addr as *mut VulkanCmdBuffer).as_mut()
        }
    }

    fn find_memory_type_index(&self, filter: u32, flags: vk::MemoryPropertyFlags) -> u32 {
        (0..self.physical_device_memory_properties.memory_type_count)
            .map(|memory_type_index| {
                (
                    memory_type_index,
                    self.physical_device_memory_properties.memory_types[memory_type_index as usize],
                )
            })
            .find(|(i, memory_type)| {
                (filter & (1 << i)) != 0 && memory_type.property_flags.contains(flags)
            })
            .expect("could not find memory type matching flags")
            .0
    }

    fn allocate_memory(&self, desc: &VulkanMemoryDesc) -> VulkanMemory {
        let memory_property_flags = match desc.memory_location {
            MemoryLocation::HostMapped => vk::MemoryPropertyFlags::HOST_VISIBLE,
            MemoryLocation::Device => vk::MemoryPropertyFlags::DEVICE_LOCAL,
        };

        let memory_type_index =
            self.find_memory_type_index(desc.requirements.memory_type_bits, memory_property_flags);
        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: desc.requirements.size,
            memory_type_index,
            ..default()
        };
        let mut memory = vk::DeviceMemory::null();
        vk_check!(self
            .device_fn
            .allocate_memory(self.device, &allocate_info, None, &mut memory));

        VulkanMemory {
            memory,
            offset: 0,
            size: desc.requirements.size,
        }
    }

    fn allocate_memory_for_buffer(
        &self,
        buffer: vk::Buffer,
        memory_location: MemoryLocation,
    ) -> VulkanMemory {
        let info = vk::BufferMemoryRequirementsInfo2 {
            buffer,
            ..default()
        };
        let mut memory_requirements = vk::MemoryRequirements2::default();
        self.device_fn.get_buffer_memory_requirements2(
            self.device,
            &info,
            &mut memory_requirements,
        );

        self.allocate_memory(&VulkanMemoryDesc {
            requirements: memory_requirements.memory_requirements,
            memory_location,
            _linear: true,
        })
    }

    fn allocate_memory_for_image(
        &self,
        image: vk::Image,
        memory_location: MemoryLocation,
    ) -> VulkanMemory {
        let info = vk::ImageMemoryRequirementsInfo2 { image, ..default() };
        let mut memory_requirements = vk::MemoryRequirements2::default();
        self.device_fn
            .get_image_memory_requirements2(self.device, &info, &mut memory_requirements);

        self.allocate_memory(&VulkanMemoryDesc {
            requirements: memory_requirements.memory_requirements,
            memory_location,
            _linear: true,
        })
    }

    fn request_descriptor_pool(&self) -> vk::DescriptorPool {
        if let Some(descriptor_pool) = self.recycled_descriptor_pools.lock().pop_front() {
            descriptor_pool
        } else {
            let pool_sizes: [vk::DescriptorPoolSize; 6] = [
                vk::DescriptorType::Sampler,
                vk::DescriptorType::UniformBuffer,
                vk::DescriptorType::UniformBufferDynamic,
                vk::DescriptorType::StorageBuffer,
                vk::DescriptorType::StorageBufferDynamic,
                vk::DescriptorType::SampledImage,
            ]
            .map(|descriptor_type| vk::DescriptorPoolSize {
                descriptor_type,
                descriptor_count: 500,
            });
            let mut descriptor_pool = vk::DescriptorPool::null();
            let create_info = vk::DescriptorPoolCreateInfo {
                max_sets: 500,
                pool_sizes: pool_sizes.as_ref().into(),
                ..default()
            };
            vk_check!(self.device_fn.create_descriptor_pool(
                self.device,
                &create_info,
                None,
                &mut descriptor_pool
            ));
            descriptor_pool
        }
    }

    fn request_semaphore(&self) -> vk::Semaphore {
        if let Some(semaphore) = self.recycled_semaphores.lock().pop_front() {
            semaphore
        } else {
            let mut semaphore = vk::Semaphore::null();
            let create_info = vk::SemaphoreCreateInfo::default();
            vk_check!(self.device_fn.create_semaphore(
                self.device,
                &create_info,
                None,
                &mut semaphore
            ));
            semaphore
        }
    }

    fn request_transient_semaphore(&self, frame: &VulkanFrame) -> vk::Semaphore {
        let semaphore = self.request_semaphore();
        frame.recycle_semaphore(semaphore);
        semaphore
    }

    fn destroy_deferred(
        device_fn: &vk::DeviceFunctions,
        device: vk::Device,
        frame: &mut VulkanFrame,
    ) {
        for pipeline_layout in frame.destroyed_pipeline_layouts.get_mut().drain(..) {
            unsafe { device_fn.destroy_pipeline_layout(device, pipeline_layout, None) }
        }
        for pipeline in frame.destroyed_pipelines.get_mut().drain(..) {
            unsafe { device_fn.destroy_pipeline(device, pipeline, None) }
        }
        for descriptor_set_layout in frame.destroyed_descriptor_set_layouts.get_mut().drain(..) {
            unsafe { device_fn.destroy_descriptor_set_layout(device, descriptor_set_layout, None) }
        }
        for sampler in frame.destroyed_samplers.get_mut().drain(..) {
            unsafe { device_fn.destroy_sampler(device, sampler, None) }
        }
        for image_view in frame.destroyed_image_views.get_mut().drain(..) {
            unsafe { device_fn.destroy_image_view(device, image_view, None) }
        }
        for image in frame.destroyed_images.get_mut().drain(..) {
            unsafe { device_fn.destroy_image(device, image, None) }
        }
        for buffer_view in frame.destroyed_buffer_views.get_mut().drain(..) {
            unsafe { device_fn.destroy_buffer_view(device, buffer_view, None) }
        }
        for buffer in frame.destroyed_buffers.get_mut().drain(..) {
            unsafe { device_fn.destroy_buffer(device, buffer, None) }
        }
        for memory in frame.destroyed_allocations.get_mut().drain(..) {
            unsafe { device_fn.free_memory(device, memory.memory, None) };
        }
    }

    fn destroy_swapchain_deferred(
        &self,
        surface: vk::SurfaceKHR,
        swapchain: vk::SwapchainKHR,
        image_views: &[vk::ImageView],
    ) {
        let device_fn = &self.device_fn;
        let swapchain_fn = &self.swapchain_fn;
        let surface_fn = &self.surface_fn;
        let instance = self.instance;
        let device = self.device;

        if !image_views.is_empty() {
            for &image_view in image_views {
                unsafe { device_fn.destroy_image_view(device, image_view, None) }
            }
        }
        if !swapchain.is_null() {
            unsafe { swapchain_fn.destroy_swapchain(device, swapchain, None) }
        }
        if !surface.is_null() {
            unsafe { surface_fn.destroy_surface(instance, surface, None) }
        }
    }
}

impl Device for VulkanDevice {
    fn create_buffer(&self, desc: &BufferDesc) -> Buffer {
        let mut usage = vk::BufferUsageFlags::default();
        if desc.usage.contains(BufferUsageFlags::UNIFORM) {
            usage |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if desc.usage.contains(BufferUsageFlags::STORAGE) {
            usage |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if desc.usage.contains(BufferUsageFlags::INDEX) {
            usage |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if desc.usage.contains(BufferUsageFlags::TRANSFER_SRC) {
            usage |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if desc.usage.contains(BufferUsageFlags::TRANSFER_DST) {
            usage |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        let queue_family_indices = &[self.universal_queue_family_index];

        let create_info = vk::BufferCreateInfo {
            size: desc.size as u64,
            usage,
            queue_family_indices: queue_family_indices.into(),
            sharing_mode: vk::SharingMode::Exclusive,
            ..default()
        };
        let mut buffer = vk::Buffer::null();
        vk_check!(self
            .device_fn
            .create_buffer(self.device, &create_info, None, &mut buffer));

        let memory = self.allocate_memory_for_buffer(buffer, desc.location);

        unsafe {
            self.device_fn.bind_buffer_memory2(
                self.device,
                &[vk::BindBufferMemoryInfo {
                    buffer,
                    memory: memory.memory,
                    offset: memory.offset,
                    ..default()
                }],
            )
        };

        let handle = self
            .buffer_pool
            .lock()
            .insert(VulkanBuffer { memory, buffer });

        Buffer(handle)
    }

    fn create_buffer_with_data(&self, desc: &BufferDesc, initial_data: &[u8]) -> Buffer {
        let len = initial_data.len();

        assert!(len <= desc.size, "initial data larger than buffer");
        assert!(desc.location == MemoryLocation::HostMapped);
        let buffer = self.create_buffer(desc);

        unsafe {
            let dst = std::slice::from_raw_parts_mut(self.map_buffer(buffer), len);
            dst.copy_from_slice(initial_data);
            self.unmap_buffer(buffer);
        }

        buffer
    }

    fn create_image(&self, desc: &ImageDesc) -> Image {
        debug_assert_ne!(desc.layer_count, 0, "layers must be at least one");
        debug_assert_ne!(desc.width, 0, "width must be at least one");
        debug_assert_ne!(desc.height, 0, "height must be at least one");
        debug_assert_ne!(desc.depth, 0, "depth must be at least one");

        if desc.dimension == ImageDimension::Type3d {
            debug_assert_eq!(desc.layer_count, 1, "3d image arrays are illegal");
        }

        if desc.dimension == ImageDimension::TypeCube {
            debug_assert!(
                desc.layer_count % 6 == 0,
                "cubemaps must have 6 layers each"
            );
            debug_assert_eq!(desc.depth, 1, "cubemap faces must be 2d");
        }

        let mut flags = vk::ImageCreateFlags::default();
        if desc.dimension == ImageDimension::TypeCube {
            flags |= vk::ImageCreateFlags::CUBE_COMPATIBLE
        }

        let image_type = match desc.dimension {
            ImageDimension::Type1d => vk::ImageType::Type1d,
            ImageDimension::Type2d => vk::ImageType::Type2d,
            ImageDimension::Type3d => vk::ImageType::Type3d,
            ImageDimension::TypeCube => vk::ImageType::Type2d,
        };
        let format = vulkan_format(desc.format);
        let extent = vk::Extent3d {
            width: desc.width,
            height: desc.height,
            depth: desc.depth,
        };

        let mut usage = default();
        if desc.usage.contains(ImageUsageFlags::SAMPLED) {
            usage |= vk::ImageUsageFlags::SAMPLED;
        }
        if desc.usage.contains(ImageUsageFlags::STORAGE) {
            usage |= vk::ImageUsageFlags::STORAGE;
        }
        if desc
            .usage
            .contains(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        {
            usage |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        }
        if desc.usage.contains(ImageUsageFlags::COLOR_ATTACHMENT) {
            usage |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
        }
        if desc.usage.contains(ImageUsageFlags::TRANSFER_DST) {
            usage |= vk::ImageUsageFlags::TRANSFER_DST;
        }
        if desc.usage.contains(ImageUsageFlags::TRANSFER_SRC) {
            usage |= vk::ImageUsageFlags::TRANSFER_SRC;
        }

        let queue_family_indices = &[self.universal_queue_family_index];
        let create_info = vk::ImageCreateInfo {
            flags,
            image_type,
            format,
            extent,
            mip_levels: desc.mip_levels,
            array_layers: desc.layer_count,
            samples: vk::SampleCountFlags::SAMPLE_COUNT_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage,
            sharing_mode: vk::SharingMode::Exclusive,
            queue_family_indices: queue_family_indices.into(),
            initial_layout: vk::ImageLayout::Undefined,
            ..default()
        };

        let mut image = vk::Image::null();
        vk_check!(self
            .device_fn
            .create_image(self.device, &create_info, None, &mut image));

        let memory = self.allocate_memory_for_image(image, desc.location);

        unsafe {
            self.device_fn.bind_image_memory2(
                self.device,
                &[vk::BindImageMemoryInfo {
                    image,
                    memory: memory.memory,
                    offset: memory.offset,
                    ..default()
                }],
            )
        };

        let view_type = vulkan_image_view_type(desc.layer_count, desc.dimension);
        let aspect_mask = vulkan_aspect_for_format(desc.format);
        let create_info = vk::ImageViewCreateInfo {
            image,
            view_type,
            format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: desc.mip_levels,
                base_array_layer: 0,
                layer_count: desc.layer_count,
            },
            ..default()
        };

        let mut view = vk::ImageView::null();
        vk_check!(self
            .device_fn
            .create_image_view(self.device, &create_info, None, &mut view));

        let image = VulkanImageUnique {
            image: VulkanImage { image, memory },
            view,
        };

        let handle = self
            .image_pool
            .lock()
            .insert(VulkanImageHolder::Unique(image));

        Image(handle)
    }

    fn create_image_view(&self, desc: &ImageViewDesc) -> Image {
        let mut image_pool = self.image_pool.lock();
        let image = image_pool.get_mut(desc.image.0).unwrap();

        let arc_image;
        match image {
            VulkanImageHolder::Shared(shared) => arc_image = shared.image.clone(),
            VulkanImageHolder::Unique(unique) => {
                let unique_image = ManualArc::new(unique.image.clone());
                arc_image = unique_image.clone();
                let unique_view = unique.view;
                *image = VulkanImageHolder::Shared(VulkanImageShared {
                    image: unique_image,
                    view: unique_view,
                })
            }
            VulkanImageHolder::Swapchain(_) => {
                panic!("unable to create additional views of swapchain images")
            }
        }

        let subresource_range = vulkan_subresource_range(&desc.subresource_range);
        let view_type =
            vulkan_image_view_type(desc.subresource_range.array_layer_count, desc.dimension);
        let format = vulkan_format(desc.format);

        let create_info = vk::ImageViewCreateInfo {
            image: arc_image.image,
            view_type,
            format,
            subresource_range,
            ..default()
        };

        let mut view = vk::ImageView::null();
        vk_check!(self
            .device_fn
            .create_image_view(self.device, &create_info, None, &mut view));

        let handle = image_pool.insert(VulkanImageHolder::Shared(VulkanImageShared {
            image: arc_image,
            view,
        }));

        Image(handle)
    }

    fn create_sampler(&self, desc: &SamplerDesc) -> Sampler {
        let (filter, mipmap_mode, anisotropy_enable) = match desc.filter {
            SamplerFilter::Point => (
                vk::Filter::Nearest,
                vk::SamplerMipmapMode::Nearest,
                vk::Bool32::False,
            ),
            SamplerFilter::Bilinear => (
                vk::Filter::Linear,
                vk::SamplerMipmapMode::Nearest,
                vk::Bool32::False,
            ),
            SamplerFilter::Trilinear => (
                vk::Filter::Linear,
                vk::SamplerMipmapMode::Linear,
                vk::Bool32::False,
            ),
            SamplerFilter::Anisotropic => (
                vk::Filter::Linear,
                vk::SamplerMipmapMode::Linear,
                vk::Bool32::True,
            ),
        };

        let address_mode = match desc.address_mode {
            SamplerAddressMode::Wrap => vk::SamplerAddressMode::Repeat,
            SamplerAddressMode::Clamp => vk::SamplerAddressMode::ClampToEdge,
        };

        let (compare_enable, compare_op) = match desc.compare_op {
            None => (vk::Bool32::False, vk::CompareOp::Always),
            Some(SamplerCompareOp::Less) => (vk::Bool32::True, vk::CompareOp::Less),
            Some(SamplerCompareOp::LessEq) => (vk::Bool32::True, vk::CompareOp::LessOrEqual),
            Some(SamplerCompareOp::Greater) => (vk::Bool32::True, vk::CompareOp::Greater),
            Some(SamplerCompareOp::GreaterEq) => (vk::Bool32::True, vk::CompareOp::GreaterOrEqual),
        };

        let mut sampler = vk::Sampler::null();
        vk_check!(self.device_fn.create_sampler(
            self.device,
            &vk::SamplerCreateInfo {
                max_lod: desc.max_lod,
                min_lod: desc.min_lod,
                mip_lod_bias: desc.mip_lod_bias,
                min_filter: filter,
                mag_filter: filter,
                mipmap_mode,
                anisotropy_enable,
                max_anisotropy: 16.0, // TODO: check maxSamplerAnisotropy
                address_mode_u: address_mode,
                address_mode_v: address_mode,
                address_mode_w: address_mode,
                compare_enable,
                compare_op,
                ..default()
            },
            None,
            &mut sampler,
        ));

        let handle = self.sampler_pool.lock().insert(VulkanSampler(sampler));
        Sampler(handle)
    }

    fn create_bind_group_layout(&self, desc: &BindGroupLayoutDesc) -> BindGroupLayout {
        let arena = HybridArena::<256>::new();
        let layout_bindings = arena.alloc_slice_fill_iter(desc.entries.iter().map(|x| {
            vk::DescriptorSetLayoutBinding {
                binding: x.slot,
                descriptor_type: vulkan_descriptor_type(x.binding_type),
                descriptor_count: x.count,
                stage_flags: vulkan_shader_stage_flags(x.stages),
                immutable_samplers: std::ptr::null(),
            }
        }));
        let create_info = &vk::DescriptorSetLayoutCreateInfo {
            bindings: layout_bindings.into(),
            ..default()
        };
        let mut set_layout = vk::DescriptorSetLayout::null();
        vk_check!(self.device_fn.create_descriptor_set_layout(
            self.device,
            create_info,
            None,
            &mut set_layout,
        ));
        let bind_group_layout = self
            .bind_group_layout_pool
            .lock()
            .insert(VulkanBindGroupLayout(set_layout));

        BindGroupLayout(bind_group_layout)
    }

    fn create_graphics_pipeline(&self, desc: &GraphicsPipelineDesc) -> Pipeline {
        let arena = HybridArena::<1024>::new();
        let bind_group_layout_pool = self.bind_group_layout_pool.lock();
        let set_layouts_iter = desc
            .bind_group_layouts
            .iter()
            .map(|bind_group_layout| bind_group_layout_pool.get(bind_group_layout.0).unwrap().0);
        let set_layouts = arena.alloc_slice_fill_iter(set_layouts_iter);

        let layout = {
            let create_info = vk::PipelineLayoutCreateInfo {
                set_layouts: set_layouts.into(),
                ..default()
            };
            let mut pipeline_layout = vk::PipelineLayout::null();
            vk_check!(self.device_fn.create_pipeline_layout(
                self.device,
                &create_info,
                None,
                &mut pipeline_layout,
            ));
            pipeline_layout
        };

        let shader_module = |code: &[u8]| {
            let create_info = vk::ShaderModuleCreateInfo {
                code: code.into(),
                ..default()
            };
            let mut shader_module = vk::ShaderModule::null();
            vk_check!(self.device_fn.create_shader_module(
                self.device,
                &create_info,
                None,
                &mut shader_module
            ));
            shader_module
        };

        let vertex_module = shader_module(desc.vertex_shader.code);
        let fragment_module = shader_module(desc.fragment_shader.code);

        let stages = &[
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::VERTEX,
                name: desc.vertex_shader.entry.as_ptr(),
                module: vertex_module,
                ..default()
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::FRAGMENT,
                name: desc.fragment_shader.entry.as_ptr(),
                module: fragment_module,
                ..default()
            },
        ];

        let topology = vulkan_primitive_topology(desc.topology);
        let polygon_mode = vulkan_polygon_mode(desc.polygon_mode);
        let cull_mode = vulkan_cull_mode(desc.culling_mode);
        let front_face = vulkan_front_face(desc.front_face);
        let (
            depth_bias_enable,
            depth_bias_constant_factor,
            depth_bias_clamp,
            depth_bias_slope_factor,
        ) = if let Some(depth_bias) = &desc.depth_bias {
            (
                vk::Bool32::True,
                depth_bias.constant_factor,
                depth_bias.clamp,
                depth_bias.slope_factor,
            )
        } else {
            (vk::Bool32::False, 0.0, 0.0, 0.0)
        };
        let depth_compare_op = vulkan_compare_op(desc.depth_compare_op);
        let depth_test_enable = vulkan_bool32(desc.depth_test_enable);
        let depth_write_enable = vulkan_bool32(desc.depth_write_enable);
        let stencil_test_enable = vulkan_bool32(desc.stencil_test_enable);
        let back = vulkan_stencil_op_state(desc.stencil_back);
        let front = vulkan_stencil_op_state(desc.stencil_front);

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            topology,
            ..default()
        };
        let viewport_state = vk::PipelineViewportStateCreateInfo::default();
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo {
            polygon_mode,
            cull_mode,
            front_face,
            line_width: 1.0,
            depth_bias_enable,
            depth_bias_constant_factor,
            depth_bias_clamp,
            depth_bias_slope_factor,
            ..default()
        };
        let multisample_state = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::SAMPLE_COUNT_1,
            ..default()
        };
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo {
            depth_compare_op,
            depth_test_enable,
            depth_write_enable,
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
            stencil_test_enable,
            back,
            front,
            ..default()
        };
        let color_blend_attachments = &[vulkan_blend_mode(desc.blend_mode)];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            attachments: color_blend_attachments.into(),
            ..default()
        };
        let dynamic_states = &[
            vk::DynamicState::ViewportWithCount,
            vk::DynamicState::ScissorWithCount,
        ];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            dynamic_states: dynamic_states.into(),
            ..default()
        };
        let color_attachment_formats = arena.alloc_slice_fill_iter(
            desc.layout
                .color_attachment_formats
                .iter()
                .copied()
                .map(vulkan_format),
        );
        let pipeline_rendering_create_info = vk::PipelineRenderingCreateInfo {
            view_mask: 0,
            color_attachment_formats: color_attachment_formats.into(),
            depth_attachment_format: desc
                .layout
                .depth_attachment_format
                .map_or(vk::Format::Undefined, vulkan_format),
            stencil_attachment_format: desc
                .layout
                .stencil_attachment_format
                .map_or(vk::Format::Undefined, vulkan_format),
            ..default()
        };

        let create_infos = &mut [vk::GraphicsPipelineCreateInfo {
            _next: unsafe {
                std::mem::transmute::<_, *mut c_void>(&pipeline_rendering_create_info)
            },
            stages: stages.into(),
            vertex_input_state: Some(&vertex_input_state),
            input_assembly_state: Some(&input_assembly_state),
            tessellation_state: None,
            viewport_state: Some(&viewport_state),
            rasterization_state: Some(&rasterization_state),
            multisample_state: Some(&multisample_state),
            depth_stencil_state: Some(&depth_stencil_state),
            color_blend_state: Some(&color_blend_state),
            dynamic_state: Some(&dynamic_state),
            layout,
            ..default()
        }];
        let mut pipelines = [vk::Pipeline::null()];
        vk_check!(self.device_fn.create_graphics_pipelines(
            self.device,
            vk::PipelineCache::null(),
            create_infos,
            None,
            &mut pipelines
        ));

        unsafe {
            self.device_fn
                .destroy_shader_module(self.device, vertex_module, None)
        };
        unsafe {
            self.device_fn
                .destroy_shader_module(self.device, fragment_module, None)
        };

        let handle = self.pipeline_pool.lock().insert(VulkanPipeline {
            pipeline: pipelines[0],
            pipeline_layout: layout,
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
        });

        Pipeline(handle)
    }

    fn create_compute_pipeline(&self, _desc: &ComputePipelineDesc) -> Pipeline {
        todo!()
    }

    fn destroy_buffer(&self, frame: &Frame, buffer: Buffer) {
        if let Some(buffer) = self.buffer_pool.lock().remove(buffer.0) {
            let frame = self.frame(frame);
            frame.destroyed_buffers.lock().push_back(buffer.buffer);
            frame.destroyed_allocations.lock().push_back(buffer.memory);
        }
    }

    fn destroy_image(&self, frame: &Frame, image: Image) {
        if let Some(image_holder) = self.image_pool.lock().remove(image.0) {
            let frame = self.frame(frame);

            match image_holder {
                // The image is unique, we've never allocated a reference counted object for it.
                VulkanImageHolder::Unique(image) => {
                    frame.destroyed_image_views.lock().push_back(image.view);
                    frame.destroyed_images.lock().push_back(image.image.image);
                    frame
                        .destroyed_allocations
                        .lock()
                        .push_back(image.image.memory);
                }
                // The image was at one point shared, we may or may not have the last reference.
                VulkanImageHolder::Shared(image) => {
                    frame.destroyed_image_views.lock().push_back(image.view);
                    // If we had the last reference we need to destroy the image and memory too
                    if let manual_arc::Release::Unique(image) = image.image.release() {
                        frame.destroyed_images.lock().push_back(image.image);
                        frame.destroyed_allocations.lock().push_back(image.memory);
                    }
                }
                VulkanImageHolder::Swapchain(_) => {
                    panic!("cannot directly destroy swapchain images")
                }
            }
        }
    }

    fn destroy_sampler(&self, frame: &Frame, sampler: Sampler) {
        if let Some(sampler) = self.sampler_pool.lock().remove(sampler.0) {
            self.frame(frame)
                .destroyed_samplers
                .lock()
                .push_back(sampler.0)
        }
    }

    fn destroy_bind_group_layout(&self, frame: &Frame, bind_group_layout: BindGroupLayout) {
        if let Some(bind_group_layout) = self
            .bind_group_layout_pool
            .lock()
            .remove(bind_group_layout.0)
        {
            self.frame(frame)
                .destroyed_descriptor_set_layouts
                .lock()
                .push_back(bind_group_layout.0)
        }
    }

    fn destroy_pipeline(&self, frame: &Frame, pipeline: Pipeline) {
        if let Some(pipeline) = self.pipeline_pool.lock().remove(pipeline.0) {
            let frame = self.frame(frame);
            frame
                .destroyed_pipeline_layouts
                .lock()
                .push_back(pipeline.pipeline_layout);
            frame
                .destroyed_pipelines
                .lock()
                .push_back(pipeline.pipeline);
        }
    }

    fn create_cmd_buffer(&self, frame: &Frame, thread_token: &mut ThreadToken) -> CmdBuffer {
        let frame = self.frame(frame);
        let per_thread = frame.per_thread.get_mut(thread_token);
        let cmd_buffer_pool = &mut per_thread.cmd_buffer_pool;

        // We have consumed all available command buffers, need to allocate a new one.
        if cmd_buffer_pool.next_free_index >= cmd_buffer_pool.command_buffers.len() {
            let mut cmd_buffers = [vk::CommandBuffer::null(); 4];
            let allocate_info = vk::CommandBufferAllocateInfo {
                command_pool: cmd_buffer_pool.command_pool,
                level: vk::CommandBufferLevel::Primary,
                command_buffer_count: cmd_buffers.len() as u32,
                ..default()
            };
            vk_check!(self.device_fn.allocate_command_buffers(
                self.device,
                &allocate_info,
                cmd_buffers.as_mut_ptr()
            ));
            cmd_buffer_pool.command_buffers.extend(cmd_buffers.iter());
        }

        let index = cmd_buffer_pool.next_free_index;
        cmd_buffer_pool.next_free_index += 1;
        let command_buffer = cmd_buffer_pool.command_buffers[index];

        vk_check!(self.device_fn.begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo {
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..default()
            }
        ));

        let vulkan_cmd_buffer = per_thread.arena.alloc(VulkanCmdBuffer {
            command_buffer,
            bound_pipeline: None,
            swapchains_touched: HashMap::new(),
        });

        CmdBuffer {
            cmd_buffer_addr: vulkan_cmd_buffer as *mut _ as usize,
            _phantom: &PhantomData,
            phantom_unsend: PhantomUnsend {},
        }
    }

    fn cmd_barrier(
        &self,
        cmd_buffer: &mut CmdBuffer,
        global_barrier: Option<&GlobalBarrier>,
        image_barriers: &[ImageBarrier],
    ) {
        let arena = HybridArena::<4096>::new();

        let memory_barriers = arena.alloc_slice_fill_iter(
            global_barrier
                .iter()
                .map(|global_barrier| vulkan_memory_barrier(global_barrier)),
        );

        let image_memory_barriers =
            arena.alloc_slice_fill_iter(image_barriers.iter().map(|image_barrier| {
                let image = self
                    .image_pool
                    .lock()
                    .get(image_barrier.image.0)
                    .expect("invalid image handle")
                    .image();
                let subresource_range = vulkan_subresource_range(&image_barrier.subresource_range);
                vulkan_image_memory_barrier(image_barrier, image, subresource_range)
            }));

        let command_buffer = self.cmd_buffer_mut(cmd_buffer).command_buffer;
        unsafe {
            self.device_fn.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo {
                    memory_barriers: memory_barriers.into(),
                    image_memory_barriers: image_memory_barriers.into(),
                    ..default()
                },
            )
        }
    }

    fn cmd_copy_buffer_to_image(
        &self,
        cmd_buffer: &mut CmdBuffer,
        src_buffer: Buffer,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        copies: &[BufferImageCopy],
    ) {
        let arena = HybridArena::<4096>::new();

        let regions = arena.alloc_slice_fill_iter(copies.iter().map(|copy| vk::BufferImageCopy {
            buffer_offset: copy.buffer_offset,
            buffer_row_length: copy.buffer_row_length,
            buffer_image_height: copy.buffer_image_height,
            image_subresource: vulkan_subresource_layers(&copy.image_subresource),
            image_offset: copy.image_offset.into(),
            image_extent: copy.image_extent.into(),
        }));

        let src_buffer = self
            .buffer_pool
            .lock()
            .get(src_buffer.0)
            .expect("invalid buffer handle")
            .buffer;

        let dst_image = self
            .image_pool
            .lock()
            .get(dst_image.0)
            .expect("invalid image handle")
            .image();

        let dst_image_layout = match dst_image_layout {
            ImageLayout::Optimal => vk::ImageLayout::TransferDstOptimal,
            ImageLayout::General => vk::ImageLayout::General,
        };

        let command_buffer = self.cmd_buffer_mut(cmd_buffer).command_buffer;
        unsafe {
            self.device_fn.cmd_copy_buffer_to_image(
                command_buffer,
                src_buffer,
                dst_image,
                dst_image_layout,
                regions,
            )
        }
    }

    fn cmd_blit_image(
        &self,
        cmd_buffer: &mut CmdBuffer,
        src_image: Image,
        src_image_layout: ImageLayout,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        regions: &[ImageBlit],
    ) {
        let arena = HybridArena::<4096>::new();

        let regions = arena.alloc_slice_fill_iter(regions.iter().map(|blit| vk::ImageBlit {
            src_subresource: vulkan_subresource_layers(&blit.src_subresource),
            src_offsets: [blit.src_offset_min.into(), blit.src_offset_max.into()],
            dst_subresource: vulkan_subresource_layers(&blit.dst_subresource),
            dst_offsets: [blit.dst_offset_min.into(), blit.dst_offset_max.into()],
        }));

        let src_image = self
            .image_pool
            .lock()
            .get(src_image.0)
            .expect("invalid src image handle")
            .image();

        let src_image_layout = match src_image_layout {
            ImageLayout::Optimal => vk::ImageLayout::TransferSrcOptimal,
            ImageLayout::General => vk::ImageLayout::General,
        };

        let dst_image = self
            .image_pool
            .lock()
            .get(dst_image.0)
            .expect("invalid dst image handle")
            .image();

        let dst_image_layout = match dst_image_layout {
            ImageLayout::Optimal => vk::ImageLayout::TransferDstOptimal,
            ImageLayout::General => vk::ImageLayout::General,
        };

        let command_buffer = self.cmd_buffer_mut(cmd_buffer).command_buffer;
        unsafe {
            self.device_fn.cmd_blit_image(
                command_buffer,
                src_image,
                src_image_layout,
                dst_image,
                dst_image_layout,
                regions,
                vk::Filter::Linear,
            );
        }
    }

    fn cmd_set_bind_group(
        &self,
        frame: &Frame,
        thread_token: &mut ThreadToken,
        cmd_buffer: &mut CmdBuffer,
        layout: BindGroupLayout,
        bind_group_index: u32,
        bindings: &[Bind],
    ) {
        let arena = HybridArena::<4096>::new();

        let descriptor_set_layout = self.bind_group_layout_pool.lock().get(layout.0).unwrap().0;

        let frame = self.frame(frame);
        let per_thread = frame.per_thread.get_mut(thread_token);

        let mut descriptor_pool = per_thread.descriptor_pool;
        let mut allocated_pool = false;
        let descriptor_set = loop {
            if descriptor_pool.is_null() {
                // Need to fetch a new descriptor pool
                descriptor_pool = self.request_descriptor_pool();
                frame.recycle_descriptor_pool(descriptor_pool);
                per_thread.descriptor_pool = descriptor_pool;
                allocated_pool = true;
            }
            let allocate_info = vk::DescriptorSetAllocateInfo {
                descriptor_pool,
                set_layouts: std::slice::from_ref(&descriptor_set_layout).into(),
                ..default()
            };
            let mut descriptor_set = vk::DescriptorSet::null();
            match unsafe {
                self.device_fn.allocate_descriptor_sets(
                    self.device,
                    &allocate_info,
                    &mut descriptor_set,
                )
            } {
                vk::Result::Success => break descriptor_set,
                _ => {
                    // If we fail to allocate after just creating a new descriptor set, then we'll
                    // never be able to allocate one. :'(
                    if allocated_pool {
                        panic!("failed to allocate descriptor set")
                    }
                }
            }
        };

        let write_descriptors_iter = bindings.iter().map(|bind| match bind.typed {
            TypedBind::Sampler(samplers) => {
                let sampler_infos_iter = samplers.iter().map(|sampler| {
                    let sampler = self.sampler_pool.lock().get(sampler.0).unwrap().0;
                    vk::DescriptorImageInfo {
                        image_layout: vk::ImageLayout::Undefined,
                        image_view: vk::ImageView::null(),
                        sampler,
                    }
                });
                let image_infos = arena.alloc_slice_fill_iter(sampler_infos_iter);
                vk::WriteDescriptorSet {
                    dst_set: descriptor_set,
                    dst_binding: bind.binding,
                    dst_array_element: bind.array_element,
                    descriptor_count: image_infos.len() as u32,
                    descriptor_type: vk::DescriptorType::Sampler,
                    image_info: image_infos.as_ptr(),
                    ..default()
                }
            }
            TypedBind::Image(images) => {
                let image_infos_iter = images.iter().map(|(image_layout, image)| {
                    let image_view = self.image_pool.lock().get(image.0).unwrap().image_view();
                    vk::DescriptorImageInfo {
                        image_layout: match image_layout {
                            ImageLayout::Optimal => vk::ImageLayout::ReadOnlyOptimal,
                            ImageLayout::General => vk::ImageLayout::General,
                        },
                        image_view,
                        sampler: vk::Sampler::null(),
                    }
                });
                let image_infos = arena.alloc_slice_fill_iter(image_infos_iter);
                vk::WriteDescriptorSet {
                    dst_set: descriptor_set,
                    dst_binding: bind.binding,
                    dst_array_element: bind.array_element,
                    descriptor_count: image_infos.len() as u32,
                    descriptor_type: vk::DescriptorType::SampledImage,
                    image_info: image_infos.as_ptr(),
                    ..default()
                }
            }
            TypedBind::UniformBuffer(buffers) => {
                let buffer_infos_iter = buffers.iter().map(|buffer| {
                    let buffer = self.buffer_pool.lock().get(buffer.0).unwrap().buffer;
                    vk::DescriptorBufferInfo {
                        buffer,
                        offset: 0,
                        range: !0,
                    }
                });
                let buffer_infos = arena.alloc_slice_fill_iter(buffer_infos_iter);
                vk::WriteDescriptorSet {
                    dst_set: descriptor_set,
                    dst_binding: bind.binding,
                    dst_array_element: bind.array_element,
                    descriptor_count: buffer_infos.len() as u32,
                    descriptor_type: vk::DescriptorType::UniformBuffer,
                    buffer_info: buffer_infos.as_ptr(),
                    ..default()
                }
            }
            TypedBind::StorageBuffer(buffers) => {
                let buffer_infos_iter = buffers.iter().map(|buffer| {
                    let buffer = self.buffer_pool.lock().get(buffer.0).unwrap().buffer;
                    vk::DescriptorBufferInfo {
                        buffer,
                        offset: 0,
                        range: !0,
                    }
                });
                let buffer_infos = arena.alloc_slice_fill_iter(buffer_infos_iter);
                vk::WriteDescriptorSet {
                    dst_set: descriptor_set,
                    dst_binding: bind.binding,
                    dst_array_element: bind.array_element,
                    descriptor_count: buffer_infos.len() as u32,
                    descriptor_type: vk::DescriptorType::StorageBuffer,
                    buffer_info: buffer_infos.as_ptr(),
                    ..default()
                }
            }
        });
        let write_descriptors = arena.alloc_slice_fill_iter(write_descriptors_iter);

        unsafe {
            self.device_fn
                .update_descriptor_sets(self.device, write_descriptors, &[])
        };

        let cmd_buffer = self.cmd_buffer_mut(cmd_buffer);
        let VulkanBoundPipeline {
            pipeline_layout,
            pipeline_bind_point,
        } = cmd_buffer
            .bound_pipeline
            .as_ref()
            .expect("cannot set bind groups without a pipeline bound")
            .clone();

        let command_buffer = cmd_buffer.command_buffer;

        unsafe {
            self.device_fn.cmd_bind_descriptor_sets(
                command_buffer,
                pipeline_bind_point,
                pipeline_layout,
                bind_group_index,
                &[descriptor_set],
                &[],
            )
        }
    }

    fn cmd_set_index_buffer(
        &self,
        cmd_buffer: &mut CmdBuffer,
        buffer: Buffer,
        offset: u64,
        index_type: IndexType,
    ) {
        let buffer = self.buffer_pool.lock().get(buffer.0).unwrap().buffer;
        let command_buffer = self.cmd_buffer_mut(cmd_buffer).command_buffer;
        let index_type = vulkan_index_type(index_type);
        unsafe {
            self.device_fn
                .cmd_bind_index_buffer(command_buffer, buffer, offset, index_type)
        }
    }

    fn cmd_set_pipeline(&self, cmd_buffer: &mut CmdBuffer, pipeline: Pipeline) {
        let cmd_buffer = self.cmd_buffer_mut(cmd_buffer);

        let VulkanPipeline {
            pipeline,
            pipeline_layout,
            pipeline_bind_point,
        } = *self.pipeline_pool.lock().get(pipeline.0).unwrap();

        cmd_buffer.bound_pipeline = Some(VulkanBoundPipeline {
            pipeline_layout,
            pipeline_bind_point,
        });

        let command_buffer = cmd_buffer.command_buffer;

        unsafe {
            self.device_fn
                .cmd_bind_pipeline(command_buffer, pipeline_bind_point, pipeline)
        };
    }

    fn cmd_begin_rendering(&self, cmd_buffer: &mut CmdBuffer, desc: &crate::RenderingDesc) {
        let cmd_buffer = self.cmd_buffer_mut(cmd_buffer);

        let color_attachments = desc
            .color_attachments
            .iter()
            .map(|attachment| {
                let image_view = match self.image_pool.lock().get(attachment.image.0).unwrap() {
                    VulkanImageHolder::Unique(image) => image.view,
                    VulkanImageHolder::Shared(image) => image.view,
                    VulkanImageHolder::Swapchain(image) => {
                        assert!(
                            !cmd_buffer.swapchains_touched.contains_key(&image.surface),
                            "swapchain attached multiple times in a command buffer"
                        );
                        cmd_buffer.swapchains_touched.insert(
                            image.surface,
                            (
                                image.image,
                                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                            ),
                        );

                        // transition swapchain image to optimal
                        let image_memory_barriers = &[vk::ImageMemoryBarrier2 {
                            src_stage_mask: vk::PipelineStageFlags2::TOP_OF_PIPE,
                            src_access_mask: vk::AccessFlags2::NONE,
                            dst_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                            dst_access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                            src_queue_family_index: self.universal_queue_family_index,
                            dst_queue_family_index: self.universal_queue_family_index,
                            old_layout: vk::ImageLayout::Undefined,
                            new_layout: vk::ImageLayout::AttachmentOptimal,
                            image: image.image,
                            subresource_range: vk::ImageSubresourceRange {
                                aspect_mask: vk::ImageAspectFlags::COLOR,
                                base_mip_level: 0,
                                level_count: !0,
                                base_array_layer: 0,
                                layer_count: !0,
                            },
                            ..default()
                        }];

                        let dependency_info = vk::DependencyInfo {
                            image_memory_barriers: image_memory_barriers.into(),
                            ..default()
                        };

                        unsafe {
                            self.device_fn
                                .cmd_pipeline_barrier2(cmd_buffer.command_buffer, &dependency_info)
                        };

                        image.view
                    }
                };

                let (load_op, clear_value) = vulkan_load_op(attachment.load_op);
                let store_op = vulkan_store_op(attachment.store_op);

                vk::RenderingAttachmentInfo {
                    image_view,
                    image_layout: vk::ImageLayout::ColorAttachmentOptimal,
                    load_op,
                    store_op,
                    clear_value,
                    ..default()
                }
            })
            .collect::<Vec<_>>();

        let depth_attachment = desc.depth_attachment.as_ref().map(|attachment| {
            let image_view = match self.image_pool.lock().get(attachment.image.0).unwrap() {
                VulkanImageHolder::Unique(image) => image.view,
                VulkanImageHolder::Shared(image) => image.view,
                VulkanImageHolder::Swapchain(_) => panic!(),
            };

            let (load_op, clear_value) = vulkan_load_op(attachment.load_op);
            let store_op = vulkan_store_op(attachment.store_op);

            vk::RenderingAttachmentInfo {
                image_view,
                image_layout: vk::ImageLayout::DepthAttachmentOptimal,
                load_op,
                store_op,
                clear_value,
                ..default()
            }
        });

        let rendering_info = vk::RenderingInfo {
            flags: vk::RenderingFlags::default(),
            render_area: vk::Rect2d {
                offset: vk::Offset2d {
                    x: desc.x,
                    y: desc.y,
                },
                extent: vk::Extent2d {
                    width: desc.width,
                    height: desc.height,
                },
            },
            layer_count: 1,
            view_mask: 0,
            color_attachments: color_attachments.as_slice().into(),
            depth_attachment: depth_attachment.as_ref(),
            stencil_attachment: None,
            ..default()
        };
        unsafe {
            self.device_fn
                .cmd_begin_rendering(cmd_buffer.command_buffer, &rendering_info)
        }
    }

    fn cmd_end_rendering(&self, cmd_buffer: &mut CmdBuffer) {
        let command_buffer = self.cmd_buffer_mut(cmd_buffer).command_buffer;
        unsafe { self.device_fn.cmd_end_rendering(command_buffer) }
    }

    fn cmd_set_viewports(&self, cmd_buffer: &mut CmdBuffer, viewports: &[crate::Viewport]) {
        let command_buffer = self.cmd_buffer_mut(cmd_buffer).command_buffer;
        unsafe {
            self.device_fn.cmd_set_viewport_with_count(
                command_buffer,
                std::mem::transmute::<_, &[vk::Viewport]>(viewports), // yolo
            );
        }
    }

    fn cmd_set_scissors(&self, cmd_buffer: &mut CmdBuffer, scissors: &[crate::Scissor]) {
        let command_buffer = self.cmd_buffer_mut(cmd_buffer).command_buffer;
        unsafe {
            self.device_fn.cmd_set_scissor_with_count(
                command_buffer,
                std::mem::transmute::<_, &[vk::Rect2d]>(scissors), // yolo
            );
        }
    }

    fn cmd_draw(
        &self,
        cmd_buffer: &mut CmdBuffer,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        let command_buffer = self.cmd_buffer_mut(cmd_buffer).command_buffer;
        unsafe {
            self.device_fn.cmd_draw(
                command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
    }

    fn cmd_draw_indexed(
        &self,
        cmd_buffer: &mut CmdBuffer,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        let command_buffer = self.cmd_buffer_mut(cmd_buffer).command_buffer;
        unsafe {
            self.device_fn.cmd_draw_indexed(
                command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            )
        }
    }

    fn submit(&self, frame: &Frame, mut cmd_buffer: CmdBuffer) {
        let fence = self.universal_queue_fence.fetch_add(1, Ordering::SeqCst) + 1;

        let frame = self.frame(frame);
        frame.universal_queue_fence.store(fence, Ordering::Relaxed);

        let cmd_buffer = self.cmd_buffer_mut(&mut cmd_buffer);

        for &(image, _) in cmd_buffer.swapchains_touched.values() {
            // transition swapchain image from attachment optimal to present src
            let image_memory_barriers = &[vk::ImageMemoryBarrier2 {
                src_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                dst_stage_mask: vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
                dst_access_mask: vk::AccessFlags2::NONE,
                src_queue_family_index: self.universal_queue_family_index,
                dst_queue_family_index: self.universal_queue_family_index,
                old_layout: vk::ImageLayout::AttachmentOptimal,
                new_layout: vk::ImageLayout::PresentSrcKhr,
                image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..default()
            }];
            let dependency_info = vk::DependencyInfo {
                image_memory_barriers: image_memory_barriers.into(),
                ..default()
            };
            unsafe {
                self.device_fn
                    .cmd_pipeline_barrier2(cmd_buffer.command_buffer, &dependency_info)
            };
        }

        vk_check!(self.device_fn.end_command_buffer(cmd_buffer.command_buffer));

        let mut wait_semaphores = Vec::new();
        let mut signal_semaphores = Vec::new();

        if !cmd_buffer.swapchains_touched.is_empty() {
            let mut present_swapchains = frame.present_swapchains.lock();

            for (swapchain, (_, stage_mask)) in cmd_buffer.swapchains_touched.drain() {
                let present_swapchain = present_swapchains
                    .get_mut(&swapchain)
                    .expect("presenting a swapchain that hasn't been acquired this frame");

                assert!(
                    !present_swapchain.acquire.is_null(),
                    "acquiring a swapchain image multiple times"
                );
                present_swapchain.release = self.request_transient_semaphore(frame);

                wait_semaphores.push(vk::SemaphoreSubmitInfo {
                    semaphore: present_swapchain.acquire,
                    stage_mask,
                    ..default()
                });
                signal_semaphores.push(vk::SemaphoreSubmitInfo {
                    semaphore: present_swapchain.release,
                    ..default()
                });
            }
        }

        signal_semaphores.push(vk::SemaphoreSubmitInfo {
            semaphore: self.universal_queue_semaphore,
            semaphore_value: fence,
            stage_mask: vk::PipelineStageFlags2::ALL_GRAPHICS,
            ..default()
        });

        let cmd_buffer_infos = &[vk::CommandBufferSubmitInfo {
            command_buffer: cmd_buffer.command_buffer,
            device_mask: 1,
            ..default()
        }];

        vk_check!(self.device_fn.queue_submit2(
            self.universal_queue,
            &[vk::SubmitInfo2 {
                wait_semaphore_infos: wait_semaphores.as_slice().into(),
                command_buffer_infos: cmd_buffer_infos.into(),
                signal_semaphore_infos: signal_semaphores.as_slice().into(),
                ..default()
            }],
            vk::Fence::null()
        ));
    }

    fn begin_frame(&self) -> Frame {
        let device_fn = &self.device_fn;
        let device = self.device;

        let mut frame = self.frame_counter.acquire(self as *const _ as usize);
        {
            let frame = self.frame_mut(&mut frame);

            {
                let semaphore_fences = &[frame
                    .universal_queue_fence
                    .load(std::sync::atomic::Ordering::Relaxed)];
                let semaphores = &[self.universal_queue_semaphore];
                let wait_info = vk::SemaphoreWaitInfo {
                    semaphores: (semaphores, semaphore_fences).into(),
                    ..default()
                };
                vk_check!(device_fn.wait_semaphores(device, &wait_info, !0));
            }

            for per_thread in frame.per_thread.slots_mut() {
                per_thread.descriptor_pool = vk::DescriptorPool::null();
                if per_thread.cmd_buffer_pool.next_free_index != 0 {
                    vk_check!(device_fn.reset_command_pool(
                        device,
                        per_thread.cmd_buffer_pool.command_pool,
                        vk::CommandPoolResetFlags::default()
                    ));

                    per_thread.cmd_buffer_pool.next_free_index = 0;
                }
                per_thread.arena.reset()
            }

            self.recycled_semaphores
                .lock()
                .extend(frame.recycled_semaphores.get_mut().drain(..));

            for descriptor_pool in frame.recycled_descriptor_pools.get_mut() {
                vk_check!(device_fn.reset_descriptor_pool(
                    device,
                    *descriptor_pool,
                    vk::DescriptorPoolResetFlags::default()
                ))
            }

            self.recycled_descriptor_pools
                .lock()
                .extend(frame.recycled_descriptor_pools.get_mut().drain(..));

            Self::destroy_deferred(device_fn, device, frame);

            self.destroyed_swapchains
                .lock()
                .expire(|(swapchain, surface, image_views)| {
                    self.destroy_swapchain_deferred(surface, swapchain, &image_views);
                });
        }

        frame
    }

    fn end_frame(&self, mut frame: Frame) {
        let arena = HybridArena::<512>::new();

        {
            let frame = self.frame_mut(&mut frame);

            self.swapchains.lock();

            let present_swapchains = frame.present_swapchains.get_mut();
            if !present_swapchains.is_empty() {
                for present_info in present_swapchains.values() {
                    assert!(
                        !present_info.release.is_null(),
                        "swapchain image was acquired, but not consumed"
                    );
                }

                let windows = arena.alloc_slice_fill_iter(present_swapchains.keys().copied());
                let wait_semaphores =
                    arena.alloc_slice_fill_iter(present_swapchains.values().map(|x| x.release));
                let swapchains =
                    arena.alloc_slice_fill_iter(present_swapchains.values().map(|x| x.swapchain));
                let swapchain_image_indices =
                    arena.alloc_slice_fill_iter(present_swapchains.values().map(|x| x.image_index));

                present_swapchains.clear();

                let results = arena.alloc_slice_fill_copy(swapchains.len(), vk::Result::Success);

                let present_info = vk::PresentInfoKHR {
                    wait_semaphores: wait_semaphores.into(),
                    swapchains: (swapchains, swapchain_image_indices).into(),
                    results: results.as_mut_ptr(),
                    ..default()
                };

                unsafe {
                    // check results below, so ignore this return value.
                    let _ = self
                        .swapchain_fn
                        .queue_present(self.universal_queue, &present_info);
                };

                for (i, &result) in results.iter().enumerate() {
                    match result {
                        vk::Result::Success => {}
                        vk::Result::SuboptimalKHR => {
                            // Yikes
                            if let VulkanSwapchainState::Occupied {
                                width: _,
                                height: _,
                                suboptimal,
                                swapchain: _,
                                image_views: _,
                            } = &mut self.swapchains.lock().get_mut(&windows[i]).unwrap().state
                            {
                                *suboptimal = true;
                            }
                        }
                        _ => vk_check!(result),
                    }
                }
            }
        }

        self.frame_counter.release(frame);
    }

    unsafe fn map_buffer(&self, buffer: Buffer) -> *mut u8 {
        let mut ptr = std::ptr::null_mut();
        if let Some(buffer) = self.buffer_pool.lock().get(buffer.0) {
            vk_check!(self.device_fn.map_memory(
                self.device,
                buffer.memory.memory,
                buffer.memory.offset,
                buffer.memory.size,
                vk::MemoryMapFlags::default(),
                &mut ptr
            ))
        }
        std::mem::transmute::<*mut c_void, *mut u8>(ptr)
    }

    unsafe fn unmap_buffer(&self, buffer: Buffer) {
        if let Some(buffer) = self.buffer_pool.lock().get(buffer.0) {
            self.device_fn
                .unmap_memory(self.device, buffer.memory.memory)
        }
    }

    fn acquire_swapchain(
        &self,
        frame: &Frame,
        window: &dyn AsRawWindow,
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> Result<(u32, u32, Image), SwapchainOutOfDateError> {
        let raw_window = window.as_raw_window();
        let mut surfaces = self.surfaces.lock();
        let surface = surfaces
            .entry(raw_window)
            .or_insert_with(|| match raw_window {
                RawWindow::Xcb(xcb) => {
                    let create_info = vk::XcbSurfaceCreateInfoKHR {
                        connection: xcb.connection,
                        window: xcb.window,
                        ..default()
                    };
                    let mut surface = vk::SurfaceKHR::null();
                    vk_check!(self.xcb_surface_fn.as_ref().unwrap().create_xcb_surface(
                        self.instance,
                        &create_info,
                        None,
                        &mut surface,
                    ));
                    surface
                }
                RawWindow::Xlib(xlib) => {
                    let create_info = vk::XlibSurfaceCreateInfoKHR {
                        display: xlib.display,
                        window: xlib.window,
                        ..default()
                    };
                    let mut surface = vk::SurfaceKHR::null();
                    vk_check!(self.xlib_surface_fn.as_ref().unwrap().create_xlib_surface(
                        self.instance,
                        &create_info,
                        None,
                        &mut surface,
                    ));
                    surface
                }
                RawWindow::Wayland(wayland) => {
                    let create_info = vk::WaylandSurfaceCreateInfoKHR {
                        display: wayland.display,
                        surface: wayland.surface,
                        ..default()
                    };
                    let mut surface = vk::SurfaceKHR::null();
                    vk_check!(self
                        .wayland_surface_fn
                        .as_ref()
                        .unwrap()
                        .create_wayland_surface(self.instance, &create_info, None, &mut surface,));
                    surface
                }
            });
        self.acquire_swapchain(frame, *surface, width, height, format)
    }

    fn destroy_swapchain(&self, window: &dyn AsRawWindow) {
        let raw_window = window.as_raw_window();
        if let Some(surface) = self.surfaces.lock().remove(&raw_window) {
            self.destroy_swapchain(surface)
        }
    }
}

impl VulkanDevice {
    fn acquire_swapchain(
        &self,
        frame: &Frame,
        surface: vk::SurfaceKHR,
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> Result<(u32, u32, Image), SwapchainOutOfDateError> {
        let format = vulkan_format(format);

        let mut swapchains = self.swapchains.lock();
        let mut vulkan_swapchain = swapchains.entry(surface).or_insert_with(|| {
            let mut supported = vk::Bool32::False;
            vk_check!(self.surface_fn.get_physical_device_surface_support(
                self.physical_device,
                self.universal_queue_family_index,
                surface,
                &mut supported
            ));

            assert_eq!(
                supported,
                vk::Bool32::True,
                "universal queue does not support presenting this surface"
            );

            let formats = vk_vec(|count, ptr| unsafe {
                self.surface_fn.get_physical_device_surface_formats(
                    self.physical_device,
                    surface,
                    count,
                    ptr,
                )
            })
            .into_boxed_slice();

            let present_modes = vk_vec(|count, ptr| unsafe {
                self.surface_fn.get_physical_device_surface_present_modes(
                    self.physical_device,
                    surface,
                    count,
                    ptr,
                )
            })
            .into_boxed_slice();

            let mut capabilities = vk::SurfaceCapabilitiesKHR::default();
            vk_check!(self.surface_fn.get_physical_device_surface_capabilities(
                self.physical_device,
                surface,
                &mut capabilities
            ));

            let surface_format = formats
                .iter()
                .copied()
                .find(|&x| x.format == format)
                .expect("failed to find matching surface format");

            VulkanSwapchain {
                surface_format,
                state: VulkanSwapchainState::Vacant,
                _formats: formats,
                _present_modes: present_modes,
                capabilities,
            }
        });

        assert_eq!(format, vulkan_swapchain.surface_format.format);

        let frame = self.frame(frame);
        let mut image_pool = self.image_pool.lock();

        let mut present_swapchains = frame.present_swapchains.lock();
        let present_info = match present_swapchains.entry(surface) {
            Entry::Occupied(_) => panic!("acquiring swapchain multiple times in a frame"),
            Entry::Vacant(entry) => entry.insert(default()),
        };

        vk_check!(self.surface_fn.get_physical_device_surface_capabilities(
            self.physical_device,
            surface,
            &mut vulkan_swapchain.capabilities
        ));

        let width = width.clamp(
            vulkan_swapchain.capabilities.min_image_extent.width,
            vulkan_swapchain.capabilities.max_image_extent.width,
        );
        let height = height.clamp(
            vulkan_swapchain.capabilities.min_image_extent.height,
            vulkan_swapchain.capabilities.max_image_extent.height,
        );

        let mut old_swapchain = vk::SwapchainKHR::null();
        loop {
            match &mut vulkan_swapchain.state {
                VulkanSwapchainState::Vacant => {
                    let mut new_swapchain = vk::SwapchainKHR::null();
                    let create_info = vk::SwapchainCreateInfoKHR {
                        surface,
                        min_image_count: vulkan_swapchain.capabilities.min_image_count,
                        image_format: vulkan_swapchain.surface_format.format,
                        image_color_space: vulkan_swapchain.surface_format.color_space,
                        image_extent: vk::Extent2d { width, height },
                        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                            | vk::ImageUsageFlags::TRANSFER_SRC
                            | vk::ImageUsageFlags::TRANSFER_DST,
                        image_array_layers: 1,
                        image_sharing_mode: vk::SharingMode::Exclusive,
                        pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
                        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                        present_mode: vk::PresentModeKHR::Fifo,
                        clipped: vk::Bool32::True,
                        old_swapchain,
                        ..default()
                    };
                    vk_check!(self.swapchain_fn.create_swapchain(
                        self.device,
                        &create_info,
                        None,
                        &mut new_swapchain
                    ));
                    assert!(!new_swapchain.is_null());

                    let images = vk_vec(|count, ptr| unsafe {
                        self.swapchain_fn.get_swapchain_images(
                            self.device,
                            new_swapchain,
                            count,
                            ptr,
                        )
                    });

                    let image_views = images
                        .iter()
                        .map(|&image| {
                            let create_info = vk::ImageViewCreateInfo {
                                image,
                                view_type: vk::ImageViewType::Type2d,
                                format: vulkan_swapchain.surface_format.format,
                                subresource_range: vk::ImageSubresourceRange {
                                    aspect_mask: vk::ImageAspectFlags::COLOR,
                                    base_mip_level: 0,
                                    level_count: 1,
                                    base_array_layer: 0,
                                    layer_count: 1,
                                },
                                ..default()
                            };
                            let mut view = vk::ImageView::null();
                            vk_check!(self.device_fn.create_image_view(
                                self.device,
                                &create_info,
                                None,
                                &mut view,
                            ));

                            let handle = image_pool.insert(VulkanImageHolder::Swapchain(
                                VulkanImageSwapchain {
                                    surface,
                                    image,
                                    view,
                                },
                            ));
                            Image(handle)
                        })
                        .collect::<Box<_>>();

                    vulkan_swapchain.state = VulkanSwapchainState::Occupied {
                        width,
                        height,
                        suboptimal: false,
                        swapchain: new_swapchain,
                        image_views,
                    };
                }
                VulkanSwapchainState::Occupied {
                    width: current_width,
                    height: current_height,
                    suboptimal,
                    swapchain,
                    image_views,
                } => {
                    let destroy_image_views =
                        |images: &mut Pool<VulkanImageHolder>| -> Box<[vk::ImageView]> {
                            let mut vulkan_image_views = Vec::new();
                            for &image_view in image_views.iter() {
                                match images.remove(image_view.0) {
                                    Some(VulkanImageHolder::Swapchain(VulkanImageSwapchain {
                                        surface: _,
                                        image: _,
                                        view,
                                    })) => vulkan_image_views.push(view),
                                    _ => panic!("swapchain image in wrong state"),
                                }
                            }
                            vulkan_image_views.into_boxed_slice()
                        };

                    let swapchain = *swapchain;

                    if width != *current_width || height != *current_height || *suboptimal {
                        let image_views = destroy_image_views(&mut image_pool);
                        old_swapchain = swapchain;
                        self.destroyed_swapchains.lock().push((
                            old_swapchain,
                            vk::SurfaceKHR::null(),
                            image_views,
                        ));

                        vulkan_swapchain.state = VulkanSwapchainState::Vacant;
                        continue;
                    }

                    let acquire = self.request_transient_semaphore(frame);
                    let mut image_index = 0;
                    match unsafe {
                        self.swapchain_fn.acquire_next_image2(
                            self.device,
                            &vk::AcquireNextImageInfoKHR {
                                swapchain,
                                timeout: !0,
                                semaphore: acquire,
                                fence: vk::Fence::null(),
                                device_mask: 1,
                                ..default()
                            },
                            &mut image_index,
                        )
                    } {
                        vk::Result::Success => {}
                        vk::Result::SuboptimalKHR => {
                            *suboptimal = true;
                        }
                        vk::Result::ErrorOutOfDateKHR => {
                            let image_views = destroy_image_views(&mut image_pool);

                            old_swapchain = swapchain;
                            self.destroyed_swapchains.lock().push((
                                old_swapchain,
                                vk::SurfaceKHR::null(),
                                image_views,
                            ));

                            vulkan_swapchain.state = VulkanSwapchainState::Vacant;
                            return Err(SwapchainOutOfDateError(()));
                        }
                        result => vk_check!(result),
                    }

                    present_info.acquire = acquire;
                    present_info.image_index = image_index;
                    present_info.swapchain = swapchain;
                    let view = image_views[image_index as usize];

                    return Ok((width, height, view));
                }
            }
        }
    }

    fn destroy_swapchain(&self, surface: vk::SurfaceKHR) {
        if let Some(VulkanSwapchain {
            surface_format: _,
            state,
            _formats: _,
            _present_modes: _,
            capabilities: _,
        }) = self.swapchains.lock().remove(&surface)
        {
            let mut image_pool = self.image_pool.lock();

            if let VulkanSwapchainState::Occupied {
                width: _,
                height: _,
                suboptimal: _,
                swapchain,
                image_views,
            } = state
            {
                let mut vulkan_image_views = Vec::new();
                for &image_view in image_views.iter() {
                    match image_pool.remove(image_view.0) {
                        Some(VulkanImageHolder::Swapchain(VulkanImageSwapchain {
                            surface: _,
                            image: _,
                            view,
                        })) => vulkan_image_views.push(view),
                        _ => panic!("swapchain image in wrong state"),
                    }
                }

                self.destroyed_swapchains.lock().push((
                    swapchain,
                    surface,
                    vulkan_image_views.into_boxed_slice(),
                ));
            }
        }
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        vk_check!(self.device_fn.device_wait_idle(self.device));

        let device_fn = &self.device_fn;
        let instance = self.instance;
        let device = self.device;

        for frame in self.frames.as_mut() {
            let frame = frame.get_mut();

            for semaphore in frame.recycled_semaphores.get_mut() {
                unsafe { device_fn.destroy_semaphore(device, *semaphore, None) }
            }

            for descriptor_pool in frame.recycled_descriptor_pools.get_mut() {
                unsafe { device_fn.destroy_descriptor_pool(device, *descriptor_pool, None) }
            }

            Self::destroy_deferred(device_fn, device, frame);

            let mut arena = HybridArena::<512>::new();

            for per_thread in frame.per_thread.slots_mut() {
                if !per_thread.cmd_buffer_pool.command_buffers.is_empty() {
                    arena.reset();
                    let command_buffers = arena.alloc_slice_fill_iter(
                        per_thread.cmd_buffer_pool.command_buffers.iter().copied(),
                    );
                    unsafe {
                        device_fn.free_command_buffers(
                            device,
                            per_thread.cmd_buffer_pool.command_pool,
                            command_buffers,
                        )
                    };
                }
                unsafe {
                    device_fn.destroy_command_pool(
                        device,
                        per_thread.cmd_buffer_pool.command_pool,
                        None,
                    )
                }
            }
        }

        for buffer in self.buffer_pool.get_mut().values() {
            unsafe { device_fn.destroy_buffer(device, buffer.buffer, None) }
            unsafe { device_fn.free_memory(device, buffer.memory.memory, None) }
        }

        {
            let mut image_views = Vec::new();
            let mut images = Vec::new();
            let mut memories = Vec::new();
            for image in self.image_pool.get_mut().values() {
                match image {
                    VulkanImageHolder::Unique(image) => {
                        image_views.push(image.view);
                        images.push(image.image.image);
                        memories.push(image.image.memory.memory);
                    }
                    VulkanImageHolder::Shared(image) => {
                        image_views.push(image.view);
                    }
                    VulkanImageHolder::Swapchain(image) => {
                        image_views.push(image.view);
                    }
                }
            }

            for image_view in image_views {
                unsafe { device_fn.destroy_image_view(device, image_view, None) }
            }

            for image in images {
                unsafe { device_fn.destroy_image(device, image, None) }
            }

            for memory in memories {
                unsafe { device_fn.free_memory(device, memory, None) }
            }
        }

        for sampler in self.sampler_pool.get_mut().values() {
            unsafe { device_fn.destroy_sampler(device, sampler.0, None) }
        }

        for pipeline in self.pipeline_pool.get_mut().values() {
            unsafe {
                self.device_fn
                    .destroy_pipeline_layout(self.device, pipeline.pipeline_layout, None)
            };
            unsafe { device_fn.destroy_pipeline(device, pipeline.pipeline, None) }
        }

        for descriptor_set_layout in self.bind_group_layout_pool.get_mut().values() {
            unsafe {
                device_fn.destroy_descriptor_set_layout(device, descriptor_set_layout.0, None)
            }
        }

        for semaphore in self
            .recycled_semaphores
            .get_mut()
            .iter()
            .chain(std::iter::once(&self.universal_queue_semaphore))
        {
            unsafe { device_fn.destroy_semaphore(device, *semaphore, None) }
        }

        for descriptor_pool in self.recycled_descriptor_pools.get_mut() {
            unsafe { device_fn.destroy_descriptor_pool(device, *descriptor_pool, None) }
        }

        {
            let destroyed_swapchains = self
                .destroyed_swapchains
                .get_mut()
                .drain(..)
                .collect::<Vec<_>>();
            for (_, (swapchain, surface, image_views)) in destroyed_swapchains {
                self.destroy_swapchain_deferred(surface, swapchain, &image_views);
            }
        }

        for (&surface, swapchain) in self.swapchains.get_mut().iter() {
            if let VulkanSwapchainState::Occupied {
                width: _,
                height: _,
                suboptimal: _,
                swapchain,
                image_views: _,
            } = swapchain.state
            {
                unsafe { self.swapchain_fn.destroy_swapchain(device, swapchain, None) }
            }
            unsafe { self.surface_fn.destroy_surface(instance, surface, None) }
        }

        unsafe { device_fn.destroy_device(device, None) }
        unsafe { self.instance_fn.destroy_instance(self.instance, None) };
    }
}
