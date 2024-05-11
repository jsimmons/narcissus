use std::{ffi::c_void, os::raw::c_char};

use super::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Offset2d {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Offset3d {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Extent2d {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Extent3d {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Rect2d {
    pub offset: Offset2d,
    pub extent: Extent2d,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ClearRect {
    pub rect: Rect2d,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ComponentMapping {
    pub r: ComponentSwizzle,
    pub g: ComponentSwizzle,
    pub b: ComponentSwizzle,
    pub a: ComponentSwizzle,
}

#[repr(C)]
pub struct ExtensionProperties {
    pub extension_name: [u8; 256],
    pub spec_version: u32,
}

#[repr(C)]
pub struct LayerProperties {
    pub layer_name: [u8; 256],
    pub spec_version: u32,
    pub implementation_version: u32,
    pub description: [u8; 256],
}

#[repr(C)]
pub struct AllocationCallbacks {
    user_data: *mut c_void,
    allocation: Option<FnAllocationFunction>,
    reallocation: Option<FnReallocationFunction>,
    free: Option<FnFreeFunction>,
    internal_allocation: Option<FnInternalAllocationNotification>,
    internal_free: Option<FnInternalFreeNotification>,
}

#[repr(C)]
pub struct DeviceQueueCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: DeviceQueueCreateFlags,
    pub queue_family_index: u32,
    pub queue_priorities: VulkanSlice1<'a, u32, f32, 4>,
}

impl<'a> Default for DeviceQueueCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::DeviceQueueCreateInfo;
        x
    }
}

#[repr(C)]
pub struct DeviceCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: DeviceCreateFlags,
    pub queue_create_infos: VulkanSlice1<'a, u32, DeviceQueueCreateInfo<'a>, 0>,
    pub enabled_layers: VulkanSlice1<'a, u32, *const c_char, 4>,
    pub enabled_extension_names: VulkanSlice1<'a, u32, *const c_char, 4>,
    pub enabled_features: *const PhysicalDeviceFeatures,
}

impl<'a> Default for DeviceCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::DeviceCreateInfo;
        x
    }
}

#[repr(C)]
pub struct ApplicationInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub application_name: *const c_char,
    pub application_version: u32,
    pub engine_name: *const c_char,
    pub engine_version: u32,
    pub api_version: u32,
}

impl Default for ApplicationInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::ApplicationInfo;
        x
    }
}

#[repr(C)]
pub struct InstanceCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: InstanceCreateFlags,
    pub application_info: Option<&'a ApplicationInfo>,
    pub enabled_layers: VulkanSlice1<'a, u32, *const c_char, 4>,
    pub enabled_extension_names: VulkanSlice1<'a, u32, *const c_char, 4>,
}

impl<'a> Default for InstanceCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::InstanceCreateInfo;
        x
    }
}

#[repr(C)]
pub struct XcbSurfaceCreateInfoKHR {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: XcbSurfaceCreateFlagsKHR,
    pub connection: *mut c_void, // xcb_connection_t*
    pub window: i32,             // xcb_window_t
}

impl Default for XcbSurfaceCreateInfoKHR {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::XcbSurfaceCreateInfoKHR;
        x
    }
}

#[repr(C)]
pub struct XlibSurfaceCreateInfoKHR {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: XlibSurfaceCreateFlagsKHR,
    pub display: *mut c_void, // Display*
    pub window: i32,          // Window
}

impl Default for XlibSurfaceCreateInfoKHR {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::XlibSurfaceCreateInfoKHR;
        x
    }
}

#[repr(C)]
pub struct WaylandSurfaceCreateInfoKHR {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: WaylandSurfaceCreateFlagsKHR,
    pub display: *mut c_void, // wl_display*
    pub surface: *mut c_void, // wl_surface*
}

impl Default for WaylandSurfaceCreateInfoKHR {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::WaylandSurfaceCreateInfoKHR;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct QueueFamilyProperties {
    pub queue_flags: QueueFlags,
    pub queue_count: u32,
    pub timestamp_valid_bits: u32,
    /// Minimum alignment requirement for image transfers
    pub min_image_transfer_granularity: Extent3d,
}

#[repr(C)]
pub struct PhysicalDeviceMemoryProperties {
    pub memory_type_count: u32,
    pub memory_types: [MemoryType; MAX_MEMORY_TYPES as usize],
    pub memory_heap_count: u32,
    pub memory_heaps: [MemoryHeap; MAX_MEMORY_HEAPS as usize],
}

impl Default for PhysicalDeviceMemoryProperties {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MemoryAllocateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub allocation_size: DeviceSize, // Size of memory allocation
    pub memory_type_index: u32,      // Index of the of the memory type to allocate from
}

impl Default for MemoryAllocateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::MemoryAllocateInfo;
        x
    }
}

#[repr(C)]
pub struct MemoryRequirements {
    pub size: DeviceSize,      // Specified in bytes
    pub alignment: DeviceSize, // Specified in bytes
    pub memory_type_bits: u32, // Bitmask of the allowed memory type indices into memoryTypes[] for this object
}

impl Default for MemoryRequirements {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[repr(C)]
pub struct MemoryRequirements2 {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub memory_requirements: MemoryRequirements,
}

impl Default for MemoryRequirements2 {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::MemoryRequirements2;
        x
    }
}

#[repr(C)]
pub struct ImageMemoryRequirementsInfo2 {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub image: Image,
}

impl Default for ImageMemoryRequirementsInfo2 {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::ImageMemoryRequirementsInfo2;
        x
    }
}

#[repr(C)]
pub struct BufferMemoryRequirementsInfo2 {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub buffer: Buffer,
}

impl Default for BufferMemoryRequirementsInfo2 {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::BufferMemoryRequirementsInfo2;
        x
    }
}

#[repr(C)]
pub struct BindImageMemoryInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub image: Image,
    pub memory: DeviceMemory,
    pub offset: DeviceSize,
}

impl Default for BindImageMemoryInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::BindImageMemoryInfo;
        x
    }
}

#[repr(C)]
pub struct BindBufferMemoryInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub buffer: Buffer,
    pub memory: DeviceMemory,
    pub offset: DeviceSize,
}

impl Default for BindBufferMemoryInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::BindBufferMemoryInfo;
        x
    }
}

#[repr(C)]
pub struct SparseImageFormatProperties {
    pub aspect_mask: ImageAspectFlags,
    pub image_granularity: Extent3d,
    pub flags: SparseImageFormatFlags,
}

#[repr(C)]
pub struct SparseImageMemoryRequirements {
    pub format_properties: SparseImageFormatProperties,
    pub image_mip_tail_first_lod: u32,
    pub image_mip_tail_size: DeviceSize, // Specified in bytes, must be a multiple of sparse block size in bytes / alignment
    pub image_mip_tail_offset: DeviceSize, // Specified in bytes, must be a multiple of sparse block size in bytes / alignment
    pub image_mip_tail_stride: DeviceSize, // Specified in bytes, must be a multiple of sparse block size in bytes / alignment
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MemoryType {
    pub property_flags: MemoryPropertyFlags, // Memory properties of this memory type
    pub heap_index: u32, // Index of the memory heap allocations of this memory type are taken from
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MemoryHeap {
    pub size: DeviceSize,       // Available memory in the heap
    pub flags: MemoryHeapFlags, // Flags for the heap
}

#[repr(C)]
pub struct MemoryDedicatedRequirements {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub prefers_dedicated_allocation: Bool32,
    pub requires_dedicated_allocation: Bool32,
}

impl Default for MemoryDedicatedRequirements {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::MemoryDedicatedRequirements;
        x
    }
}

#[repr(C)]
pub struct MemoryDedicatedAllocateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub image: Image,
    pub buffer: Buffer,
}

impl Default for MemoryDedicatedAllocateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::MemoryDedicatedAllocateInfo;
        x
    }
}

#[repr(C)]
pub struct SubmitInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub wait_semaphores: VulkanSlice2<'a, u32, Semaphore, PipelineStageFlags, 4>,
    pub command_buffers: VulkanSlice1<'a, u32, CommandBuffer, 4>,
    pub signal_semaphores: VulkanSlice1<'a, u32, Semaphore, 4>,
}

impl<'a> Default for SubmitInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SubmitInfo;
        x
    }
}

#[repr(C)]
pub struct SubmitInfo2<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: SubmitFlags,
    pub wait_semaphore_infos: VulkanSlice1<'a, u32, SemaphoreSubmitInfo, 0>,
    pub command_buffer_infos: VulkanSlice1<'a, u32, CommandBufferSubmitInfo, 4>,
    pub signal_semaphore_infos: VulkanSlice1<'a, u32, SemaphoreSubmitInfo, 4>,
}

impl<'a> Default for SubmitInfo2<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SubmitInfo2;
        x
    }
}

#[repr(C)]
pub struct SemaphoreSubmitInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub semaphore: Semaphore,
    pub semaphore_value: u64,
    pub stage_mask: PipelineStageFlags2,
    pub device_index: u32,
}

impl Default for SemaphoreSubmitInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SemaphoreSubmitInfo;
        x
    }
}

#[repr(C)]
pub struct CommandBufferSubmitInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub command_buffer: CommandBuffer,
    pub device_mask: u32,
}

impl Default for CommandBufferSubmitInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::CommandBufferSubmitInfo;
        x
    }
}

#[repr(C)]
pub struct MappedMemoryRange {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub memory: DeviceMemory, // Mapped memory object
    pub offset: DeviceSize,   // Offset within the memory object where the range starts
    pub size: DeviceSize,     // Size of the range within the memory object
}

