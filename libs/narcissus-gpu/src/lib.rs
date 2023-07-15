use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use backend::vulkan;
use narcissus_core::{
    default, flags_def, raw_window::AsRawWindow, thread_token_def, Handle, PhantomUnsend,
};

mod backend;
mod delay_queue;
mod frame_counter;
pub mod tlsf;

pub enum DeviceBackend {
    Vulkan,
}

pub fn create_device(backend: DeviceBackend) -> Box<dyn Device> {
    match backend {
        DeviceBackend::Vulkan => Box::new(vulkan::VulkanDevice::new()),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Offset2d {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Extent2d {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Offset3d {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Extent3d {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Image(Handle);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Buffer(Handle);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sampler(Handle);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindGroupLayout(Handle);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pipeline(Handle);

pub struct TransientBuffer<'a> {
    ptr: NonNull<u8>,
    len: usize,
    buffer: u64,
    offset: u64,
    _phantom: &'a PhantomData<()>,
}

impl<'a> TransientBuffer<'a> {
    pub fn copy_from_slice(&mut self, bytes: &[u8]) {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
            .copy_from_slice(bytes)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MemoryLocation {
    HostMapped,
    Device,
}

#[repr(C)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[repr(C)]
pub struct Scissor {
    pub offset: Offset2d,
    pub extent: Extent2d,
}

impl Scissor {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            offset: Offset2d { x, y },
            extent: Extent2d { width, height },
        }
    }
}

flags_def!(ShaderStageFlags);
impl ShaderStageFlags {
    pub const VERTEX: Self = Self(1 << 0);
    pub const FRAGMENT: Self = Self(1 << 1);
    pub const COMPUTE: Self = Self(1 << 2);
    pub const ALL: Self = Self(0b111); /* Self::VERTEX | Self::FRAGMENT | Self::COMPUTE */
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImageDimension {
    Type1d,
    Type2d,
    Type3d,
    TypeCube,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ImageFormat {
    R8_SRGB,
    R8_UNORM,
    BGRA8_SRGB,
    BGRA8_UNORM,
    RGBA8_SRGB,
    RGBA8_UNORM,
    DEPTH_F32,
}

flags_def!(ImageAspectFlags);
impl ImageAspectFlags {
    pub const COLOR: Self = Self(1 << 0);
    pub const DEPTH: Self = Self(1 << 1);
    pub const STENCIL: Self = Self(1 << 2);
}

flags_def!(ImageUsageFlags);
impl ImageUsageFlags {
    pub const SAMPLED: Self = Self(1 << 0);
    pub const STORAGE: Self = Self(1 << 1);
    pub const COLOR_ATTACHMENT: Self = Self(1 << 2);
    pub const DEPTH_STENCIL_ATTACHMENT: Self = Self(1 << 3);
    pub const TRANSFER: Self = Self(1 << 4);
}

pub struct ImageSubresourceLayers {
    pub aspect: ImageAspectFlags,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub array_layer_count: u32,
}

impl Default for ImageSubresourceLayers {
    fn default() -> Self {
        Self {
            aspect: ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            array_layer_count: 1,
        }
    }
}

pub struct ImageSubresourceRange {
    pub aspect: ImageAspectFlags,
    pub base_mip_level: u32,
    pub mip_level_count: u32,
    pub base_array_layer: u32,
    pub array_layer_count: u32,
}

impl ImageSubresourceRange {
    /// Constant that can be used to represent "all remaining mip levels / array layers" in an
    /// `ImageSubresourceRange`
    pub const ALL_REMAINING: u32 = !0;
}

impl Default for ImageSubresourceRange {
    fn default() -> Self {
        Self {
            aspect: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            mip_level_count: ImageSubresourceRange::ALL_REMAINING,
            base_array_layer: 0,
            array_layer_count: ImageSubresourceRange::ALL_REMAINING,
        }
    }
}

flags_def!(BufferUsageFlags);
impl BufferUsageFlags {
    pub const UNIFORM: Self = Self(1 << 0);
    pub const STORAGE: Self = Self(1 << 1);
    pub const INDEX: Self = Self(1 << 2);
    pub const TRANSFER: Self = Self(1 << 3);
}

pub struct BufferDesc {
    pub location: MemoryLocation,
    pub usage: BufferUsageFlags,
    pub size: usize,
}

pub struct ImageDesc {
    pub location: MemoryLocation,
    pub usage: ImageUsageFlags,
    pub dimension: ImageDimension,
    pub format: ImageFormat,
    pub initial_layout: ImageLayout,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub layer_count: u32,
    pub mip_levels: u32,
}

pub struct ImageViewDesc {
    pub image: Image,
    pub dimension: ImageDimension,
    pub format: ImageFormat,
    pub subresource_range: ImageSubresourceRange,
}

pub struct BufferImageCopy {
    pub buffer_offset: u64,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: ImageSubresourceLayers,
    pub image_offset: Offset3d,
    pub image_extent: Extent3d,
}

pub struct ImageBlit {
    pub src_subresource: ImageSubresourceLayers,
    pub src_offset_min: Offset3d,
    pub src_offset_max: Offset3d,
    pub dst_subresource: ImageSubresourceLayers,
    pub dst_offset_min: Offset3d,
    pub dst_offset_max: Offset3d,
}

pub struct ShaderDesc<'a> {
    pub entry: &'a CStr,
    pub code: &'a [u8],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SamplerFilter {
    Point,
    Bilinear,
    Trilinear,
    Anisotropic,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SamplerCompareOp {
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SamplerAddressMode {
    Wrap,
    Clamp,
}

pub struct SamplerDesc {
    pub filter: SamplerFilter,
    pub address_mode: SamplerAddressMode,
    pub compare_op: Option<SamplerCompareOp>,
    pub mip_lod_bias: f32,
    pub min_lod: f32,
    pub max_lod: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Topology {
    Points = 0,
    Lines = 1,
    LineStrip = 2,
    Triangles = 3,
    TriangleStrip = 4,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CullingMode {
    None,
    Front,
    Back,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BlendMode {
    Opaque,
    Mask,
    Translucent,
    Premultiplied,
    Additive,
    Modulate,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    IncrementAndClamp,
    DecrementAndClamp,
    Invert,
    IncrementAndWrap,
    DecrementAndWrap,
}

#[derive(Clone, Copy)]
pub struct StencilOpState {
    pub fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub compare_op: CompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

impl Default for StencilOpState {
    fn default() -> Self {
        Self {
            fail_op: StencilOp::Keep,
            pass_op: StencilOp::Keep,
            depth_fail_op: StencilOp::Keep,
            compare_op: CompareOp::Never,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        }
    }
}

pub struct DepthBias {
    pub constant_factor: f32,
    pub clamp: f32,
    pub slope_factor: f32,
}

pub struct GraphicsPipelineLayout<'a> {
    pub color_attachment_formats: &'a [ImageFormat],
    pub depth_attachment_format: Option<ImageFormat>,
    pub stencil_attachment_format: Option<ImageFormat>,
}

pub struct GraphicsPipelineDesc<'a> {
    pub vertex_shader: ShaderDesc<'a>,
    pub fragment_shader: ShaderDesc<'a>,
    pub bind_group_layouts: &'a [BindGroupLayout],
    pub layout: GraphicsPipelineLayout<'a>,
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
    pub culling_mode: CullingMode,
    pub front_face: FrontFace,
    pub blend_mode: BlendMode,
    pub depth_bias: Option<DepthBias>,
    pub depth_compare_op: CompareOp,
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub stencil_test_enable: bool,
    pub stencil_back: StencilOpState,
    pub stencil_front: StencilOpState,
}

pub struct ComputePipelineDesc<'a> {
    pub shader: ShaderDesc<'a>,
    pub bind_group_layouts: &'a [BindGroupLayout],
}

#[derive(Clone, Copy, Debug)]
pub enum ClearValue {
    ColorF32([f32; 4]),
    ColorU32([u32; 4]),
    ColorI32([i32; 4]),
    DepthStencil { depth: f32, stencil: u32 },
}

#[derive(Clone, Copy, Debug)]
pub enum LoadOp {
    Load,
    Clear(ClearValue),
    DontCare,
}

#[derive(Clone, Copy, Debug)]
pub enum StoreOp {
    Store,
    DontCare,
}

pub struct RenderingAttachment {
    pub image: Image,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
}

pub struct RenderingDesc<'a> {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub color_attachments: &'a [RenderingAttachment],
    pub depth_attachment: Option<RenderingAttachment>,
    pub stencil_attachment: Option<RenderingAttachment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexType {
    U16,
    U32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingType {
    Sampler,
    Image,
    UniformBuffer,
    StorageBuffer,
    DynamicUniformBuffer,
    DynamicStorageBuffer,
}

pub struct BindGroupLayoutEntryDesc {
    pub slot: u32,
    pub stages: ShaderStageFlags,
    pub binding_type: BindingType,
    pub count: u32,
}

pub struct BindGroupLayoutDesc<'a> {
    pub entries: &'a [BindGroupLayoutEntryDesc],
}

pub struct Bind<'a> {
    pub binding: u32,
    pub array_element: u32,
    pub typed: TypedBind<'a>,
}

pub enum BufferBind<'a> {
    Unmanaged(Buffer),
    Transient(TransientBuffer<'a>),
}

impl<'a> From<Buffer> for BufferBind<'a> {
    fn from(value: Buffer) -> Self {
        BufferBind::Unmanaged(value)
    }
}

impl<'a> From<TransientBuffer<'a>> for BufferBind<'a> {
    fn from(value: TransientBuffer<'a>) -> Self {
        BufferBind::Transient(value)
    }
}

pub enum TypedBind<'a> {
    Sampler(&'a [Sampler]),
    Image(&'a [(ImageLayout, Image)]),
    UniformBuffer(&'a [BufferBind<'a>]),
    StorageBuffer(&'a [BufferBind<'a>]),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// No access.
    None,

    /// Read as an indirect buffer for drawing or dispatch.
    IndirectBuffer,
    /// Read as an index buffer.
    IndexBuffer,
    /// Read as a vertex buffer.
    VertexBuffer,

    /// Read as a uniform buffer in a vertex shader.
    VertexShaderUniformBufferRead,
    /// Read as a sampled image or uniform texel buffer in a vertex shader.
    VertexShaderSampledImageRead,
    /// Read as any other resource in a vertex shader.
    VertexShaderOtherRead,

    /// Read as a uniform buffer in a fragment shader.
    FragmentShaderUniformBufferRead,
    /// Read as a sampled image or uniform texel buffer in a fragment shader.
    FragmentShaderSampledImageRead,
    /// Read as any other resource in a fragment shader.
    FragmentShaderOtherRead,

    /// Read as a color attachement.
    ColorAttachmentRead,
    /// Read as a depth-stencil attachment.
    DepthStencilAttachmentRead,

    /// Read as a uniform buffer in any shader.
    ShaderUniformBufferRead,
    /// Read as a uniform buffer or vertex buffer in any shader.
    ShaderUniformBufferOrVertexBufferRead,
    /// Read as a sampled image or uniform texel buffer in any shader.
    ShaderSampledImageRead,
    /// Read as any other resource (excluding attachments) in any shader.
    ShaderOtherRead,

    /// Read as the source of a transfer operation.
    TransferRead,
    /// Read on the host.
    HostRead,

    /// Read by the presentation engine.
    PresentRead,

    /// Written as any resource in a vertex shader.
    VertexShaderWrite,
    /// Written as any resource in a fragment shader.
    FragmentShaderWrite,

    /// Written as a color attachment during rendering.
    ColorAttachmentWrite,
    /// Written as a depth-stencil attachment during rendering.
    DepthStencilAttachmentWrite,

    /// Written as any resource in any shader.
    ShaderWrite,

    /// Written as the destination of a transfer operation.
    TransferWrite,
    /// Pre-initialized on the host before device access starts.
    HostPreInitializedWrite,
    /// Written on the host.
    HostWrite,

    /// Read or written as a color attachment during rendering.
    ColorAttachmentReadWrite,

    /// Covers any access. Slow mode like snail.
    General,
}

impl Access {
    /// Check whether this access type exclusively reads.
    pub fn is_read(self) -> bool {
        match self {
            Access::None => true,
            Access::IndirectBuffer => true,
            Access::IndexBuffer => true,
            Access::VertexBuffer => true,
            Access::VertexShaderUniformBufferRead => true,
            Access::VertexShaderSampledImageRead => true,
            Access::VertexShaderOtherRead => true,
            Access::FragmentShaderUniformBufferRead => true,
            Access::FragmentShaderSampledImageRead => true,
            Access::FragmentShaderOtherRead => true,
            Access::ColorAttachmentRead => true,
            Access::DepthStencilAttachmentRead => true,
            Access::ShaderUniformBufferRead => true,
            Access::ShaderUniformBufferOrVertexBufferRead => true,
            Access::ShaderSampledImageRead => true,
            Access::ShaderOtherRead => true,
            Access::TransferRead => true,
            Access::HostRead => true,
            Access::PresentRead => true,
            Access::VertexShaderWrite => false,
            Access::FragmentShaderWrite => false,
            Access::ColorAttachmentWrite => false,
            Access::DepthStencilAttachmentWrite => false,
            Access::ShaderWrite => false,
            Access::TransferWrite => false,
            Access::HostPreInitializedWrite => false,
            Access::HostWrite => false,
            Access::ColorAttachmentReadWrite => false,
            Access::General => false,
        }
    }

    /// Check whether this access type contains a write.
    pub fn is_write(self) -> bool {
        !self.is_read()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImageLayout {
    Optimal,
    General,
}

pub struct GlobalBarrier<'a> {
    pub prev_access: &'a [Access],
    pub next_access: &'a [Access],
}

pub struct ImageBarrier<'a> {
    pub prev_access: &'a [Access],
    pub next_access: &'a [Access],
    pub prev_layout: ImageLayout,
    pub next_layout: ImageLayout,
    pub image: Image,
    pub subresource_range: ImageSubresourceRange,
}

impl<'a> ImageBarrier<'a> {
    pub fn layout_optimal(
        prev_access: &'a [Access],
        next_access: &'a [Access],
        image: Image,
        aspect: ImageAspectFlags,
    ) -> ImageBarrier<'a> {
        Self {
            prev_access,
            next_access,
            prev_layout: ImageLayout::Optimal,
            next_layout: ImageLayout::Optimal,
            image,
            subresource_range: ImageSubresourceRange {
                aspect,
                ..default()
            },
        }
    }
}

thread_token_def!(ThreadToken, GpuConcurrent, 8);

pub struct Frame<'a> {
    device_addr: usize,
    frame_index: usize,
    _phantom: &'a PhantomData<()>,
}

impl<'a> Frame<'a> {
    fn check_device(&self, device_addr: usize) {
        assert_eq!(self.device_addr, device_addr, "frame device mismatch")
    }

    fn check_frame_counter(&self, frame_counter_value: usize) {
        assert!(frame_counter_value & 1 == 0, "frame counter isn't acquired");
        assert_eq!(
            self.frame_index,
            frame_counter_value >> 1,
            "frame does not match device frame"
        );
    }
}

pub struct CmdBuffer<'a, 'thread> {
    cmd_buffer_addr: usize,
    thread_token: &'thread ThreadToken,
    _phantom: &'a PhantomData<()>,
    phantom_unsend: PhantomUnsend,
}

#[derive(Debug)]
pub struct SwapchainOutOfDateError(());

impl std::fmt::Display for SwapchainOutOfDateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "swapchain out of date")
    }
}

impl std::error::Error for SwapchainOutOfDateError {}

pub trait Device {
    fn create_buffer(&self, desc: &BufferDesc) -> Buffer;
    fn create_buffer_with_data(&self, desc: &BufferDesc, inital_data: &[u8]) -> Buffer;
    fn create_image(&self, desc: &ImageDesc) -> Image;
    fn create_image_view(&self, desc: &ImageViewDesc) -> Image;
    fn create_sampler(&self, desc: &SamplerDesc) -> Sampler;
    fn create_bind_group_layout(&self, desc: &BindGroupLayoutDesc) -> BindGroupLayout;
    fn create_graphics_pipeline(&self, desc: &GraphicsPipelineDesc) -> Pipeline;
    fn create_compute_pipeline(&self, desc: &ComputePipelineDesc) -> Pipeline;

    fn destroy_buffer(&self, frame: &Frame, buffer: Buffer);
    fn destroy_image(&self, frame: &Frame, image: Image);
    fn destroy_sampler(&self, frame: &Frame, sampler: Sampler);
    fn destroy_bind_group_layout(&self, frame: &Frame, bind_group_layout: BindGroupLayout);
    fn destroy_pipeline(&self, frame: &Frame, pipeline: Pipeline);

    fn acquire_swapchain(
        &self,
        frame: &Frame,
        window: &dyn AsRawWindow,
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> Result<(u32, u32, Image), SwapchainOutOfDateError>;

    fn destroy_swapchain(&self, window: &dyn AsRawWindow);

    /// Map the given buffer in its entirety to system memory and return a pointer to it.
    ///
    /// # Safety
    ///
    /// `buffer` must be host mappable.
    unsafe fn map_buffer(&self, buffer: Buffer) -> *mut u8;

    /// Unmap from system memory a buffer previously mapped.
    ///
    /// # Safety
    ///
    /// This will invalidate the pointer returned previously from `map_buffer`, so there must not be
    /// any remaining references derived from that address.
    unsafe fn unmap_buffer(&self, buffer: Buffer);

    #[must_use]
    fn request_transient_uniform_buffer<'a>(
        &self,
        frame: &'a Frame<'a>,
        thread_token: &'a ThreadToken,
        size: usize,
        align: usize,
    ) -> TransientBuffer<'a>;

    #[must_use]
    fn request_transient_storage_buffer<'a>(
        &self,
        frame: &'a Frame<'a>,
        thread_token: &'a ThreadToken,
        size: usize,
        align: usize,
    ) -> TransientBuffer<'a>;

    #[must_use]
    fn request_transient_index_buffer<'a>(
        &self,
        frame: &'a Frame<'a>,
        thread_token: &'a ThreadToken,
        size: usize,
        align: usize,
    ) -> TransientBuffer<'a>;

    #[must_use]
    fn create_cmd_buffer<'a, 'thread>(
        &'a self,
        frame: &'a Frame,
        thread_token: &'thread ThreadToken,
    ) -> CmdBuffer<'a, 'thread>;

    fn cmd_set_bind_group(
        &self,
        frame: &Frame,
        cmd_buffer: &mut CmdBuffer,
        layout: BindGroupLayout,
        bind_group_index: u32,
        bindings: &[Bind],
    );

    fn cmd_set_index_buffer(
        &self,
        cmd_buffer: &mut CmdBuffer,
        buffer: Buffer,
        offset: u64,
        index_type: IndexType,
    );

    fn cmd_set_pipeline(&self, cmd_buffer: &mut CmdBuffer, pipeline: Pipeline);

    fn cmd_set_viewports(&self, cmd_buffer: &mut CmdBuffer, viewports: &[Viewport]);

    fn cmd_set_scissors(&self, cmd_buffer: &mut CmdBuffer, scissors: &[Scissor]);

    fn cmd_barrier(
        &self,
        cmd_buffer: &mut CmdBuffer,
        global_barrier: Option<&GlobalBarrier>,
        image_barriers: &[ImageBarrier],
    );

    fn cmd_copy_buffer_to_image(
        &self,
        cmd_buffer: &mut CmdBuffer,
        src_buffer: Buffer,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        copies: &[BufferImageCopy],
    );

    fn cmd_blit_image(
        &self,
        cmd_buffer: &mut CmdBuffer,
        src_image: Image,
        src_image_layout: ImageLayout,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        regions: &[ImageBlit],
    );

    fn cmd_begin_rendering(&self, cmd_buffer: &mut CmdBuffer, desc: &RenderingDesc);

    fn cmd_end_rendering(&self, cmd_buffer: &mut CmdBuffer);

    fn cmd_draw(
        &self,
        cmd_buffer: &mut CmdBuffer,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    );

    fn cmd_draw_indexed(
        &self,
        cmd_buffer: &mut CmdBuffer,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    );

    fn cmd_dispatch(
        &self,
        cmd_buffer: &mut CmdBuffer,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    );

    fn submit(&self, frame: &Frame, cmd_buffer: CmdBuffer);

    fn begin_frame(&self) -> Frame;

    fn end_frame<'device>(&'device self, frame: Frame<'device>);

    #[cfg(debug_assertions)]
    fn debug_allocator_dump_svg(&self) -> Result<(), std::io::Error>;
}
