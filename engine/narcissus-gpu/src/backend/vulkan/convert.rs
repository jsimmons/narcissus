//! Conversions between narcissus-gpu exposed types and vulkan types

use narcissus_core::default;
use vulkan_sys as vk;

use crate::{
    BindingType, BlendMode, BufferUsageFlags, ClearValue, CompareOp, CullingMode, FrontFace,
    ImageAspectFlags, ImageDimension, ImageFormat, ImageSubresourceLayers, ImageSubresourceRange,
    ImageTiling, ImageUsageFlags, IndexType, LoadOp, PolygonMode, ShaderStageFlags, StencilOp,
    StencilOpState, StoreOp, Topology,
};

#[must_use]
pub fn vulkan_bool32(b: bool) -> vk::Bool32 {
    match b {
        false => vk::Bool32::False,
        true => vk::Bool32::True,
    }
}

#[must_use]
pub fn vulkan_format(format: ImageFormat) -> vk::Format {
    match format {
        ImageFormat::R8_SRGB => vk::Format::R8_SRGB,
        ImageFormat::R8_UNORM => vk::Format::R8_UNORM,
        ImageFormat::RGBA8_SRGB => vk::Format::R8G8B8A8_SRGB,
        ImageFormat::RGBA8_UNORM => vk::Format::R8G8B8A8_UNORM,
        ImageFormat::RGBA16_FLOAT => vk::Format::R16G16B16A16_SFLOAT,
        ImageFormat::BGRA8_SRGB => vk::Format::B8G8R8A8_SRGB,
        ImageFormat::BGRA8_UNORM => vk::Format::B8G8R8A8_UNORM,
        ImageFormat::A2R10G10B10_UNORM => vk::Format::A2R10G10B10_UNORM_PACK32,
        ImageFormat::A2B10G10R10_UNORM => vk::Format::A2B10G10R10_UNORM_PACK32,
        ImageFormat::DEPTH_F32 => vk::Format::D32_SFLOAT,
    }
}

pub fn vulkan_aspect_for_format(format: ImageFormat) -> vk::ImageAspectFlags {
    match format {
        ImageFormat::R8_SRGB
        | ImageFormat::R8_UNORM
        | ImageFormat::BGRA8_SRGB
        | ImageFormat::BGRA8_UNORM
        | ImageFormat::RGBA8_SRGB
        | ImageFormat::RGBA8_UNORM
        | ImageFormat::RGBA16_FLOAT
        | ImageFormat::A2R10G10B10_UNORM
        | ImageFormat::A2B10G10R10_UNORM => vk::ImageAspectFlags::COLOR,
        ImageFormat::DEPTH_F32 => vk::ImageAspectFlags::DEPTH,
    }
}

