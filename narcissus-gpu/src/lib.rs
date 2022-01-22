use std::{ffi::CStr, marker::PhantomData};

use narcissus_app::{App, Window};
use narcissus_core::{flags_def, thread_token_def, Handle, PhantomUnsend};

mod vulkan;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Texture(Handle);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Buffer(Handle);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sampler(Handle);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pipeline(Handle);

#[derive(Clone, Copy, Debug)]
pub enum MemoryLocation {
    Auto,
    PreferHost,
    PreferDevice,
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
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

flags_def!(ShaderStageFlags);
impl ShaderStageFlags {
    pub const VERTEX: Self = Self(1 << 0);
    pub const FRAGMENT: Self = Self(1 << 1);
    pub const COMPUTE: Self = Self(1 << 2);
    pub const ALL: Self = Self(0b111); /* Self::VERTEX | Self::FRAGMENT | Self::COMPUTE */
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureDimension {
    Type1d,
    Type2d,
    Type3d,
    TypeCube,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TextureFormat {
    BGRA8_SRGB,
    BGRA8_UNORM,
    RGBA8_SRGB,
    RGBA8_UNORM,
}

flags_def!(TextureUsageFlags);
impl TextureUsageFlags {
    pub const SAMPLED: Self = Self(1 << 0);
    pub const STORAGE: Self = Self(1 << 1);
    pub const DEPTH_STENCIL: Self = Self(1 << 2);
    pub const RENDER_TARGET: Self = Self(1 << 3);
    pub const TRANSFER_SRC: Self = Self(1 << 4);
    pub const TRANSFER_DST: Self = Self(1 << 5);
}

flags_def!(BufferUsageFlags);
impl BufferUsageFlags {
    pub const UNIFORM: Self = Self(1 << 0);
    pub const STORAGE: Self = Self(1 << 1);
    pub const INDEX: Self = Self(1 << 2);
    pub const TRANSFER_SRC: Self = Self(1 << 3);
    pub const TRANSFER_DST: Self = Self(1 << 4);
}

pub struct BufferDesc {
    pub memory_location: MemoryLocation,
    pub usage: BufferUsageFlags,
    pub size: usize,
}

pub struct TextureDesc {
    pub memory_location: MemoryLocation,
    pub usage: TextureUsageFlags,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub layers: u32,
    pub mip_levels: u32,
}

pub struct TextureViewDesc {
    pub texture: Texture,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub base_mip: u32,
    pub mip_count: u32,
    pub base_layer: u32,
    pub layer_count: u32,
}

pub struct ShaderDesc<'a> {
    pub entrypoint_name: &'a CStr,
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
    None,
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
    pub compare_op: SamplerCompareOp,
    pub mip_lod_bias: f32,
    pub min_lod: f32,
    pub max_lod: f32,
}

pub struct GraphicsPipelineLayout<'a> {
    pub color_attachment_formats: &'a [TextureFormat],
    pub depth_attachment_format: Option<TextureFormat>,
    pub stencil_attachment_format: Option<TextureFormat>,
}

pub struct GraphicsPipelineDesc<'a> {
    pub vertex_shader: ShaderDesc<'a>,
    pub fragment_shader: ShaderDesc<'a>,
    pub layout: GraphicsPipelineLayout<'a>,
}

pub struct ComputePipelineDesc<'a> {
    pub shader: ShaderDesc<'a>,
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
    pub texture: Texture,
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

thread_token_def!(ThreadToken, GpuConcurrent, 8);

pub struct FrameToken<'device> {
    device_address: usize,
    frame_index: usize,
    phantom: PhantomData<&'device dyn Device>,
}

pub struct CommandBufferToken<'frame, 'thread> {
    frame_token: &'frame FrameToken<'frame>,
    thread_token: &'thread mut ThreadToken,
    index: usize,
    raw: u64,
    phantom: PhantomUnsend,
}

pub trait Device {
    fn create_buffer(&self, buffer_desc: &BufferDesc) -> Buffer;
    fn create_texture(&self, texture_desc: &TextureDesc) -> Texture;
    fn create_texture_view(&self, desc: &TextureViewDesc) -> Texture;
    fn create_sampler(&self, desc: &SamplerDesc) -> Sampler;
    fn create_graphics_pipeline(&self, desc: &GraphicsPipelineDesc) -> Pipeline;
    fn create_compute_pipeline(&self, desc: &ComputePipelineDesc) -> Pipeline;

    fn destroy_buffer(&self, frame_token: &FrameToken, buffer: Buffer);
    fn destroy_texture(&self, frame_token: &FrameToken, texture: Texture);
    fn destroy_sampler(&self, frame_token: &FrameToken, sampler: Sampler);
    fn destroy_pipeline(&self, frame_token: &FrameToken, pipeline: Pipeline);

    fn acquire_swapchain(
        &self,
        frame_token: &FrameToken,
        window: Window,
        format: TextureFormat,
    ) -> (u32, u32, Texture);
    fn destroy_window(&self, window: Window);

    fn request_command_buffer<'frame>(
        &'frame self,
        frame_token: &'frame FrameToken,
        thread_token: &'frame mut ThreadToken,
    ) -> CommandBufferToken;

    fn cmd_bind_pipeline(&self, command_buffer_token: &mut CommandBufferToken, pipeline: Pipeline);
    fn cmd_begin_rendering(
        &self,
        command_buffer_token: &mut CommandBufferToken,
        desc: &RenderingDesc,
    );
    fn cmd_end_rendering(&self, command_buffer_token: &mut CommandBufferToken);
    fn cmd_set_viewports(
        &self,
        command_buffer_token: &mut CommandBufferToken,
        viewports: &[Viewport],
    );
    fn cmd_set_scissors(&self, command_buffer_token: &mut CommandBufferToken, scissors: &[Scissor]);
    fn cmd_draw(
        &self,
        command_buffer_token: &mut CommandBufferToken,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    );

    fn submit(&self, command_buffer_token: CommandBufferToken);

    fn begin_frame<'device>(&'device self) -> FrameToken<'device>;
    fn end_frame<'device>(&'device self, frame_token: FrameToken<'device>);
}

pub fn create_vulkan_device<'app>(app: &'app dyn App) -> Box<dyn Device + 'app> {
    Box::new(vulkan::VulkanDevice::new(app))
}