impl Default for MappedMemoryRange {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::MappedMemoryRange;
        x
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct FormatProperties {
    pub linear_tiling_features: FormatFeatureFlags, // Format features in case of linear tiling
    pub optimal_tiling_features: FormatFeatureFlags, // Format features in case of optimal tiling
    pub buffer_features: FormatFeatureFlags,        // Format features supported by buffers
}

#[repr(C)]
#[derive(Clone)]
pub struct ImageFormatProperties {
    pub max_extent: Extent3d,  // max image dimensions for this resource type
    pub max_mip_levels: u32,   // max number of mipmap levels for this resource type
    pub max_array_layers: u32, // max array size for this resource type
    pub sample_counts: SampleCountFlags, // supported sample counts for this resource type
    pub max_resource_size: DeviceSize, // max size (in bytes) of this resource type
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SurfaceCapabilitiesKHR {
    pub min_image_count: u32, // Supported minimum number of images for the surface
    pub max_image_count: u32, // Supported maximum number of images for the surface, 0 for unlimited
    pub current_extent: Extent2d, // Current image width and height for the surface, (0, 0) if undefined
    pub min_image_extent: Extent2d, // Supported minimum image width and height for the surface
    pub max_image_extent: Extent2d, // Supported maximum image width and height for the surface
    pub max_image_array_layers: u32, // Supported maximum number of image layers for the surface
    pub supported_transforms: SurfaceTransformFlagsKHR, // 1 or more bits representing the transforms supported
    pub current_transform: SurfaceTransformFlagsKHR, // The surface's current transform relative to the device's natural orientation
    pub supported_composite_alpha: CompositeAlphaFlagsKHR, // 1 or more bits representing the alpha compositing modes supported
    pub supported_usage_flags: ImageUsageFlags, // Supported image usage flags for the surface
}

impl Default for SurfaceCapabilitiesKHR {
    fn default() -> Self {
        unsafe { MaybeUninit::<Self>::zeroed().assume_init() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SurfaceFormatKHR {
    pub format: Format,             // Supported pair of rendering format
    pub color_space: ColorSpaceKHR, // and color space for the surface
}

impl Default for SurfaceFormatKHR {
    fn default() -> Self {
        Self {
            format: Format::Undefined,
            color_space: ColorSpaceKHR::SrgbNonlinearKhr,
        }
    }
}

#[repr(C)]
pub struct SurfacePresentModeEXT {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub present_mode: PresentModeKHR,
}

impl Default for SurfacePresentModeEXT {
    fn default() -> Self {
        Self {
            _type: StructureType::SurfacePresentModeExt,
            _next: core::ptr::null(),
            present_mode: PresentModeKHR::Fifo,
        }
    }
}

#[repr(C)]
pub struct SwapchainPresentModesCreateInfoEXT<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub present_modes: VulkanSlice1<'a, u32, PresentModeKHR, 4>,
}

impl<'a> Default for SwapchainPresentModesCreateInfoEXT<'a> {
    fn default() -> Self {
        Self {
            _type: StructureType::SwapchainPresentModesCreateInfoExt,
            _next: core::ptr::null(),
            present_modes: Default::default(),
        }
    }
}

#[repr(C)]
pub struct SurfacePresentModeCompatibilityEXT<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub present_modes: VulkanSlice1<'a, u32, PresentModeKHR, 4>,
}

impl<'a> Default for SurfacePresentModeCompatibilityEXT<'a> {
    fn default() -> Self {
        Self {
            _type: StructureType::SurfacePresentModeCompatibilityExt,
            _next: core::ptr::null(),
            present_modes: Default::default(),
        }
    }
}

#[repr(C)]
pub struct SurfacePresentScalingCapabilitiesEXT {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub supported_present_scaling: PresentScalingFlagsEXT,
    pub supported_present_gravity_x: PresentGravityFlagsEXT,
    pub supported_present_gravity_y: PresentGravityFlagsEXT,
    pub min_scaled_image_extent: Extent2d,
    pub max_scaled_image_extent: Extent2d,
}

impl Default for SurfacePresentScalingCapabilitiesEXT {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SurfacePresentScalingCapabilitiesExt;
        x
    }
}

#[repr(C)]
pub struct SwapchainPresentScalingCreateInfoEXT {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub scaling_behavior: PresentScalingFlagsEXT,
    pub present_gravity_x: PresentGravityFlagsEXT,
    pub present_gravity_y: PresentGravityFlagsEXT,
}

impl Default for SwapchainPresentScalingCreateInfoEXT {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SwapchainPresentScalingCreateInfoExt;
        x
    }
}

#[repr(C)]
pub struct SwapchainPresentFenceInfoEXT<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub fences: VulkanSlice1<'a, u32, Fence, 4>,
}

impl<'a> Default for SwapchainPresentFenceInfoEXT<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SwapchainPresentFenceInfoExt;
        x
    }
}

#[repr(C)]
pub struct ReleaseSwapchainImagesInfoEXT<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub swapchain: SwapchainKHR,
    pub image_indices: VulkanSlice1<'a, u32, u32, 4>,
}

impl<'a> Default for ReleaseSwapchainImagesInfoEXT<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::ReleaseSwapchainImagesInfoExt;
        x
    }
}

#[repr(C)]
pub struct SwapchainCreateInfoKHR<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: SwapchainCreateFlagsKHR,
    ///  The swapchain's target surface
    pub surface: SurfaceKHR,
    ///  Minimum number of presentation images the application needs
    pub min_image_count: u32,
    ///  Format of the presentation images
    pub image_format: Format,
    ///  Colorspace of the presentation images
    pub image_color_space: ColorSpaceKHR,
    ///  Dimensions of the presentation images
    pub image_extent: Extent2d,
    ///  Determines the number of views for multiview/stereo presentation
    pub image_array_layers: u32,
    ///  Bits indicating how the presentation images will be used
    pub image_usage: ImageUsageFlags,
    ///  Sharing mode used for the presentation images
    pub image_sharing_mode: SharingMode,
    ///  Array of queue family indices having access to the images in case of concurrent sharing mode
    pub queue_family_indices: VulkanSlice1<'a, u32, u32, 4>,
    ///  The transform, relative to the device's natural orientation, applied to the image content prior to presentation
    pub pre_transform: SurfaceTransformFlagsKHR,
    ///  The alpha blending mode used when compositing this surface with other surfaces in the window system
    pub composite_alpha: CompositeAlphaFlagsKHR,
    ///  Which presentation mode to use for presents on this swap chain
    pub present_mode: PresentModeKHR,
    ///  Specifies whether presentable images may be affected by window clip regions
    pub clipped: Bool32,
    ///  Existing swap chain to replace, if any
    pub old_swapchain: SwapchainKHR,
}

impl<'a> Default for SwapchainCreateInfoKHR<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SwapchainCreateInfoKhr;
        x
    }
}

#[repr(C)]
pub struct PresentInfoKHR<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    /// Semaphores to wait for before presenting
    pub wait_semaphores: VulkanSlice1<'a, u32, Semaphore, 4>,
    /// Swapchains and swapchain image indices to present
    pub swapchains: VulkanSlice2<'a, u32, SwapchainKHR, u32, 4>,
    /// Optional (i.e. if non-NULL) VkResult for each swapchain
    pub results: *mut Result,
}

impl<'a> Default for PresentInfoKHR<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PresentInfoKhr;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AcquireNextImageInfoKHR {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub swapchain: SwapchainKHR,
    pub timeout: u64,
    pub semaphore: Semaphore,
    pub fence: Fence,
    pub device_mask: u32,
}

impl Default for AcquireNextImageInfoKHR {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::AcquireNextImageInfoKhr;
        x
    }
}

#[repr(C)]
pub struct ImageCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: ImageCreateFlags, // Image creation flags
    pub image_type: ImageType,
    pub format: Format,
    pub extent: Extent3d,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: SampleCountFlags,
    pub tiling: ImageTiling,
    /// Image usage flags
    pub usage: ImageUsageFlags,
    /// Cross-queue-family sharing mode
    pub sharing_mode: SharingMode,
    /// Array of queue family indices to share across
    pub queue_family_indices: VulkanSlice1<'a, u32, u32, 4>,
    /// Initial image layout for all subresources
    pub initial_layout: ImageLayout,
}

impl<'a> Default for ImageCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::ImageCreateInfo;
        x
    }
}

#[repr(C)]
pub struct SubresourceLayout {
    pub offset: DeviceSize,
    pub size: DeviceSize,
    pub row_pitch: DeviceSize,
    pub array_pitch: DeviceSize,
    pub depth_pitch: DeviceSize,
}

impl Default for SubresourceLayout {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[repr(C)]
pub struct ImageViewCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: ImageViewCreateFlags,
    pub image: Image,
    pub view_type: ImageViewType,
    pub format: Format,
    pub components: ComponentMapping,
    pub subresource_range: ImageSubresourceRange,
}

impl Default for ImageViewCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::ImageViewCreateInfo;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CommandPoolCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: CommandPoolCreateFlags,
    pub queue_family_index: u32,
}

impl Default for CommandPoolCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::CommandPoolCreateInfo;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CommandBufferAllocateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub command_pool: CommandPool,
    pub level: CommandBufferLevel,
    pub command_buffer_count: u32,
}

impl Default for CommandBufferAllocateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::CommandBufferAllocateInfo;
        x
    }
}

#[repr(C)]
pub struct CommandBufferInheritanceInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub render_pass: RenderPass, // Render pass for secondary command buffers
    pub subpass: u32,
    pub framebuffer: Framebuffer, // Framebuffer for secondary command buffers
    pub occlusion_query_enable: Bool32, // Whether this secondary command buffer may be executed during an occlusion query
    pub query_flags: QueryControlFlags, // Query flags used by this secondary command buffer, if executed during an occlusion query
    pub pipeline_statistics: QueryPipelineStatisticFlags, // Pipeline statistics that may be counted for this secondary command buffer
}

impl Default for CommandBufferInheritanceInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::CommandBufferInheritanceInfo;
        x
    }
}

#[repr(C)]
pub struct CommandBufferBeginInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: CommandBufferUsageFlags,
    /// Inheritance info for secondary command buffers
    pub inheritance_info: Option<&'a CommandBufferInheritanceInfo>,
}

impl<'a> Default for CommandBufferBeginInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::CommandBufferBeginInfo;
        x
    }
}

#[repr(C)]
pub struct RenderingAttachmentInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub image_view: ImageView,
    pub image_layout: ImageLayout,
    pub resolve_mode: ResolveModeFlags,
    pub resolve_image_view: ImageView,
    pub resolve_image_layout: ImageLayout,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub clear_value: ClearValue,
}

impl Default for RenderingAttachmentInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::RenderingAttachmentInfo;
        x
    }
}

#[repr(C)]
pub struct RenderingInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: RenderingFlags,
    pub render_area: Rect2d,
    pub layer_count: u32,
    pub view_mask: u32,
    pub color_attachments: VulkanSlice1<'a, u32, RenderingAttachmentInfo, 0>,
    pub depth_attachment: Option<&'a RenderingAttachmentInfo>,
    pub stencil_attachment: Option<&'a RenderingAttachmentInfo>,
}

impl<'a> Default for RenderingInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::RenderingInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineRenderingCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub view_mask: u32,
    pub color_attachment_formats: VulkanSlice1<'a, u32, Format, 0>,
    pub depth_attachment_format: Format,
    pub stencil_attachment_format: Format,
}