pub fn vulkan_aspect(aspect: ImageAspectFlags) -> vk::ImageAspectFlags {
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

pub fn vulkan_buffer_usage_flags(usage: BufferUsageFlags) -> vk::BufferUsageFlags {
    let mut usage_flags = vk::BufferUsageFlags::default();
    if usage.contains(BufferUsageFlags::UNIFORM) {
        usage_flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
    }
    if usage.contains(BufferUsageFlags::STORAGE) {
        usage_flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
    }
    if usage.contains(BufferUsageFlags::INDEX) {
        usage_flags |= vk::BufferUsageFlags::INDEX_BUFFER;
    }
    if usage.contains(BufferUsageFlags::TRANSFER) {
        usage_flags |= vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST;
    }
    usage_flags
}

pub fn vulkan_image_usage_flags(usage: ImageUsageFlags) -> vk::ImageUsageFlags {
    let mut usage_flags = vk::ImageUsageFlags::default();
    if usage.contains(ImageUsageFlags::SAMPLED) {
        usage_flags |= vk::ImageUsageFlags::SAMPLED;
    }
    if usage.contains(ImageUsageFlags::STORAGE) {
        usage_flags |= vk::ImageUsageFlags::STORAGE;
    }
    if usage.contains(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
        usage_flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
    }
    if usage.contains(ImageUsageFlags::COLOR_ATTACHMENT) {
        usage_flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
    }
    if usage.contains(ImageUsageFlags::TRANSFER) {
        usage_flags |= vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC;
    }
    usage_flags
}

#[must_use]
pub fn vulkan_clear_value(clear_value: ClearValue) -> vk::ClearValue {
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
pub fn vulkan_load_op(load_op: LoadOp) -> (vk::AttachmentLoadOp, vk::ClearValue) {
    match load_op {
        LoadOp::Load => (vk::AttachmentLoadOp::Load, vk::ClearValue::default()),
        LoadOp::Clear(clear_value) => {
            (vk::AttachmentLoadOp::Clear, vulkan_clear_value(clear_value))
        }
        LoadOp::DontCare => (vk::AttachmentLoadOp::DontCare, vk::ClearValue::default()),
    }
}

#[must_use]
pub fn vulkan_store_op(store_op: StoreOp) -> vk::AttachmentStoreOp {
    match store_op {
        StoreOp::Store => vk::AttachmentStoreOp::Store,
        StoreOp::DontCare => vk::AttachmentStoreOp::DontCare,
    }
}

#[must_use]
pub fn vulkan_shader_stage_flags(stage_flags: ShaderStageFlags) -> vk::ShaderStageFlags {
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
pub fn vulkan_descriptor_type(binding_type: BindingType) -> vk::DescriptorType {
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
pub fn vulkan_index_type(index_type: IndexType) -> vk::IndexType {
    match index_type {
        IndexType::U16 => vk::IndexType::Uint16,
        IndexType::U32 => vk::IndexType::Uint32,
    }
}

#[must_use]
pub fn vulkan_primitive_topology(primitive_topology: Topology) -> vk::PrimitiveTopology {
    match primitive_topology {
        Topology::Points => vk::PrimitiveTopology::PointList,
        Topology::Lines => vk::PrimitiveTopology::LineList,
        Topology::LineStrip => vk::PrimitiveTopology::LineStrip,
        Topology::Triangles => vk::PrimitiveTopology::TriangleList,
        Topology::TriangleStrip => vk::PrimitiveTopology::TriangleStrip,
    }
}

#[must_use]
pub fn vulkan_polygon_mode(polygon_mode: PolygonMode) -> vk::PolygonMode {
    match polygon_mode {
        PolygonMode::Fill => vk::PolygonMode::Fill,
        PolygonMode::Line => vk::PolygonMode::Line,
        PolygonMode::Point => vk::PolygonMode::Point,
    }
}

#[must_use]
pub fn vulkan_cull_mode(culling_mode: CullingMode) -> vk::CullModeFlags {
    match culling_mode {
        CullingMode::None => vk::CullModeFlags::NONE,
        CullingMode::Front => vk::CullModeFlags::FRONT,
        CullingMode::Back => vk::CullModeFlags::BACK,
    }
}

#[must_use]
pub fn vulkan_front_face(front_face: FrontFace) -> vk::FrontFace {
    match front_face {
        FrontFace::Clockwise => vk::FrontFace::Clockwise,
        FrontFace::CounterClockwise => vk::FrontFace::CounterClockwise,
    }
}

#[must_use]
pub fn vulkan_compare_op(compare_op: CompareOp) -> vk::CompareOp {
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
pub fn vulkan_stencil_op(stencil_op: StencilOp) -> vk::StencilOp {
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
pub fn vulkan_stencil_op_state(stencil_op_state: StencilOpState) -> vk::StencilOpState {
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
pub fn vulkan_blend_mode(blend_mode: BlendMode) -> vk::PipelineColorBlendAttachmentState {
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
pub fn vulkan_image_view_type(
    layer_count: u32,
    image_dimension: ImageDimension,
) -> vk::ImageViewType {
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

pub fn vulkan_subresource_layers(
    subresource_layers: &ImageSubresourceLayers,
) -> vk::ImageSubresourceLayers {
    vk::ImageSubresourceLayers {
        aspect_mask: vulkan_aspect(subresource_layers.aspect),
        mip_level: subresource_layers.mip_level,
        base_array_layer: subresource_layers.base_array_layer,
        layer_count: subresource_layers.array_layer_count,
    }
}

pub fn vulkan_subresource_range(subresource: &ImageSubresourceRange) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange {
        aspect_mask: vulkan_aspect(subresource.aspect),
        base_mip_level: subresource.base_mip_level,
        level_count: subresource.mip_level_count,
        base_array_layer: subresource.base_array_layer,
        layer_count: subresource.array_layer_count,
    }
}

pub fn vulkan_image_tiling(tiling: ImageTiling) -> vk::ImageTiling {
    match tiling {
        ImageTiling::Linear => vk::ImageTiling::LINEAR,
        ImageTiling::Optimal => vk::ImageTiling::OPTIMAL,
    }
}