impl<'a> Default for PipelineRenderingCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineRenderingCreateInfo;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BufferCopy {
    /// Specified in bytes
    pub src_offset: DeviceSize,
    /// Specified in bytes
    pub dst_offset: DeviceSize,
    /// Specified in bytes
    pub size: DeviceSize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImageSubresourceLayers {
    pub aspect_mask: ImageAspectFlags,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImageCopy {
    pub src_subresource: ImageSubresourceLayers,
    /// Specified in pixels for both compressed and uncompressed images
    pub src_offset: Offset3d,
    pub dst_subresource: ImageSubresourceLayers,
    /// Specified in pixels for both compressed and uncompressed images
    pub dst_offset: Offset3d,
    /// Specified in pixels for both compressed and uncompressed images
    pub extent: Extent3d,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImageBlit {
    pub src_subresource: ImageSubresourceLayers,
    /// Specified in pixels for both compressed and uncompressed images
    pub src_offsets: [Offset3d; 2],
    pub dst_subresource: ImageSubresourceLayers,
    /// Specified in pixels for both compressed and uncompressed images
    pub dst_offsets: [Offset3d; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BufferImageCopy {
    /// Specified in bytes
    pub buffer_offset: DeviceSize,
    /// Specified in texels
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: ImageSubresourceLayers,
    /// Specified in pixels for both compressed and uncompressed images
    pub image_offset: Offset3d,
    /// Specified in pixels for both compressed and uncompressed images
    pub image_extent: Extent3d,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImageResolve {
    pub src_subresource: ImageSubresourceLayers,
    pub src_offset: Offset3d,
    pub dst_subresource: ImageSubresourceLayers,
    pub dst_offset: Offset3d,
    pub extent: Extent3d,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ClearColorValue {
    pub f32: [f32; 4],
    pub i32: [i32; 4],
    pub u32: [u32; 4],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ClearDepthStencilValue {
    pub depth: f32,
    pub stencil: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
/// Union allowing specification of color or depth and stencil values. Actual value selected is based on attachment being cleared.
pub union ClearValue {
    pub color: ClearColorValue,
    pub depth_stencil: ClearDepthStencilValue,
}

impl Default for ClearValue {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[repr(C)]
pub struct RenderPassBeginInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub render_pass: RenderPass,
    pub framebuffer: Framebuffer,
    pub render_area: Rect2d,
    pub clear_values: VulkanSlice1<'a, u32, ClearValue, 4>,
}

impl<'a> Default for RenderPassBeginInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::RenderPassBeginInfo;
        x
    }
}

#[repr(C)]
pub struct MemoryBarrier2 {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub src_stage_mask: PipelineStageFlags2,
    pub src_access_mask: AccessFlags2,
    pub dst_stage_mask: PipelineStageFlags2,
    pub dst_access_mask: AccessFlags2,
}

impl Default for MemoryBarrier2 {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::MemoryBarrier2;
        x
    }
}

#[repr(C)]
pub struct BufferMemoryBarrier2 {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub src_stage_mask: PipelineStageFlags2,
    pub src_access_mask: AccessFlags2,
    pub dst_stage_mask: PipelineStageFlags2,
    pub dst_access_mask: AccessFlags2,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub buffer: Buffer,
    pub offset: DeviceSize,
    pub size: DeviceSize,
}

impl Default for BufferMemoryBarrier2 {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::BufferMemoryBarrier2;
        x
    }
}

#[repr(C)]
pub struct ImageMemoryBarrier2 {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub src_stage_mask: PipelineStageFlags2,
    pub src_access_mask: AccessFlags2,
    pub dst_stage_mask: PipelineStageFlags2,
    pub dst_access_mask: AccessFlags2,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub image: Image,
    pub subresource_range: ImageSubresourceRange,
}

impl Default for ImageMemoryBarrier2 {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::ImageMemoryBarrier2;
        x
    }
}

#[repr(C)]
pub struct DependencyInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: DependencyFlags,
    pub memory_barriers: VulkanSlice1<'a, u32, MemoryBarrier2, 0>,
    pub buffer_memory_barriers: VulkanSlice1<'a, u32, BufferMemoryBarrier2, 4>,
    pub image_memory_barriers: VulkanSlice1<'a, u32, ImageMemoryBarrier2, 4>,
}

impl<'a> Default for DependencyInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::DependencyInfo;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ClearAttachment {
    pub aspect_mask: ImageAspectFlags,
    pub color_attachment: u32,
    pub clear_value: ClearValue,
}

#[repr(C)]
pub struct AttachmentDescription {
    pub flags: AttachmentDescriptionFlags,
    pub format: Format,
    pub samples: SampleCountFlags,
    pub load_op: AttachmentLoadOp, // Load operation for color or depth data
    pub store_op: AttachmentStoreOp, // Store operation for color or depth data
    pub stencil_load_op: AttachmentLoadOp, // Load operation for stencil data
    pub stencil_store_op: AttachmentStoreOp, // Store operation for stencil data
    pub initial_layout: ImageLayout,
    pub final_layout: ImageLayout,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AttachmentReference {
    pub attachment: u32,
    pub layout: ImageLayout,
}

#[repr(C)]
pub struct SubpassDescription<'a> {
    pub flags: SubpassDescriptionFlags,
    /// Must be VK_PIPELINE_BIND_POINT_GRAPHICS for now
    pub pipeline_bind_point: PipelineBindPoint,
    pub input_attachments: VulkanSlice1<'a, u32, AttachmentReference, 4>,
    pub color_attachments: VulkanSlice1<'a, u32, AttachmentReference, 4>,
    pub resolve_attachments: *const AttachmentReference,
    pub depth_stencil_attachment: Option<&'a AttachmentReference>,
    pub preserve_attachments: VulkanSlice1<'a, u32, u32, 4>,
}

impl<'a> Default for SubpassDescription<'a> {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[repr(C)]
pub struct SubpassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage_mask: PipelineStageFlags,
    pub dst_stage_mask: PipelineStageFlags,
    /// Memory accesses from the source of the dependency to synchronize
    pub src_access_mask: AccessFlags,
    /// Memory accesses from the destination of the dependency to synchronize
    pub dst_access_mask: AccessFlags,
    pub dependency_flags: DependencyFlags,
}

#[repr(C)]
pub struct RenderPassCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: RenderPassCreateFlags,
    pub attachments: VulkanSlice1<'a, u32, AttachmentDescription, 0>,
    pub subpasses: VulkanSlice1<'a, u32, SubpassDescription<'a>, 4>,
    pub dependencies: VulkanSlice1<'a, u32, SubpassDependency, 4>,
}

impl<'a> Default for RenderPassCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::RenderPassCreateInfo;
        x
    }
}

#[repr(C)]
pub struct ShaderModuleCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: ShaderModuleCreateFlags,
    pub _pad: [u8; 4],
    pub code: VulkanSlice1<'a, usize, u8, 0>,
}

impl<'a> Default for ShaderModuleCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::ShaderModuleCreateInfo;
        x
    }
}

#[repr(C)]
pub struct DescriptorSetLayoutBinding {
    ///  Binding number for this entry
    pub binding: u32,
    ///  Type of the descriptors in this binding
    pub descriptor_type: DescriptorType,
    ///  Number of descriptors in this binding
    pub descriptor_count: u32,
    ///  Shader stages this binding is visible to
    pub stage_flags: ShaderStageFlags,
    ///  Immutable samplers (used if descriptor type is SAMPLER or COMBINED_IMAGE_SAMPLER, is either NULL or contains count number of elements)
    pub immutable_samplers: *const Sampler,
}

#[repr(C)]
pub struct DescriptorSetLayoutCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: DescriptorSetLayoutCreateFlags,
    ///  Array of descriptor set layout bindings
    pub bindings: VulkanSlice1<'a, u32, DescriptorSetLayoutBinding, 0>,
}

impl<'a> Default for DescriptorSetLayoutCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::DescriptorSetLayoutCreateInfo;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DescriptorPoolSize {
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
}

#[repr(C)]
pub struct DescriptorPoolCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: DescriptorPoolCreateFlags,
    pub max_sets: u32,
    pub pool_sizes: VulkanSlice1<'a, u32, DescriptorPoolSize, 4>,
}

impl<'a> Default for DescriptorPoolCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::DescriptorPoolCreateInfo;
        x
    }
}

#[repr(C)]
pub struct DescriptorSetAllocateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub descriptor_pool: DescriptorPool,
    pub set_layouts: VulkanSlice1<'a, u32, DescriptorSetLayout, 4>,
}

impl<'a> Default for DescriptorSetAllocateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::DescriptorSetAllocateInfo;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SpecializationMapEntry {
    ///  The SpecConstant ID specified in the BIL
    pub constant_id: u32,
    ///  Offset of the value in the data block
    pub offset: u32,
    ///  Size in bytes of the SpecConstant
    pub size: usize,
}

#[repr(C)]
pub struct SpecializationInfo<'a> {
    pub map_entries: VulkanSlice1<'a, u32, SpecializationMapEntry, 4>,
    ///  Size in bytes of pData
    pub data_size: usize,
    ///  Pointer to SpecConstant data
    pub data: *const c_void,
}

impl<'a> Default for SpecializationInfo<'a> {
    fn default() -> Self {
        unsafe { MaybeUninit::<Self>::zeroed().assume_init() }
    }
}

#[repr(C)]
pub struct PipelineShaderStageCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineShaderStageCreateFlags,
    ///  Shader stage
    pub stage: ShaderStageFlags,
    ///  Module containing entry point
    pub module: ShaderModule,
    ///  Null-terminated entry point name
    pub name: *const c_char,
    pub specialization_info: Option<&'a SpecializationInfo<'a>>,
}

impl<'a> Default for PipelineShaderStageCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineShaderStageCreateInfo;
        x
    }
}

#[repr(C)]
pub struct ComputePipelineCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineCreateFlags,
    pub stage: PipelineShaderStageCreateInfo<'a>,
    ///  Interface layout of the pipeline
    pub layout: PipelineLayout,
    ///  If VK_PIPELINE_CREATE_DERIVATIVE_BIT is set and this value is nonzero, it specifies the handle of the base pipeline this is a derivative of
    pub base_pipeline_handle: Pipeline,
    ///  If VK_PIPELINE_CREATE_DERIVATIVE_BIT is set and this value is not -1, it specifies an index into pCreateInfos of the base pipeline this is a derivative of
    pub base_pipeline_index: i32,
}

impl<'a> Default for ComputePipelineCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::ComputePipelineCreateInfo;
        x
    }
}

#[repr(C)]
pub struct VertexInputBindingDescription {
    ///  Vertex buffer binding id
    pub binding: u32,
    ///  Distance between vertices in bytes (0 = no advancement)
    pub stride: u32,
    ///  The rate at which the vertex data is consumed
    pub input_rate: VertexInputRate,
}

#[repr(C)]
pub struct VertexInputAttributeDescription {
    ///  location of the shader vertex attrib
    pub location: u32,
    ///  Vertex buffer binding id
    pub binding: u32,
    ///  format of source data
    pub format: Format,
    ///  Offset of first element in bytes from base of vertex
    pub offset: u32,
}

#[repr(C)]
pub struct PipelineVertexInputStateCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineVertexInputStateCreateFlags,
    pub vertex_binding_descriptions: VulkanSlice1<'a, u32, VertexInputBindingDescription, 0>,
    pub vertex_attribute_descriptions: VulkanSlice1<'a, u32, VertexInputAttributeDescription, 4>,
}

impl<'a> Default for PipelineVertexInputStateCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineVertexInputStateCreateInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineInputAssemblyStateCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineInputAssemblyStateCreateFlags,
    pub topology: PrimitiveTopology,
    pub primitive_restart_enable: Bool32,
}

impl Default for PipelineInputAssemblyStateCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineInputAssemblyStateCreateInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineTessellationStateCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineTessellationStateCreateFlags,
    pub patch_control_points: u32,
}

impl PipelineTessellationStateCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineTessellationStateCreateInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineViewportStateCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineViewportStateCreateFlags,
    pub viewports: VulkanSlice1<'a, u32, Viewport, 0>,
    pub scissors: VulkanSlice1<'a, u32, Rect2d, 4>,
}

impl<'a> Default for PipelineViewportStateCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineViewportStateCreateInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineRasterizationStateCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineRasterizationStateCreateFlags,
    pub depth_clamp_enable: Bool32,
    pub rasterizer_discard_enable: Bool32,
    pub polygon_mode: PolygonMode,
    pub cull_mode: CullModeFlags,
    pub front_face: FrontFace,
    pub depth_bias_enable: Bool32,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
    pub line_width: f32,
}

impl Default for PipelineRasterizationStateCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineRasterizationStateCreateInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineMultisampleStateCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineMultisampleStateCreateFlags,
    pub rasterization_samples: SampleCountFlags,
    pub sample_shading_enable: Bool32,
    pub min_sample_shading: f32,
    pub sample_mask: *const SampleMask,
    pub alpha_to_coverage_enable: Bool32,
    pub alpha_to_one_enable: Bool32,
}

impl Default for PipelineMultisampleStateCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineMultisampleStateCreateInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineColorBlendAttachmentState {
    pub blend_enable: Bool32,
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op: BlendOp,
    pub color_write_mask: ColorComponentFlags,
}

impl Default for PipelineColorBlendAttachmentState {
    fn default() -> Self {
        unsafe { MaybeUninit::<Self>::zeroed().assume_init() }
    }
}

#[repr(C)]
pub struct PipelineColorBlendStateCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineColorBlendStateCreateFlags,
    pub logic_op_enable: Bool32,
    pub logic_op: LogicOp,
    pub attachments: VulkanSlice1<'a, u32, PipelineColorBlendAttachmentState, 0>,
    pub blend_constants: [f32; 4],
}

impl<'a> Default for PipelineColorBlendStateCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineColorBlendStateCreateInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineDynamicStateCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineDynamicStateCreateFlags,
    pub dynamic_states: VulkanSlice1<'a, u32, DynamicState, 0>,
}

impl<'a> Default for PipelineDynamicStateCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineDynamicStateCreateInfo;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StencilOpState {
    pub fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub compare_op: CompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

#[repr(C)]
pub struct PipelineDepthStencilStateCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineDepthStencilStateCreateFlags,
    pub depth_test_enable: Bool32,
    pub depth_write_enable: Bool32,
    pub depth_compare_op: CompareOp,
    pub depth_bounds_test_enable: Bool32, // optional (depth_bounds_test)
    pub stencil_test_enable: Bool32,
    pub front: StencilOpState,
    pub back: StencilOpState,
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}

impl Default for PipelineDepthStencilStateCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineDepthStencilStateCreateInfo;
        x
    }
}

#[repr(C)]
pub struct GraphicsPipelineCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineCreateFlags,
    pub stages: VulkanSlice1<'a, u32, PipelineShaderStageCreateInfo<'a>, 0>,
    pub vertex_input_state: Option<&'a PipelineVertexInputStateCreateInfo<'a>>,
    pub input_assembly_state: Option<&'a PipelineInputAssemblyStateCreateInfo>,
    pub tessellation_state: Option<&'a PipelineTessellationStateCreateInfo>,
    pub viewport_state: Option<&'a PipelineViewportStateCreateInfo<'a>>,
    pub rasterization_state: Option<&'a PipelineRasterizationStateCreateInfo>,
    pub multisample_state: Option<&'a PipelineMultisampleStateCreateInfo>,
    pub depth_stencil_state: Option<&'a PipelineDepthStencilStateCreateInfo>,
    pub color_blend_state: Option<&'a PipelineColorBlendStateCreateInfo<'a>>,
    pub dynamic_state: Option<&'a PipelineDynamicStateCreateInfo<'a>>,
    ///  Interface layout of the pipeline
    pub layout: PipelineLayout,
    pub render_pass: RenderPass,
    pub subpass: u32,
    ///  If VK_PIPELINE_CREATE_DERIVATIVE_BIT is set and this value is nonzero, it specifies the handle of the base pipeline this is a derivative of
    pub base_pipeline_handle: Pipeline,
    ///  If VK_PIPELINE_CREATE_DERIVATIVE_BIT is set and this value is not -1, it specifies an index into pCreateInfos of the base pipeline this is a derivative of
    pub base_pipeline_index: i32,
}

impl<'a> Default for GraphicsPipelineCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::GraphicsPipelineCreateInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineCacheCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineCacheCreateFlags,
    ///  Size of initial data to populate cache, in bytes
    pub initial_data_size: usize,
    ///  Initial data to populate cache
    pub initial_data: *const c_void,
}

impl Default for PipelineCacheCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineCacheCreateInfo;
        x
    }
}

#[repr(C)]
pub struct PipelineCacheHeaderVersionOne {
    // The fields in this structure are non-normative since structure packing is implementation-defined in C. The specification defines the normative layout.
    pub header_size: u32,
    pub header_version: PipelineCacheHeaderVersion,
    pub vendor_id: u32,
    pub device_id: u32,
    pub pipeline_cache_uuid: [u8; UUID_SIZE as usize],
}

#[repr(C)]
pub struct PushConstantRange {
    ///  Which stages use the range
    pub stage_flags: ShaderStageFlags,
    ///  Start of the range, in bytes
    pub offset: u32,
    ///  Size of the range, in bytes
    pub size: u32,
}

#[repr(C)]
pub struct PipelineLayoutCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: PipelineLayoutCreateFlags,
    pub set_layouts: VulkanSlice1<'a, u32, DescriptorSetLayout, 0>,
    pub push_constant_ranges: VulkanSlice1<'a, u32, DescriptorSetLayout, 4>,
}

impl<'a> Default for PipelineLayoutCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PipelineLayoutCreateInfo;
        x
    }
}

#[repr(C)]
pub struct SamplerCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: SamplerCreateFlags,
    pub mag_filter: Filter,
    pub min_filter: Filter,
    pub mipmap_mode: SamplerMipmapMode,
    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub anisotropy_enable: Bool32,
    pub max_anisotropy: f32,
    pub compare_enable: Bool32,
    pub compare_op: CompareOp,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: BorderColor,
    pub unnormalized_coordinates: Bool32,
}

impl Default for SamplerCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SamplerCreateInfo;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DescriptorBufferInfo {
    /// Buffer used for this descriptor slot.
    pub buffer: Buffer,
    /// Base offset from buffer start in bytes to update in the descriptor set.
    pub offset: DeviceSize,
    /// Size in bytes of the buffer resource for this descriptor update.
    pub range: DeviceSize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DescriptorImageInfo {
    /// Sampler to write to the descriptor in case it is a SAMPLER or COMBINED_IMAGE_SAMPLER descriptor. Ignored otherwise.
    pub sampler: Sampler,
    /// Image view to write to the descriptor in case it is a SAMPLED_IMAGE, STORAGE_IMAGE, COMBINED_IMAGE_SAMPLER, or INPUT_ATTACHMENT descriptor. Ignored otherwise.
    pub image_view: ImageView,
    /// Layout the image is expected to be in when accessed using this descriptor (only used if imageView is not VK_NULL_HANDLE).
    pub image_layout: ImageLayout,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WriteDescriptorSet {
    pub _type: StructureType,
    pub _next: *const c_void,
    /// Destination descriptor set
    pub dst_set: DescriptorSet,
    /// Binding within the destination descriptor set to write
    pub dst_binding: u32,
    /// Array element within the destination binding to write
    pub dst_array_element: u32,
    /// Number of descriptors to write (determines the size of the array pointed by pDescriptors)
    pub descriptor_count: u32,
    /// Descriptor type to write (determines which members of the array pointed by pDescriptors are going to be used)
    pub descriptor_type: DescriptorType,
    /// Sampler, image view, and layout for SAMPLER, COMBINED_IMAGE_SAMPLER, {SAMPLED,STORAGE}_IMAGE, and INPUT_ATTACHMENT descriptor types.
    pub image_info: *const DescriptorImageInfo,
    /// Raw buffer, size, and offset for {UNIFORM,STORAGE}_BUFFER\[_DYNAMIC\] descriptor types.
    pub buffer_info: *const DescriptorBufferInfo,
    /// Buffer view to write to the descriptor for {UNIFORM,STORAGE}_TEXEL_BUFFER descriptor types.
    pub texel_buffer_view: *const BufferView,
}

impl Default for WriteDescriptorSet {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::WriteDescriptorSet;
        x
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CopyDescriptorSet {
    pub _type: StructureType,
    pub _next: *const c_void,
    /// Source descriptor set
    pub src_set: DescriptorSet,
    /// Binding within the source descriptor set to copy from
    pub src_binding: u32,
    /// Array element within the source binding to copy from
    pub src_array_element: u32,
    /// Destination descriptor set
    pub dst_set: DescriptorSet,
    /// Binding within the destination descriptor set to copy to
    pub dst_binding: u32,
    /// Array element within the destination binding to copy to
    pub dst_array_element: u32,
    /// Number of descriptors to write (determines the size of the array pointed by pDescriptors)
    pub descriptor_count: u32,
}

impl Default for CopyDescriptorSet {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::CopyDescriptorSet;
        x
    }
}

#[repr(C)]
pub struct BufferCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    /// Buffer creation flags
    pub flags: BufferCreateFlags,
    /// Specified in bytes
    pub size: DeviceSize,
    /// Buffer usage flags
    pub usage: BufferUsageFlags,
    pub sharing_mode: SharingMode,
    pub queue_family_indices: VulkanSlice1<'a, u32, u32, 4>,
}

impl<'a> Default for BufferCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::BufferCreateInfo;
        x
    }
}

#[repr(C)]
pub struct BufferViewCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: BufferViewCreateFlags,
    pub buffer: Buffer,
    /// Optionally specifies format of elements
    pub format: Format,
    /// Specified in bytes
    pub offset: DeviceSize,
    /// View size specified in bytes
    pub range: DeviceSize,
}

impl Default for BufferViewCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::BufferViewCreateInfo;
        x
    }
}

#[repr(C)]
pub struct ImageSubresource {
    pub aspect_mask: ImageAspectFlags,
    pub mip_level: u32,
    pub array_layer: u32,
}

#[repr(C)]
pub struct ImageSubresourceRange {
    pub aspect_mask: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl Default for ImageSubresourceRange {
    fn default() -> Self {
        Self {
            aspect_mask: Default::default(),
            base_mip_level: Default::default(),
            level_count: Default::default(),
            base_array_layer: Default::default(),
            layer_count: Default::default(),
        }
    }
}

#[repr(C)]
pub struct FramebufferCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: FramebufferCreateFlags,
    pub render_pass: RenderPass,
    pub attachments: VulkanSlice1<'a, u32, ImageView, 4>,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}

impl<'a> Default for FramebufferCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::FramebufferCreateInfo;
        x
    }
}

#[repr(C)]
pub struct FramebufferAttachmentImageInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    flags: ImageCreateFlags,
    usage: ImageUsageFlags,
    width: u32,
    height: u32,
    layer_count: u32,
    view_formats: VulkanSlice1<'a, u32, Format, 0>,
}

impl<'a> Default for FramebufferAttachmentImageInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::FramebufferAttachmentImageInfo;
        x
    }
}

#[repr(C)]
pub struct FramebufferAttachmentsCreateInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    attachment_image_infos: VulkanSlice1<'a, u32, FramebufferAttachmentImageInfo<'a>, 4>,
}

impl<'a> Default for FramebufferAttachmentsCreateInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::FramebufferAttachmentsCreateInfo;
        x
    }
}

#[repr(C)]
pub struct FenceCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: FenceCreateFlags,
}

impl Default for FenceCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::FenceCreateInfo;
        x
    }
}

#[repr(C)]
pub struct SemaphoreCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: SemaphoreCreateFlags, // Semaphore creation flags
}

impl Default for SemaphoreCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SemaphoreCreateInfo;
        x
    }
}

#[repr(C)]
pub struct SemaphoreTypeCreateInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub semaphore_type: SemaphoreType,
    pub initial_value: u64,
}

impl Default for SemaphoreTypeCreateInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SemaphoreTypeCreateInfo;
        x
    }
}

#[repr(C)]
pub struct TimelineSemaphoreSubmitInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub wait_semaphore_values: VulkanSlice1<'a, u32, u64, 4>,
    pub signal_semaphore_values: VulkanSlice1<'a, u32, u64, 4>,
}

impl<'a> Default for TimelineSemaphoreSubmitInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::TimelineSemaphoreSubmitInfo;
        x
    }
}

#[repr(C)]
pub struct SemaphoreWaitInfo<'a> {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub flags: SemaphoreWaitFlags,
    pub semaphores: VulkanSlice2<'a, u32, Semaphore, u64, 0>,
}

impl<'a> Default for SemaphoreWaitInfo<'a> {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SemaphoreWaitInfo;
        x
    }
}

#[repr(C)]
pub struct SemaphoreSignalInfo {
    pub _type: StructureType,
    pub _next: *const c_void,
    pub semaphore: Semaphore,
    pub value: u64,
}

impl Default for SemaphoreSignalInfo {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::SemaphoreSignalInfo;
        x
    }
}

#[repr(C)]
pub struct MemoryBarrier {
    pub _type: StructureType,
    pub _next: *const c_void,
    /// Memory accesses from the source of the dependency to synchronize
    pub src_access_mask: AccessFlags,
    /// Memory accesses from the destination of the dependency to synchronize
    pub dst_access_mask: AccessFlags,
}

impl Default for MemoryBarrier {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::MemoryBarrier;
        x
    }
}

#[repr(C)]
pub struct BufferMemoryBarrier {
    pub _type: StructureType,
    pub _next: *const c_void,
    /// Memory accesses from the source of the dependency to synchronize
    pub src_access_mask: AccessFlags,
    /// Memory accesses from the destination of the dependency to synchronize
    pub dst_access_mask: AccessFlags,
    /// Queue family to transition ownership from
    pub src_queue_family_index: u32,
    /// Queue family to transition ownership to
    pub dst_queue_family_index: u32,
    /// Buffer to sync
    pub buffer: Buffer,
    /// Offset within the buffer to sync
    pub offset: DeviceSize,
    /// Amount of bytes to sync
    pub size: DeviceSize,
}

impl Default for BufferMemoryBarrier {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::BufferMemoryBarrier;
        x
    }
}

#[repr(C)]
pub struct ConformanceVersion {
    pub major: u8,
    pub minor: u8,
    pub subminor: u8,
    pub patch: u8,
}

#[repr(C)]
pub struct ImageMemoryBarrier {
    pub _type: StructureType,
    pub _next: *const c_void,
    /// Memory accesses from the source of the dependency to synchronize
    pub src_access_mask: AccessFlags,
    /// Memory accesses from the destination of the dependency to synchronize
    pub dst_access_mask: AccessFlags,
    /// Current layout of the image
    pub old_layout: ImageLayout,
    /// New layout to transition the image to
    pub new_layout: ImageLayout,
    /// Queue family to transition ownership from
    pub src_queue_family_index: u32,
    /// Queue family to transition ownership to
    pub dst_queue_family_index: u32,
    /// Image to sync
    pub image: Image,
    /// Subresource range to sync
    pub subresource_range: ImageSubresourceRange,
}

impl Default for ImageMemoryBarrier {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::ImageMemoryBarrier;
        x
    }
}

#[repr(C)]
pub struct PhysicalDeviceSparseProperties {
    ///  Sparse resources support: GPU will access all 2D (single sample) sparse resources using the standard sparse image block shapes (based on pixel format)
    pub residency_standard_2d_block_shape: Bool32,
    ///  Sparse resources support: GPU will access all 2D (multisample) sparse resources using the standard sparse image block shapes (based on pixel format)
    pub residency_standard_2d_multisample_block_shape: Bool32,
    ///  Sparse resources support: GPU will access all 3D sparse resources using the standard sparse image block shapes (based on pixel format)
    pub residency_standard_3d_block_shape: Bool32,
    ///  Sparse resources support: Images with mip level dimensions that are NOT a multiple of the sparse image block dimensions will be placed in the mip tail
    pub residency_aligned_mip_size: Bool32,
    ///  Sparse resources support: GPU can consistently access non-resident regions of a resource, all reads return as if data is 0, writes are discarded
    pub residency_non_resident_strict: Bool32,
}

#[repr(C)]
pub struct PhysicalDeviceLimits {
    ///  max 1D image dimension
    pub max_image_dimension_1d: u32,
    ///  max 2D image dimension
    pub max_image_dimension_2d: u32,
    ///  max 3D image dimension
    pub max_image_dimension_3d: u32,
    ///  max cubemap image dimension
    pub max_image_dimension_cube: u32,
    ///  max layers for image arrays
    pub max_image_array_layers: u32,
    ///  max texel buffer size (fstexels)
    pub max_texel_buffer_elements: u32,
    ///  max uniform buffer range (bytes)
    pub max_uniform_buffer_range: u32,
    ///  max storage buffer range (bytes)
    pub max_storage_buffer_range: u32,
    ///  max size of the push constants pool (bytes)
    pub max_push_constants_size: u32,

    ///  max number of device memory allocations supported
    pub max_memory_allocation_count: u32,
    ///  max number of samplers that can be allocated on a device
    pub max_sampler_allocation_count: u32,
    ///  Granularity (in bytes) at which buffers and images can be bound to adjacent memory for simultaneous usage
    pub buffer_image_granularity: DeviceSize,
    ///  Total address space available for sparse allocations (bytes)
    pub sparse_address_space_size: DeviceSize,

    ///  max number of descriptors sets that can be bound to a pipeline
    pub max_bound_descriptor_sets: u32,
    ///  max number of samplers allowed per-stage in a descriptor set
    pub max_per_stage_descriptor_samplers: u32,
    ///  max number of uniform buffers allowed per-stage in a descriptor set
    pub max_per_stage_descriptor_uniform_buffers: u32,
    ///  max number of storage buffers allowed per-stage in a descriptor set
    pub max_per_stage_descriptor_storage_buffers: u32,
    ///  max number of sampled images allowed per-stage in a descriptor set
    pub max_per_stage_descriptor_sampled_images: u32,
    ///  max number of storage images allowed per-stage in a descriptor set
    pub max_per_stage_descriptor_storage_images: u32,
    ///  max number of input attachments allowed per-stage in a descriptor set
    pub max_per_stage_descriptor_input_attachments: u32,
    ///  max number of resources allowed by a single stage
    pub max_per_stage_resources: u32,
    ///  max number of samplers allowed in all stages in a descriptor set
    pub max_descriptor_set_samplers: u32,
    ///  max number of uniform buffers allowed in all stages in a descriptor set
    pub max_descriptor_set_uniform_buffers: u32,
    ///  max number of dynamic uniform buffers allowed in all stages in a descriptor set
    pub max_descriptor_set_uniform_buffers_dynamic: u32,
    ///  max number of storage buffers allowed in all stages in a descriptor set
    pub max_descriptor_set_storage_buffers: u32,
    ///  max number of dynamic storage buffers allowed in all stages in a descriptor set
    pub max_descriptor_set_storage_buffers_dynamic: u32,
    ///  max number of sampled images allowed in all stages in a descriptor set
    pub max_descriptor_set_sampled_images: u32,
    ///  max number of storage images allowed in all stages in a descriptor set
    pub max_descriptor_set_storage_images: u32,
    ///  max number of input attachments allowed in all stages in a descriptor set
    pub max_descriptor_set_input_attachments: u32,

    ///  max number of vertex input attribute slots
    pub max_vertex_input_attributes: u32,
    ///  max number of vertex input binding slots
    pub max_vertex_input_bindings: u32,
    ///  max vertex input attribute offset added to vertex buffer offset
    pub max_vertex_input_attribute_offset: u32,
    ///  max vertex input binding stride
    pub max_vertex_input_binding_stride: u32,
    ///  max number of output components written by vertex shader
    pub max_vertex_output_components: u32,

    ///  max level supported by tessellation primitive generator
    pub max_tessellation_generation_level: u32,
    ///  max patch size (vertices)
    pub max_tessellation_patch_size: u32,
    ///  max number of input components per-vertex in TCS
    pub max_tessellation_control_per_vertex_input_components: u32,
    ///  max number of output components per-vertex in TCS
    pub max_tessellation_control_per_vertex_output_components: u32,
    ///  max number of output components per-patch in TCS
    pub max_tessellation_control_per_patch_output_components: u32,
    ///  max total number of per-vertex and per-patch output components in TCS
    pub max_tessellation_control_total_output_components: u32,
    ///  tessellation evaluation stage limits

    ///  max number of input components per vertex in TES
    pub max_tessellation_evaluation_input_components: u32,
    ///  max number of output components per vertex in TES
    pub max_tessellation_evaluation_output_components: u32,

    ///  max invocation count supported in geometry shader
    pub max_geometry_shader_invocations: u32,
    ///  max number of input components read in geometry stage
    pub max_geometry_input_components: u32,
    ///  max number of output components written in geometry stage
    pub max_geometry_output_components: u32,
    ///  max number of vertices that can be emitted in geometry stage
    pub max_geometry_output_vertices: u32,
    ///  max total number of components (all vertices) written in geometry stage
    pub max_geometry_total_output_components: u32,

    ///  max number of input components read in fragment stage
    pub max_fragment_input_components: u32,
    ///  max number of output attachments written in fragment stage
    pub max_fragment_output_attachments: u32,
    ///  max number of output attachments written when using dual source blending
    pub max_fragment_dual_src_attachments: u32,
    ///  max total number of storage buffers, storage images and output buffers
    pub max_fragment_combined_output_resources: u32,

    ///  max total storage size of work group local storage (bytes)
    pub max_compute_shared_memory_size: u32,
    ///  max num of compute work groups that may be dispatched by a single command (x,y,z)
    pub max_compute_work_group_count: [u32; 3],
    ///  max total compute invocations in a single local work group
    pub max_compute_work_group_invocations: u32,
    ///  max local size of a compute work group (x,y,z)
    pub max_compute_work_group_size: [u32; 3],
    ///  number bits of subpixel precision in screen x and y
    pub sub_pixel_precision_bits: u32,
    ///  number bits of precision for selecting texel weights
    pub sub_texel_precision_bits: u32,
    ///  number bits of precision for selecting mipmap weights
    pub mipmap_precision_bits: u32,
    ///  max index value for indexed draw calls (for 32-bit indices)
    pub max_draw_indexed_index_value: u32,
    ///  max draw count for indirect draw calls
    pub max_draw_indirect_count: u32,
    ///  max absolute sampler LOD bias
    pub max_sampler_lod_bias: f32,
    ///  max degree of sampler anisotropy
    pub max_sampler_anisotropy: f32,
    ///  max number of active viewports
    pub max_viewports: u32,
    ///  max viewport dimensions (x,y)
    pub max_viewport_dimensions: [u32; 2],
    ///  viewport bounds range (min,max)
    pub viewport_bounds_range: [f32; 2],
    ///  number bits of subpixel precision for viewport
    pub viewport_sub_pixel_bits: u32,
    ///  min required alignment of pointers returned by MapMemory (bytes)
    pub min_memory_map_alignment: usize,
    ///  min required alignment for texel buffer offsets (bytes)
    pub min_texel_buffer_offset_alignment: DeviceSize,
    ///  min required alignment for uniform buffer sizes and offsets (bytes)
    pub min_uniform_buffer_offset_alignment: DeviceSize,
    ///  min required alignment for storage buffer offsets (bytes)
    pub min_storage_buffer_offset_alignment: DeviceSize,
    ///  min texel offset for OpTextureSampleOffset
    pub min_texel_offset: i32,
    ///  max texel offset for OpTextureSampleOffset
    pub max_texel_offset: u32,
    ///  min texel offset for OpTextureGatherOffset
    pub min_texel_gather_offset: i32,
    ///  max texel offset for OpTextureGatherOffset
    pub max_texel_gather_offset: u32,
    ///  furthest negative offset for interpolateAtOffset
    pub min_interpolation_offset: f32,
    ///  furthest positive offset for interpolateAtOffset
    pub max_interpolation_offset: f32,
    ///  number of subpixel bits for interpolateAtOffset
    pub sub_pixel_interpolation_offset_bits: u32,
    ///  max width for a framebuffer
    pub max_framebuffer_width: u32,
    ///  max height for a framebuffer
    pub max_framebuffer_height: u32,
    ///  max layer count for a layered framebuffer
    pub max_framebuffer_layers: u32,
    ///  supported color sample counts for a framebuffer
    pub framebuffer_color_sample_counts: SampleCountFlags,
    ///  supported depth sample counts for a framebuffer
    pub framebuffer_depth_sample_counts: SampleCountFlags,
    ///  supported stencil sample counts for a framebuffer
    pub framebuffer_stencil_sample_counts: SampleCountFlags,
    ///  supported sample counts for a subpass which uses no attachments
    pub framebuffer_no_attachments_sample_counts: SampleCountFlags,
    ///  max number of color attachments per subpass
    pub max_color_attachments: u32,
    ///  supported color sample counts for a non-integer sampled image
    pub sampled_image_color_sample_counts: SampleCountFlags,
    ///  supported sample counts for an integer image
    pub sampled_image_integer_sample_counts: SampleCountFlags,
    ///  supported depth sample counts for a sampled image
    pub sampled_image_depth_sample_counts: SampleCountFlags,
    ///  supported stencil sample counts for a sampled image
    pub sampled_image_stencil_sample_counts: SampleCountFlags,
    ///  supported sample counts for a storage image
    pub storage_image_sample_counts: SampleCountFlags,
    ///  max number of sample mask words
    pub max_sample_mask_words: u32,
    ///  timestamps on graphics and compute queues
    pub timestamp_compute_and_graphics: Bool32,
    ///  number of nanoseconds it takes for timestamp query value to increment by 1
    pub timestamp_period: f32,
    ///  max number of clip distances
    pub max_clip_distances: u32,
    ///  max number of cull distances
    pub max_cull_distances: u32,
    ///  max combined number of user clipping
    pub max_combined_clip_and_cull_distances: u32,
    ///  distinct queue priorities available
    pub discrete_queue_priorities: u32,
    ///  range (min,max) of supported point sizes
    pub point_size_range: [f32; 2],
    ///  range (min,max) of supported line widths
    pub line_width_range: [f32; 2],
    ///  granularity of supported point sizes
    pub point_size_granularity: f32,
    ///  granularity of supported line widths
    pub line_width_granularity: f32,
    ///  line rasterization follows preferred rules
    pub strict_lines: Bool32,
    ///  supports standard sample locations for all supported sample counts
    pub standard_sample_locations: Bool32,
    ///  optimal offset of buffer copies
    pub optimal_buffer_copy_offset_alignment: DeviceSize,
    ///  optimal pitch of buffer copies
    pub optimal_buffer_copy_row_pitch_alignment: DeviceSize,
    ///  minimum size and alignment for non-coherent host-mapped device memory access
    pub non_coherent_atom_size: DeviceSize,
}

#[repr(C)]
pub struct PhysicalDeviceFeatures {
    /// out of bounds buffer accesses are well defined
    pub robust_buffer_access: Bool32,
    /// full 32-bit range of indices for indexed draw calls
    pub full_draw_index_uint32: Bool32,
    /// image views which are arrays of cube maps
    pub image_cube_array: Bool32,
    /// blending operations are controlled per-attachment
    pub independent_blend: Bool32,
    /// geometry stage
    pub geometry_shader: Bool32,
    /// tessellation control and evaluation stage
    pub tessellation_shader: Bool32,
    /// per-sample shading and interpolation
    pub sample_rate_shading: Bool32,
    /// blend operations which take two sources
    pub dual_src_blend: Bool32,
    /// logic operations
    pub logic_op: Bool32,
    /// multi draw indirect
    pub multi_draw_indirect: Bool32,
    /// indirect draws can use non-zero firstInstance
    pub draw_indirect_first_instance: Bool32,
    /// depth clamping
    pub depth_clamp: Bool32,
    /// depth bias clamping
    pub depth_bias_clamp: Bool32,
    /// point and wireframe fill modes
    pub fill_mode_non_solid: Bool32,
    /// depth bounds test
    pub depth_bounds: Bool32,
    /// lines with width greater than 1
    pub wide_lines: Bool32,
    /// points with size greater than 1
    pub large_points: Bool32,
    /// the fragment alpha component can be forced to maximum representable alpha value
    pub alpha_to_one: Bool32,
    /// viewport arrays
    pub multi_viewport: Bool32,
    /// anisotropic sampler filtering
    pub sampler_anisotropy: Bool32,
    /// ETC texture compression formats
    pub texture_compression_etc2: Bool32,
    /// ASTC LDR texture compression formats
    pub texture_compression_astc_ldr: Bool32,
    /// BC1-7 texture compressed formats
    pub texture_compression_bc: Bool32,
    /// precise occlusion queries returning actual sample counts
    pub occlusion_query_precise: Bool32,
    /// pipeline statistics query
    pub pipeline_statistics_query: Bool32,
    /// stores and atomic ops on storage buffers and images are supported in vertex, tessellation, and geometry stages
    pub vertex_pipeline_stores_and_atomics: Bool32,
    /// stores and atomic ops on storage buffers and images are supported in the fragment stage
    pub fragment_stores_and_atomics: Bool32,
    /// tessellation and geometry stages can export point size
    pub shader_tessellation_and_geometry_point_size: Bool32,
    /// image gather with run-time values and independent offsets
    pub shader_image_gather_extended: Bool32,
    /// the extended set of formats can be used for storage images
    pub shader_storage_image_extended_formats: Bool32,
    /// multisample images can be used for storage images
    pub shader_storage_image_multisample: Bool32,
    /// read from storage image does not require format qualifier
    pub shader_storage_image_read_without_format: Bool32,
    /// write to storage image does not require format qualifier
    pub shader_storage_image_write_without_format: Bool32,
    /// arrays of uniform buffers can be accessed with dynamically uniform indices
    pub shader_uniform_buffer_array_dynamic_indexing: Bool32,
    /// arrays of sampled images can be accessed with dynamically uniform indices
    pub shader_sampled_image_array_dynamic_indexing: Bool32,
    /// arrays of storage buffers can be accessed with dynamically uniform indices
    pub shader_storage_buffer_array_dynamic_indexing: Bool32,
    /// arrays of storage images can be accessed with dynamically uniform indices
    pub shader_storage_image_array_dynamic_indexing: Bool32,
    /// clip distance in shaders
    pub shader_clip_distance: Bool32,
    /// cull distance in shaders
    pub shader_cull_distance: Bool32,
    /// 64-bit floats (doubles) in shaders
    pub shader_float64: Bool32,
    /// 64-bit integers in shaders
    pub shader_int64: Bool32,
    /// 16-bit integers in shaders
    pub shader_int16: Bool32,
    /// shader can use texture operations that return resource residency information (requires sparseNonResident support)
    pub shader_resource_residency: Bool32,
    /// shader can use texture operations that specify minimum resource LOD
    pub shader_resource_min_lod: Bool32,
    /// Sparse resources support: Resource memory can be managed at opaque page level rather than object level
    pub sparse_binding: Bool32,
    /// Sparse resources support: GPU can access partially resident buffers
    pub sparse_residency_buffer: Bool32,
    /// Sparse resources support: GPU can access partially resident 2D (non-MSAA non-depth/stencil) images
    pub sparse_residency_image_2d: Bool32,
    /// Sparse resources support: GPU can access partially resident 3D images
    pub sparse_residency_image_3d: Bool32,
    /// Sparse resources support: GPU can access partially resident MSAA 2D images with 2 samples
    pub sparse_residency_2_samples: Bool32,
    /// Sparse resources support: GPU can access partially resident MSAA 2D images with 4 samples
    pub sparse_residency_4_samples: Bool32,
    /// Sparse resources support: GPU can access partially resident MSAA 2D images with 8 samples
    pub sparse_residency_8_samples: Bool32,
    /// Sparse resources support: GPU can access partially resident MSAA 2D images with 16 samples
    pub sparse_residency_16_samples: Bool32,
    /// Sparse resources support: GPU can correctly access data aliased into multiple locations (opt-in)
    pub sparse_residency_aliased: Bool32,
    /// multisample rate must be the same for all pipelines in a subpass
    pub variable_multisample_rate: Bool32,
    /// Queries may be inherited from primary to secondary command buffers
    pub inherited_queries: Bool32,
}

impl Default for PhysicalDeviceFeatures {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[repr(C)]
pub struct PhysicalDeviceFeatures2 {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub features: PhysicalDeviceFeatures,
}

impl Default for PhysicalDeviceFeatures2 {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PhysicalDeviceFeatures2;
        x
    }
}

#[repr(C)]
pub struct PhysicalDeviceVulkan11Features {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub storage_buffer_16bit_access: Bool32,
    pub uniform_and_storage_buffer_16bit_access: Bool32,
    pub storage_push_constant16: Bool32,
    pub storage_input_output16: Bool32,
    pub multiview: Bool32,
    pub multiview_geometry_shader: Bool32,
    pub multiview_tessellation_shader: Bool32,
    pub variable_pointers_storage_buffer: Bool32,
    pub variable_pointers: Bool32,
    pub protected_memory: Bool32,
    pub sampler_ycbcr_conversion: Bool32,
    pub shader_draw_parameters: Bool32,
}

impl Default for PhysicalDeviceVulkan11Features {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PhysicalDeviceVulkan11Features;
        x
    }
}

#[repr(C)]
pub struct PhysicalDeviceVulkan12Features {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub sampler_mirror_clamp_to_edge: Bool32,
    pub draw_indirect_count: Bool32,
    pub storage_buffer_8bit_access: Bool32,
    pub uniform_and_storage_buffer_8bit_access: Bool32,
    pub storage_push_constant8: Bool32,
    pub shader_buffer_int64_atomics: Bool32,
    pub shader_shared_int64_atomics: Bool32,
    pub shader_float16: Bool32,
    pub shader_int8: Bool32,
    pub descriptor_indexing: Bool32,
    pub shader_input_attachment_array_dynamic_indexing: Bool32,
    pub shader_uniform_texel_buffer_array_dynamic_indexing: Bool32,
    pub shader_storage_texel_buffer_array_dynamic_indexing: Bool32,
    pub shader_uniform_buffer_array_non_uniform_indexing: Bool32,
    pub shader_sampled_image_array_non_uniform_indexing: Bool32,
    pub shader_storage_buffer_array_non_uniform_indexing: Bool32,
    pub shader_storage_image_array_non_uniform_indexing: Bool32,
    pub shader_input_attachment_array_non_uniform_indexing: Bool32,
    pub shader_uniform_texel_buffer_array_non_uniform_indexing: Bool32,
    pub shader_storage_texel_buffer_array_non_uniform_indexing: Bool32,
    pub descriptor_binding_uniform_buffer_update_after_bind: Bool32,
    pub descriptor_binding_sampled_image_update_after_bind: Bool32,
    pub descriptor_binding_storage_image_update_after_bind: Bool32,
    pub descriptor_binding_storage_buffer_update_after_bind: Bool32,
    pub descriptor_binding_uniform_texel_buffer_update_after_bind: Bool32,
    pub descriptor_binding_storage_texel_buffer_update_after_bind: Bool32,
    pub descriptor_binding_update_unused_while_pending: Bool32,
    pub descriptor_binding_partially_bound: Bool32,
    pub descriptor_binding_variable_descriptor_count: Bool32,
    pub runtime_descriptor_array: Bool32,
    pub sampler_filter_minmax: Bool32,
    pub scalar_block_layout: Bool32,
    pub imageless_framebuffer: Bool32,
    pub uniform_buffer_standard_layout: Bool32,
    pub shader_subgroup_extended_types: Bool32,
    pub separate_depth_stencil_layouts: Bool32,
    pub host_query_reset: Bool32,
    pub timeline_semaphore: Bool32,
    pub buffer_device_address: Bool32,
    pub buffer_device_address_capture_replay: Bool32,
    pub buffer_device_address_multi_device: Bool32,
    pub vulkan_memory_model: Bool32,
    pub vulkan_memory_model_device_scope: Bool32,
    pub vulkan_memory_model_availability_visibility_chains: Bool32,
    pub shader_output_viewport_index: Bool32,
    pub shader_output_layer: Bool32,
    pub subgroup_broadcast_dynamic_id: Bool32,
}

impl Default for PhysicalDeviceVulkan12Features {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PhysicalDeviceVulkan12Features;
        x
    }
}

#[repr(C)]
pub struct PhysicalDeviceVulkan13Features {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub robust_image_access: Bool32,
    pub inline_uniform_block: Bool32,
    pub descriptor_binding_inline_uniform_block_update_after_bind: Bool32,
    pub pipeline_creation_cache_control: Bool32,
    pub private_data: Bool32,
    pub shader_demote_to_helper_invocation: Bool32,
    pub shader_terminate_invocation: Bool32,
    pub subgroup_size_control: Bool32,
    pub compute_full_subgroups: Bool32,
    pub synchronization2: Bool32,
    pub texture_compression_astc_hdr: Bool32,
    pub shader_zero_initialize_workgroup_memory: Bool32,
    pub dynamic_rendering: Bool32,
    pub shader_integer_dot_product: Bool32,
    pub maintenance4: Bool32,
}

impl Default for PhysicalDeviceVulkan13Features {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PhysicalDeviceVulkan13Features;
        x
    }
}

pub struct PhysicalDeviceSwapchainMaintenance1FeaturesEXT {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub swapchain_maintenance1: Bool32,
}

impl Default for PhysicalDeviceSwapchainMaintenance1FeaturesEXT {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PhysicalDeviceSwapchainMaintenance1FeaturesExt;
        x
    }
}

#[repr(C)]
pub struct PhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: PhysicalDeviceType,
    pub device_name: [c_char; MAX_PHYSICAL_DEVICE_NAME_SIZE as usize],
    pub pipeline_cache_uuid: [u8; UUID_SIZE as usize],
    pub limits: PhysicalDeviceLimits,
    pub sparse_properties: PhysicalDeviceSparseProperties,
}

impl Default for PhysicalDeviceProperties {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[repr(C)]
pub struct PhysicalDeviceProperties2 {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub properties: PhysicalDeviceProperties,
}

impl Default for PhysicalDeviceProperties2 {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PhysicalDeviceProperties2;
        x
    }
}

#[repr(C)]
pub struct PhysicalDeviceVulkan11Properties {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub device_uuid: [u8; UUID_SIZE as usize],
    pub driver_uuid: [u8; UUID_SIZE as usize],
    pub device_luid: [u8; LUID_SIZE as usize],
    pub device_node_mask: u32,
    pub device_luid_valid: Bool32,
    pub subgroup_size: u32,
    pub subgroup_supported_stages: ShaderStageFlags,
    pub subgroup_supported_operations: SubgroupFeatureFlags,
    pub subgroup_quad_operations_in_all_stages: Bool32,
    pub point_clipping_behavior: PointClippingBehavior,
    pub max_multiview_view_count: u32,
    pub max_multiview_instance_index: u32,
    pub protected_no_fault: Bool32,
    pub max_per_set_descriptors: u32,
    pub max_memory_allocation_size: DeviceSize,
}

impl Default for PhysicalDeviceVulkan11Properties {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PhysicalDeviceVulkan11Properties;
        x
    }
}

#[repr(C)]
pub struct PhysicalDeviceVulkan12Properties {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub driver_id: DriverId,
    pub driver_name: [u8; MAX_DRIVER_NAME_SIZE as usize],
    pub driver_info: [u8; MAX_DRIVER_INFO_SIZE as usize],
    pub conformance_version: ConformanceVersion,
    pub denorm_behavior_independence: ShaderFloatControlsIndependence,
    pub rounding_mode_independence: ShaderFloatControlsIndependence,
    pub shader_signed_zero_inf_nan_preserve_float16: Bool32,
    pub shader_signed_zero_inf_nan_preserve_float32: Bool32,
    pub shader_signed_zero_inf_nan_preserve_float64: Bool32,
    pub shader_denorm_preserve_float16: Bool32,
    pub shader_denorm_preserve_float32: Bool32,
    pub shader_denorm_preserve_float64: Bool32,
    pub shader_denorm_flush_to_zero_float16: Bool32,
    pub shader_denorm_flush_to_zero_float32: Bool32,
    pub shader_denorm_flush_to_zero_float64: Bool32,
    pub shader_rounding_mode_rte_float16: Bool32,
    pub shader_rounding_mode_rte_float32: Bool32,
    pub shader_rounding_mode_rte_float64: Bool32,
    pub shader_rounding_mode_rtz_float16: Bool32,
    pub shader_rounding_mode_rtz_float32: Bool32,
    pub shader_rounding_mode_rtz_float64: Bool32,
    pub max_update_after_bind_descriptors_in_all_pools: u32,
    pub shader_uniform_buffer_array_non_uniform_indexing_native: Bool32,
    pub shader_sampled_image_array_non_uniform_indexing_native: Bool32,
    pub shader_storage_buffer_array_non_uniform_indexing_native: Bool32,
    pub shader_storage_image_array_non_uniform_indexing_native: Bool32,
    pub shader_input_attachment_array_non_uniform_indexing_native: Bool32,
    pub robust_buffer_access_update_after_bind: Bool32,
    pub quad_divergent_implicit_lod: Bool32,
    pub max_per_stage_descriptor_update_after_bind_samplers: u32,
    pub max_per_stage_descriptor_update_after_bind_uniform_buffers: u32,
    pub max_per_stage_descriptor_update_after_bind_storage_buffers: u32,
    pub max_per_stage_descriptor_update_after_bind_sampled_images: u32,
    pub max_per_stage_descriptor_update_after_bind_storage_images: u32,
    pub max_per_stage_descriptor_update_after_bind_input_attachments: u32,
    pub max_per_stage_update_after_bind_resources: u32,
    pub max_descriptor_set_update_after_bind_samplers: u32,
    pub max_descriptor_set_update_after_bind_uniform_buffers: u32,
    pub max_descriptor_set_update_after_bind_uniform_buffers_dynamic: u32,
    pub max_descriptor_set_update_after_bind_storage_buffers: u32,
    pub max_descriptor_set_update_after_bind_storage_buffers_dynamic: u32,
    pub max_descriptor_set_update_after_bind_sampled_images: u32,
    pub max_descriptor_set_update_after_bind_storage_images: u32,
    pub max_descriptor_set_update_after_bind_input_attachments: u32,
    pub supported_depth_resolve_modes: ResolveModeFlags,
    pub supported_stencil_resolve_modes: ResolveModeFlags,
    pub independent_resolve_none: Bool32,
    pub independent_resolve: Bool32,
    pub filter_minmax_single_component_formats: Bool32,
    pub filter_minmax_image_component_mapping: Bool32,
    pub max_timeline_semaphore_value_difference: u64,
    pub framebuffer_integer_color_sample_counts: SampleCountFlags,
}

impl Default for PhysicalDeviceVulkan12Properties {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PhysicalDeviceVulkan12Properties;
        x
    }
}

#[repr(C)]
pub struct PhysicalDeviceVulkan13Properties {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub min_subgroup_size: u32,
    pub max_subgroup_size: u32,
    pub max_compute_workgroup_subgroups: u32,
    pub required_subgroup_size_stages: ShaderStageFlags,
    pub max_inline_uniform_block_size: u32,
    pub max_per_stage_descriptor_inline_uniform_blocks: u32,
    pub max_per_stage_descriptor_update_after_bind_inline_uniform_blocks: u32,
    pub max_descriptor_set_inline_uniform_blocks: u32,
    pub max_descriptor_set_update_after_bind_inline_uniform_blocks: u32,
    pub max_inline_uniform_total_size: u32,
    pub integer_dot_product_8bit_unsigned_accelerated: Bool32,
    pub integer_dot_product_8bit_signed_accelerated: Bool32,
    pub integer_dot_product_8bit_mixed_signedness_accelerated: Bool32,
    pub integer_dot_product_4x8bit_packed_unsigned_accelerated: Bool32,
    pub integer_dot_product_4x8bit_packed_signed_accelerated: Bool32,
    pub integer_dot_product_4x8bit_packed_mixed_signedness_accelerated: Bool32,
    pub integer_dot_product_16bit_unsigned_accelerated: Bool32,
    pub integer_dot_product_16bit_signed_accelerated: Bool32,
    pub integer_dot_product_16bit_mixed_signedness_accelerated: Bool32,
    pub integer_dot_product_32bit_unsigned_accelerated: Bool32,
    pub integer_dot_product_32bit_signed_accelerated: Bool32,
    pub integer_dot_product_32bit_mixed_signedness_accelerated: Bool32,
    pub integer_dot_product_64bit_unsigned_accelerated: Bool32,
    pub integer_dot_product_64bit_signed_accelerated: Bool32,
    pub integer_dot_product_64bit_mixed_signedness_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_8bit_unsigned_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_8bit_signed_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_8bit_mixed_signedness_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_4x8bit_packed_unsigned_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_4x8bit_packed_signed_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_4x8bit_packed_mixed_signedness_accelerated:
        Bool32,
    pub integer_dot_product_accumulating_saturating_16bit_unsigned_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_16bit_signed_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_16bit_mixed_signedness_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_32bit_unsigned_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_32bit_signed_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_32bit_mixed_signedness_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_64bit_unsigned_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_64bit_signed_accelerated: Bool32,
    pub integer_dot_product_accumulating_saturating_64bit_mixed_signedness_accelerated: Bool32,
    pub storage_texel_buffer_offset_alignment_bytes: DeviceSize,
    pub storage_texel_buffer_offset_single_texel_alignment: Bool32,
    pub uniform_texel_buffer_offset_alignment_bytes: DeviceSize,
    pub uniform_texel_buffer_offset_single_texel_alignment: Bool32,
    pub max_buffer_size: DeviceSize,
}

impl Default for PhysicalDeviceVulkan13Properties {
    fn default() -> Self {
        let mut x = unsafe { MaybeUninit::<Self>::zeroed().assume_init() };
        x._type = StructureType::PhysicalDeviceVulkan13Properties;
        x
    }
}

#[repr(C)]
pub struct PhysicalDeviceSurfaceInfo2KHR {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub surface: SurfaceKHR,
}

impl Default for PhysicalDeviceSurfaceInfo2KHR {
    fn default() -> Self {
        Self {
            _type: StructureType::PhysicalDeviceSurfaceInfo2Khr,
            _next: core::ptr::null_mut(),
            surface: Default::default(),
        }
    }
}

#[repr(C)]
pub struct SurfaceCapabilities2KHR {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub surface_capabilities: SurfaceCapabilitiesKHR,
}

impl Default for SurfaceCapabilities2KHR {
    fn default() -> Self {
        Self {
            _type: StructureType::SurfaceCapabilities2Khr,
            _next: core::ptr::null_mut(),
            surface_capabilities: Default::default(),
        }
    }
}

#[repr(C)]
pub struct SurfaceFormat2KHR {
    pub _type: StructureType,
    pub _next: *mut c_void,
    pub surface_format: SurfaceFormatKHR,
}

impl Default for SurfaceFormat2KHR {
    fn default() -> Self {
        Self {
            _type: StructureType::SurfaceFormat2Khr,
            _next: core::ptr::null_mut(),
            surface_format: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[must_use]
    fn is_aligned_to<T>(ptr: *const T, align: usize) -> bool {
        if align == 0 || !align.is_power_of_two() {
            panic!("is_aligned_to: align is not a power-of-two");
        }

        (ptr as usize) & (align - 1) == 0
    }

    #[must_use]
    fn is_aligned<T>(ptr: *const T) -> bool {
        is_aligned_to(ptr, std::mem::align_of::<T>())
    }

    #[test]
    fn vulkan_slice_alignment() {
        use std::ptr::addr_of;

        {
            let device_queue_create_info = DeviceQueueCreateInfo::default();
            assert!(is_aligned(addr_of!(
                device_queue_create_info.queue_priorities.ptr
            )));
        }

        {
            let device_create_info = DeviceCreateInfo::default();
            assert!(is_aligned(addr_of!(
                device_create_info.enabled_extension_names.ptr
            )));
            assert!(is_aligned(addr_of!(device_create_info.enabled_layers.ptr)));
            assert!(is_aligned(addr_of!(
                device_create_info.queue_create_infos.ptr
            )));
        }

        {
            let instance_create_info = InstanceCreateInfo::default();
            assert!(is_aligned(addr_of!(
                instance_create_info.enabled_extension_names.ptr
            )));
            assert!(is_aligned(addr_of!(
                instance_create_info.enabled_layers.ptr
            )));
        }

        {
            let submit_info = SubmitInfo::default();
            assert!(is_aligned(addr_of!(submit_info.command_buffers.ptr)));
            assert!(is_aligned(addr_of!(submit_info.signal_semaphores.ptr)));
            assert!(is_aligned(addr_of!(submit_info.wait_semaphores.ptr0)));
            assert!(is_aligned(addr_of!(submit_info.wait_semaphores.ptr1)));
        }

        {
            let submit_info_2 = SubmitInfo2::default();
            assert!(is_aligned(addr_of!(submit_info_2.command_buffer_infos.ptr)));
            assert!(is_aligned(addr_of!(
                submit_info_2.signal_semaphore_infos.ptr
            )));
            assert!(is_aligned(addr_of!(submit_info_2.wait_semaphore_infos.ptr)));
        }

        {
            let swapchain_create_info = SwapchainCreateInfoKHR::default();
            assert!(is_aligned(addr_of!(
                swapchain_create_info.queue_family_indices.ptr
            )));
        }

        {
            let present_info_khr = PresentInfoKHR::default();
            assert!(is_aligned(addr_of!(present_info_khr.wait_semaphores.ptr)));
            assert!(is_aligned(addr_of!(present_info_khr.swapchains.ptr0)));
            assert!(is_aligned(addr_of!(present_info_khr.swapchains.ptr1)));
        }

        {
            let image_create_info = ImageCreateInfo::default();
            assert!(is_aligned(addr_of!(
                image_create_info.queue_family_indices.ptr
            )));
        }

        {
            let rendering_info = RenderingInfo::default();
            assert!(is_aligned(addr_of!(rendering_info.color_attachments.ptr)));
        }

        {
            let pipeline_rendering_create_info = PipelineRenderingCreateInfo::default();
            assert!(is_aligned(addr_of!(
                pipeline_rendering_create_info.color_attachment_formats.ptr
            )));
        }

        {
            let render_pass_begin_info = RenderPassBeginInfo::default();
            assert!(is_aligned(addr_of!(
                render_pass_begin_info.clear_values.ptr
            )));
        }

        {
            let dependency_info = DependencyInfo::default();
            assert!(is_aligned(addr_of!(dependency_info.memory_barriers.ptr)));
            assert!(is_aligned(addr_of!(
                dependency_info.buffer_memory_barriers.ptr
            )));
            assert!(is_aligned(addr_of!(
                dependency_info.image_memory_barriers.ptr
            )));
        }

        {
            let subpass_description = SubpassDescription::default();
            assert!(is_aligned(addr_of!(
                subpass_description.input_attachments.ptr
            )));
            assert!(is_aligned(addr_of!(
                subpass_description.color_attachments.ptr
            )));
            assert!(is_aligned(addr_of!(
                subpass_description.preserve_attachments.ptr
            )));
        }

        {
            let render_pass_create_info = RenderPassCreateInfo::default();
            assert!(is_aligned(addr_of!(
                render_pass_create_info.attachments.ptr
            )));
            assert!(is_aligned(addr_of!(render_pass_create_info.subpasses.ptr)));
            assert!(is_aligned(addr_of!(
                render_pass_create_info.dependencies.ptr
            )));
        }

        {
            let shader_module_create_info = ShaderModuleCreateInfo::default();
            assert!(is_aligned(addr_of!(shader_module_create_info.code.ptr)));
        }

        {
            let descriptor_set_layout_create_info = DescriptorSetLayoutCreateInfo::default();
            assert!(is_aligned(addr_of!(
                descriptor_set_layout_create_info.bindings.ptr
            )));
        }

        {
            let descriptor_pool_create_info = DescriptorPoolCreateInfo::default();
            assert!(is_aligned(addr_of!(
                descriptor_pool_create_info.pool_sizes.ptr
            )));
        }

        {
            let descriptor_set_allocate_info = DescriptorSetAllocateInfo::default();
            assert!(is_aligned(addr_of!(
                descriptor_set_allocate_info.set_layouts.ptr
            )));
        }

        {
            let specialization_info = SpecializationInfo::default();
            assert!(is_aligned(addr_of!(specialization_info.map_entries.ptr)));
        }

        {
            let pipeline_vertex_input_state_create_info =
                PipelineVertexInputStateCreateInfo::default();
            assert!(is_aligned(addr_of!(
                pipeline_vertex_input_state_create_info
                    .vertex_attribute_descriptions
                    .ptr
            )));
            assert!(is_aligned(addr_of!(
                pipeline_vertex_input_state_create_info
                    .vertex_binding_descriptions
                    .ptr
            )));
        }

        {
            let pipeline_viewport_state_create_info = PipelineViewportStateCreateInfo::default();
            assert!(is_aligned(addr_of!(
                pipeline_viewport_state_create_info.viewports.ptr
            )));
            assert!(is_aligned(addr_of!(
                pipeline_viewport_state_create_info.scissors.ptr
            )));
        }

        {
            let pipeline_color_blend_state_create_info =
                PipelineColorBlendStateCreateInfo::default();
            assert!(is_aligned(addr_of!(
                pipeline_color_blend_state_create_info.attachments.ptr
            )));
        }

        {
            let pipeline_dynamic_state_create_info = PipelineDynamicStateCreateInfo::default();
            assert!(is_aligned(addr_of!(
                pipeline_dynamic_state_create_info.dynamic_states.ptr
            )));
        }

        {
            let graphics_pipeline_create_info = GraphicsPipelineCreateInfo::default();
            assert!(is_aligned(addr_of!(
                graphics_pipeline_create_info.stages.ptr
            )));
        }

        {
            let pipeline_layout_create_info = PipelineLayoutCreateInfo::default();
            assert!(is_aligned(addr_of!(
                pipeline_layout_create_info.set_layouts.ptr
            )));
            assert!(is_aligned(addr_of!(
                pipeline_layout_create_info.push_constant_ranges.ptr
            )));
        }

        {
            let buffer_create_info = BufferCreateInfo::default();
            assert!(is_aligned(addr_of!(
                buffer_create_info.queue_family_indices.ptr
            )));
        }

        {
            let framebuffer_create_info = FramebufferCreateInfo::default();
            assert!(is_aligned(addr_of!(
                framebuffer_create_info.attachments.ptr
            )));
        }

        {
            let framebuffer_attachment_image_create_info =
                FramebufferAttachmentImageInfo::default();
            assert!(is_aligned(addr_of!(
                framebuffer_attachment_image_create_info.view_formats.ptr
            )));
        }

        {
            let framebuffer_attachments_create_info = FramebufferAttachmentsCreateInfo::default();
            assert!(is_aligned(addr_of!(
                framebuffer_attachments_create_info
                    .attachment_image_infos
                    .ptr
            )));
        }

        {
            let timeline_semaphore_submit_info = TimelineSemaphoreSubmitInfo::default();
            assert!(is_aligned(addr_of!(
                timeline_semaphore_submit_info.wait_semaphore_values.ptr
            )));
            assert!(is_aligned(addr_of!(
                timeline_semaphore_submit_info.signal_semaphore_values.ptr
            )));
        }

        {
            let semaphore_wait_info = SemaphoreWaitInfo::default();
            assert!(is_aligned(addr_of!(semaphore_wait_info.semaphores.ptr0)));
            assert!(is_aligned(addr_of!(semaphore_wait_info.semaphores.ptr1)));
        }

        {
            let swapchain_present_modes_create_info = SwapchainPresentModesCreateInfoEXT::default();
            assert!(is_aligned(addr_of!(
                swapchain_present_modes_create_info.present_modes.ptr
            )));

            let surface_present_mode_capability = SurfacePresentModeCompatibilityEXT::default();
            assert!(is_aligned(addr_of!(
                surface_present_mode_capability.present_modes.ptr
            )));

            let swapchain_present_fence_info = SwapchainPresentFenceInfoEXT::default();
            assert!(is_aligned(addr_of!(
                swapchain_present_fence_info.fences.ptr
            )));

            let release_swapchain_images_info = ReleaseSwapchainImagesInfoEXT::default();
            assert!(is_aligned(addr_of!(
                release_swapchain_images_info.image_indices.ptr
            )));
        }
    }
}
