use std::{ffi::c_void, os::raw::c_char};

use super::*;

pub type FnVoidFunction = extern "system" fn();

pub type FnAllocationFunction = extern "system" fn(
    user_data: *mut c_void,
    size: usize,
    alignment: usize,
    allocationScope: SystemAllocationScope,
) -> *mut c_void;

pub type FnReallocationFunction = extern "system" fn(
    user_data: *mut c_void,
    original: *mut c_void,
    size: usize,
    alignment: usize,
    allocation_scope: SystemAllocationScope,
) -> *mut c_void;

pub type FnFreeFunction = extern "system" fn(user_data: *mut c_void, memory: *mut c_void);

pub type FnInternalAllocationNotification = extern "system" fn(
    user_data: *mut c_void,
    size: usize,
    allocation_type: InternalAllocationType,
    allocation_scope: SystemAllocationScope,
);

pub type FnInternalFreeNotification = extern "system" fn(
    user_data: *mut c_void,
    size: usize,
    allocation_type: InternalAllocationType,
    allocation_scope: SystemAllocationScope,
);

pub type FnGetInstanceProcAddr =
    extern "system" fn(instance: Instance, name: *const c_char) -> Option<FnVoidFunction>;

pub type FnEnumerateInstanceVersion = extern "system" fn(api_version: &mut u32) -> Result;

pub type FnCreateInstance = extern "system" fn(
    create_info: &InstanceCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    instance: &mut Instance,
) -> Result;

pub type FnDestroyInstance =
    extern "system" fn(instance: Instance, allocator: Option<&AllocationCallbacks>);

pub type FnEnumeratePhysicalDevices = extern "system" fn(
    instance: Instance,
    physical_device_count: &mut u32,
    physical_devices: *mut PhysicalDevice,
) -> Result;

pub type FnGetPhysicalDeviceFeatures =
    extern "system" fn(physical_device: PhysicalDevice, features: *mut PhysicalDeviceFeatures);

pub type FnGetPhysicalDeviceFeatures2 =
    extern "system" fn(physical_device: PhysicalDevice, features: *mut PhysicalDeviceFeatures2);

pub type FnGetPhysicalDeviceFormatProperties = extern "system" fn(
    physicalDevice: PhysicalDevice,
    format: Format,
    format_properties: &mut FormatProperties,
);

pub type FnGetPhysicalDeviceImageFormatProperties = extern "system" fn(
    physicalDevice: PhysicalDevice,
    format: Format,
    r#type: ImageType,
    tiling: ImageTiling,
    usage: ImageUsageFlags,
    flags: ImageCreateFlags,
    image_format_properties: &mut ImageFormatProperties,
) -> Result;

pub type FnGetPhysicalDeviceProperties =
    extern "system" fn(physicalDevice: PhysicalDevice, properties: *mut PhysicalDeviceProperties);

pub type FnGetPhysicalDeviceProperties2 =
    extern "system" fn(physicalDevice: PhysicalDevice, properties: *mut PhysicalDeviceProperties2);

pub type FnGetPhysicalDeviceQueueFamilyProperties = extern "system" fn(
    physical_device: PhysicalDevice,
    queue_family_property_count: &mut u32,
    queue_family_properties: *mut QueueFamilyProperties,
);

pub type FnGetPhysicalDeviceMemoryProperties = extern "system" fn(
    physical_device: PhysicalDevice,
    memory_properties: *mut PhysicalDeviceMemoryProperties,
);

pub type FnDestroySurfaceKHR = extern "system" fn(
    instance: Instance,
    surface: SurfaceKHR,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnGetPhysicalDeviceSurfaceSupportKHR = extern "system" fn(
    physical_device: PhysicalDevice,
    queue_family_index: u32,
    surface: SurfaceKHR,
    supported: &mut Bool32,
) -> Result;

pub type FnGetPhysicalDeviceSurfaceCapabilitiesKHR = extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    surface_capabilities: &mut SurfaceCapabilitiesKHR,
) -> Result;

pub type FnGetPhysicalDeviceSurfaceFormatsKHR = extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    surface_format_count: &mut u32,
    surface_formats: *mut SurfaceFormatKHR,
) -> Result;

pub type FnGetPhysicalDeviceSurfacePresentModesKHR = extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    present_mode_count: &mut u32,
    present_modes: *mut PresentModeKHR,
) -> Result;

pub type FnCreateDevice = extern "system" fn(
    physical_device: PhysicalDevice,
    create_info: &DeviceCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    device: &mut Device,
) -> Result;

pub type FnGetDeviceProcAddr =
    extern "system" fn(device: Device, name: *const c_char) -> Option<FnVoidFunction>;

pub type FnDestroyDevice =
    extern "system" fn(device: Device, allocator: Option<&AllocationCallbacks>);

pub type FnGetDeviceQueue = extern "system" fn(
    device: Device,
    queue_family_index: u32,
    queue_index: u32,
    queue: &mut Queue,
);

pub type FnQueueSubmit = extern "system" fn(
    queue: Queue,
    submit_count: u32,
    submits: *const SubmitInfo,
    fence: Fence,
) -> Result;

pub type FnQueueSubmit2 = extern "system" fn(
    queue: Queue,
    submit_count: u32,
    submits: *const SubmitInfo2,
    fence: Fence,
) -> Result;

pub type FnQueueWaitIdle = extern "system" fn(queue: Queue) -> Result;

pub type FnDeviceWaitIdle = extern "system" fn(device: Device) -> Result;

pub type FnAllocateMemory = extern "system" fn(
    device: Device,
    allocate_info: &MemoryAllocateInfo,
    allocator: Option<&AllocationCallbacks>,
    memory: *mut DeviceMemory,
) -> Result;

pub type FnFreeMemory = extern "system" fn(
    device: Device,
    memory: DeviceMemory,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnMapMemory = extern "system" fn(
    device: Device,
    memory: DeviceMemory,
    offset: DeviceSize,
    size: DeviceSize,
    flags: MemoryMapFlags,
    data: &mut *mut c_void,
) -> Result;

pub type FnUnmapMemory = extern "system" fn(device: Device, memory: DeviceMemory);

pub type FnCreateImageView = extern "system" fn(
    device: Device,
    create_info: &ImageViewCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    view: &mut ImageView,
) -> Result;

pub type FnDestroyImageView = extern "system" fn(
    device: Device,
    image_view: ImageView,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnCreateShaderModule = extern "system" fn(
    device: Device,
    create_info: &ShaderModuleCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    shader_module: &mut ShaderModule,
) -> Result;

pub type FnDestroyShaderModule = extern "system" fn(
    device: Device,
    shader_module: ShaderModule,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnCreatePipelineCache = extern "system" fn(
    device: Device,
    create_info: &PipelineCacheCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    pipeline_cache: &mut PipelineCache,
) -> Result;

pub type FnDestroyPipelineCache = extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnGetPipelineCacheData = extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    data_size: &mut usize,
    data: *mut c_void,
) -> Result;

pub type FnMergePipelineCaches = extern "system" fn(
    device: Device,
    dst_cache: PipelineCache,
    src_cache_count: u32,
    src_caches: *const PipelineCache,
) -> Result;

pub type FnCreateGraphicsPipelines = extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    create_infos: *const GraphicsPipelineCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    pipelines: *mut Pipeline,
) -> Result;

pub type FnCreateComputePipelines = extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    create_infos: *const ComputePipelineCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    pipelines: *mut Pipeline,
) -> Result;

pub type FnDestroyPipeline =
    extern "system" fn(device: Device, pipeline: Pipeline, allocator: Option<&AllocationCallbacks>);

pub type FnCreatePipelineLayout = extern "system" fn(
    device: Device,
    create_info: &PipelineLayoutCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    pipeline_layout: &mut PipelineLayout,
) -> Result;

pub type FnDestroyPipelineLayout = extern "system" fn(
    device: Device,
    pipeline_layout: PipelineLayout,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnCreateSampler = extern "system" fn(
    device: Device,
    create_info: &SamplerCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    sampler: &mut Sampler,
) -> Result;

pub type FnDestroySampler =
    extern "system" fn(device: Device, sampler: Sampler, allocator: Option<&AllocationCallbacks>);

pub type FnCreateDescriptorSetLayout = extern "system" fn(
    device: Device,
    create_info: &DescriptorSetLayoutCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    set_layout: &mut DescriptorSetLayout,
) -> Result;

pub type FnDestroyDescriptorSetLayout = extern "system" fn(
    device: Device,
    descriptor_set_layout: DescriptorSetLayout,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnCreateDescriptorPool = extern "system" fn(
    device: Device,
    create_info: &DescriptorPoolCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    descriptor_pool: &mut DescriptorPool,
) -> Result;

pub type FnDestroyDescriptorPool = extern "system" fn(
    device: Device,
    descriptor_pool: DescriptorPool,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnResetDescriptorPool = extern "system" fn(
    device: Device,
    descriptor_pool: DescriptorPool,
    flags: DescriptorPoolResetFlags,
) -> Result;

pub type FnAllocateDescriptorSets = extern "system" fn(
    device: Device,
    allocate_info: &DescriptorSetAllocateInfo,
    descriptor_sets: *mut DescriptorSet,
) -> Result;

pub type FnFreeDescriptorSets = extern "system" fn(
    device: Device,
    descriptor_pool: DescriptorPool,
    descriptor_set_count: u32,
    descriptor_sets: *const DescriptorSet,
) -> Result;

pub type FnUpdateDescriptorSets = extern "system" fn(
    device: Device,
    descriptor_write_count: u32,
    descriptor_writes: *const WriteDescriptorSet,
    descriptor_copy_count: u32,
    descriptor_copies: *const CopyDescriptorSet,
);

pub type FnCreateFramebuffer = extern "system" fn(
    device: Device,
    create_info: &FramebufferCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    framebuffer: &mut Framebuffer,
) -> Result;

pub type FnDestroyFramebuffer = extern "system" fn(
    device: Device,
    framebuffer: Framebuffer,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnCreateRenderPass = extern "system" fn(
    device: Device,
    create_info: &RenderPassCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    render_pass: &mut RenderPass,
) -> Result;

pub type FnDestroyRenderPass = extern "system" fn(
    device: Device,
    render_pass: RenderPass,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnCreateCommandPool = extern "system" fn(
    device: Device,
    create_info: &CommandPoolCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    command_pool: &mut CommandPool,
) -> Result;

pub type FnDestroyCommandPool = extern "system" fn(
    device: Device,
    command_pool: CommandPool,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnResetCommandPool = extern "system" fn(
    device: Device,
    command_pool: CommandPool,
    flags: CommandPoolResetFlags,
) -> Result;

pub type FnAllocateCommandBuffers = extern "system" fn(
    device: Device,
    allocate_info: &CommandBufferAllocateInfo,
    command_buffers: *mut CommandBuffer,
) -> Result;

pub type FnFreeCommandBuffers = extern "system" fn(
    device: Device,
    command_pool: CommandPool,
    command_buffer_count: u32,
    command_buffers: *const CommandBuffer,
);

pub type FnBeginCommandBuffer = extern "system" fn(
    command_buffer: CommandBuffer,
    begin_info: &CommandBufferBeginInfo,
) -> Result;

pub type FnEndCommandBuffer = extern "system" fn(command_buffer: CommandBuffer) -> Result;

pub type FnResetCommandBuffer =
    extern "system" fn(command_buffer: CommandBuffer, flags: CommandBufferResetFlags) -> Result;

pub type FnCmdBindPipeline = extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    pipeline: Pipeline,
);

pub type FnCmdSetViewport = extern "system" fn(
    command_buffer: CommandBuffer,
    first_viewport: u32,
    viewport_count: u32,
    viewports: *const Viewport,
);

pub type FnCmdSetScissor = extern "system" fn(
    command_buffer: CommandBuffer,
    first_scissor: u32,
    scissor_count: u32,
    scissors: *const Rect2d,
);

pub type FnCmdSetLineWidth = extern "system" fn(command_buffer: CommandBuffer, line_width: f32);

pub type FnCmdSetDepthBias = extern "system" fn(
    command_buffer: CommandBuffer,
    depth_bias_constant_factor: f32,
    depth_bias_clamp: f32,
    depth_bias_slope_factor: f32,
);

pub type FnCmdSetBlendConstants =
    extern "system" fn(command_buffer: CommandBuffer, blend_constants: [f32; 4]);

pub type FnCmdSetDepthBounds =
    extern "system" fn(command_buffer: CommandBuffer, min_depth_bounds: f32, max_depth_bounds: f32);

pub type FnCmdSetStencilCompareMask = extern "system" fn(
    command_buffer: CommandBuffer,
    face_mask: StencilFaceFlags,
    compare_mask: u32,
);

pub type FnCmdSetStencilWriteMask =
    extern "system" fn(command_buffer: CommandBuffer, face_mask: StencilFaceFlags, write_mask: u32);

pub type FnCmdSetStencilReference =
    extern "system" fn(command_buffer: CommandBuffer, face_mask: StencilFaceFlags, reference: u32);

pub type FnCmdBindDescriptorSets = extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    layout: PipelineLayout,
    first_set: u32,
    descriptor_set_count: u32,
    descriptor_sets: *const DescriptorSet,
    dynamic_offset_count: u32,
    dynamic_offsets: *const u32,
);

pub type FnCmdBindIndexBuffer = extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    index_type: IndexType,
);

pub type FnCmdBindVertexBuffers = extern "system" fn(
    command_buffer: CommandBuffer,
    first_binding: u32,
    binding_count: u32,
    buffers: *const Buffer,
    offsets: *const DeviceSize,
);

pub type FnCmdDraw = extern "system" fn(
    command_buffer: CommandBuffer,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
);

pub type FnCmdDrawIndexed = extern "system" fn(
    command_buffer: CommandBuffer,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    vertex_offset: i32,
    first_instance: u32,
);

pub type FnCmdDrawIndirect = extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    draw_count: u32,
    stride: u32,
);

pub type FnCmdDrawIndexedIndirect = extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    draw_count: u32,
    stride: u32,
);

pub type FnCmdDispatch = extern "system" fn(
    command_buffer: CommandBuffer,
    group_count_x: u32,
    group_count_y: u32,
    group_count_z: u32,
);

pub type FnCmdDispatchIndirect =
    extern "system" fn(command_buffer: CommandBuffer, buffer: Buffer, offset: DeviceSize);

pub type FnCmdCopyBuffer = extern "system" fn(
    command_buffer: CommandBuffer,
    src_buffer: Buffer,
    dst_buffer: Buffer,
    region_count: u32,
    regions: *const BufferCopy,
);

pub type FnCmdCopyImage = extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    regions: *const ImageCopy,
);

pub type FnCmdBlitImage = extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    regions: *const ImageBlit,
    filter: Filter,
);

pub type FnCmdCopyBufferToImage = extern "system" fn(
    command_buffer: CommandBuffer,
    src_buffer: Buffer,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    regions: *const BufferImageCopy,
);

pub type FnCmdCopyImageToBuffer = extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_buffer: Buffer,
    region_count: u32,
    regions: *const BufferImageCopy,
);

pub type FnCmdUpdateBuffer = extern "system" fn(
    command_buffer: CommandBuffer,
    dst_buffer: Buffer,
    dst_offset: DeviceSize,
    data_size: DeviceSize,
    data: *const c_void,
);

pub type FnCmdFillBuffer = extern "system" fn(
    command_buffer: CommandBuffer,
    dst_buffer: Buffer,
    dst_offset: DeviceSize,
    size: DeviceSize,
    data: u32,
);

pub type FnCmdClearColorImage = extern "system" fn(
    command_buffer: CommandBuffer,
    image: Image,
    image_layout: ImageLayout,
    color: &ClearColorValue,
    range_count: u32,
    ranges: *const ImageSubresourceRange,
);

pub type FnCmdClearDepthStencilImage = extern "system" fn(
    command_buffer: CommandBuffer,
    image: Image,
    image_layout: ImageLayout,
    depth_stencil: &ClearDepthStencilValue,
    range_count: u32,
    ranges: *const ImageSubresourceRange,
);

pub type FnCmdClearAttachments = extern "system" fn(
    command_buffer: CommandBuffer,
    attachment_count: u32,
    attachments: *const ClearAttachment,
    rect_count: u32,
    rects: *const ClearRect,
);

pub type FnCmdResolveImage = extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    regions: *const ImageResolve,
);

pub type FnCmdSetEvent =
    extern "system" fn(command_buffer: CommandBuffer, event: Event, stage_mask: PipelineStageFlags);

pub type FnCmdResetEvent =
    extern "system" fn(command_buffer: CommandBuffer, event: Event, stage_mask: PipelineStageFlags);

pub type FnCmdWaitEvents = extern "system" fn(
    command_buffer: CommandBuffer,
    event_count: u32,
    events: *const Event,
    src_stage_mask: PipelineStageFlags,
    dst_stage_mask: PipelineStageFlags,
    memory_barrier_count: u32,
    memory_barriers: *const MemoryBarrier,
    buffer_memory_barrier_count: u32,
    buffer_memory_barriers: *const BufferMemoryBarrier,
    image_memory_barrier_count: u32,
    image_memory_barriers: *const ImageMemoryBarrier,
);

pub type FnCmdPipelineBarrier = extern "system" fn(
    command_buffer: CommandBuffer,
    src_stage_mask: PipelineStageFlags,
    dst_stage_mask: PipelineStageFlags,
    dependency_flags: DependencyFlags,
    memory_barrier_count: u32,
    memory_barriers: *const MemoryBarrier,
    buffer_memory_barrier_count: u32,
    buffer_memory_barriers: *const BufferMemoryBarrier,
    image_memory_barrier_count: u32,
    image_memory_barriers: *const ImageMemoryBarrier,
);

pub type FnCmdPipelineBarrier2 =
    extern "system" fn(command_buffer: CommandBuffer, dependency_info: &DependencyInfo);

pub type FnCmdWaitEvents2 = extern "system" fn(
    command_buffer: CommandBuffer,
    event_count: u32,
    events: *const Event,
    dependency_infos: *const DependencyInfo,
);

pub type FnCmdSetEvent2 = extern "system" fn(
    command_buffer: CommandBuffer,
    event: Event,
    dependency_info: &DependencyInfo,
);

pub type FnCmdBeginQuery = extern "system" fn(
    command_buffer: CommandBuffer,
    query_pool: QueryPool,
    query: u32,
    flags: QueryControlFlags,
);

pub type FnCmdEndQuery =
    extern "system" fn(command_buffer: CommandBuffer, query_pool: QueryPool, query: u32);

pub type FnCmdResetQueryPool = extern "system" fn(
    command_buffer: CommandBuffer,
    query_pool: QueryPool,
    first_query: u32,
    query_count: u32,
);

pub type FnCmdSetViewportWithCount = extern "system" fn(
    command_buffer: CommandBuffer,
    viewport_count: u32,
    viewports: *const Viewport,
);

pub type FnCmdSetScissorWithCount =
    extern "system" fn(command_buffer: CommandBuffer, scissors_count: u32, scissors: *const Rect2d);

pub type FnCreateBuffer = extern "system" fn(
    device: Device,
    create_info: &BufferCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    buffer: &mut Buffer,
) -> Result;

pub type FnDestroyBuffer =
    extern "system" fn(device: Device, buffer: Buffer, allocator: Option<&AllocationCallbacks>);

pub type FnCreateBufferView = extern "system" fn(
    device: Device,
    create_info: &BufferViewCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    view: &mut BufferView,
) -> Result;

pub type FnDestroyBufferView = extern "system" fn(
    device: Device,
    buffer_view: BufferView,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnCreateImage = extern "system" fn(
    device: Device,
    create_info: &ImageCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    image: &mut Image,
) -> Result;

pub type FnDestroyImage =
    extern "system" fn(device: Device, image: Image, allocator: Option<&AllocationCallbacks>);

pub type FnGetImageSubresourceLayout = extern "system" fn(
    device: Device,
    image: Image,
    subresource: &ImageSubresource,
    layout: &mut SubresourceLayout,
);

pub type FnGetImageMemoryRequirements2 = extern "system" fn(
    device: Device,
    info: &ImageMemoryRequirementsInfo2,
    memory_requirements: &mut MemoryRequirements2,
);

pub type FnBindImageMemory2 =
    extern "system" fn(device: Device, bind_info_count: u32, *const BindImageMemoryInfo);

pub type FnGetBufferMemoryRequirements2 = extern "system" fn(
    device: Device,
    info: &BufferMemoryRequirementsInfo2,
    memory_requirements: &mut MemoryRequirements2,
);

pub type FnBindBufferMemory2 =
    extern "system" fn(device: Device, bind_info_count: u32, *const BindBufferMemoryInfo);

pub type FnCmdWriteTimestamp = extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_stage: PipelineStageFlags,
    query_pool: QueryPool,
    query: u32,
);

pub type FnCmdCopyQueryPoolResults = extern "system" fn(
    command_buffer: CommandBuffer,
    query_pool: QueryPool,
    first_query: u32,
    query_count: u32,
    dst_buffer: Buffer,
    dst_offset: DeviceSize,
    stride: DeviceSize,
    flags: QueryResultFlags,
);

pub type FnCmdPushConstants = extern "system" fn(
    command_buffer: CommandBuffer,
    layout: PipelineLayout,
    stage_flags: ShaderStageFlags,
    offset: u32,
    size: u32,
    values: *const c_void,
);

pub type FnCmdBeginRenderPass = extern "system" fn(
    command_buffer: CommandBuffer,
    render_pass_begin: &RenderPassBeginInfo,
    contents: SubpassContents,
);

pub type FnCmdNextSubpass =
    extern "system" fn(command_buffer: CommandBuffer, contents: SubpassContents);

pub type FnCmdEndRenderPass = extern "system" fn(command_buffer: CommandBuffer);

pub type FnCmdExecuteCommands = extern "system" fn(
    command_buffer: CommandBuffer,
    command_buffer_count: u32,
    command_buffers: *const CommandBuffer,
);

pub type FnCmdBeginRendering =
    extern "system" fn(command_buffer: CommandBuffer, rendering_info: &RenderingInfo);

pub type FnCmdEndRendering = extern "system" fn(command_buffer: CommandBuffer);

pub type FnCreateFence = extern "system" fn(
    device: Device,
    create_info: &FenceCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    fence: &mut Fence,
) -> Result;

pub type FnDestroyFence =
    extern "system" fn(device: Device, fence: Fence, allocator: Option<&AllocationCallbacks>);

pub type FnResetFences =
    extern "system" fn(device: Device, fence_count: u32, fences: *const Fence) -> Result;

pub type FnGetFenceStatus = extern "system" fn(device: Device, fence: Fence) -> Result;

pub type FnWaitForFences = extern "system" fn(
    device: Device,
    fence_count: u32,
    fences: *const Fence,
    wait_all: Bool32,
    timeout: u64,
) -> Result;

pub type FnInvalidateMappedMemoryRanges = extern "system" fn(
    device: Device,
    memory_range_count: u32,
    memory_ranges: *const MappedMemoryRange,
) -> Result;

pub type FnCreateSemaphore = extern "system" fn(
    device: Device,
    create_info: &SemaphoreCreateInfo,
    allocator: Option<&AllocationCallbacks>,
    semaphore: &mut Semaphore,
) -> Result;

pub type FnDestroySemaphore = extern "system" fn(
    device: Device,
    semaphore: Semaphore,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnGetSemaphoreCounterValue =
    extern "system" fn(device: Device, semaphore: Semaphore, value: &mut u64) -> Result;

pub type FnWaitSemaphores =
    extern "system" fn(device: Device, wait_info: &SemaphoreWaitInfo, timeout: u64) -> Result;

pub type FnSignalSemaphore =
    extern "system" fn(device: Device, signal_info: &SemaphoreSignalInfo) -> Result;

pub type FnCreateSwapchainKHR = extern "system" fn(
    device: Device,
    create_info: &SwapchainCreateInfoKHR,
    allocator: Option<&AllocationCallbacks>,
    swapchain: &mut SwapchainKHR,
) -> Result;

pub type FnDestroySwapchainKHR = extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    allocator: Option<&AllocationCallbacks>,
);

pub type FnGetSwapchainImagesKHR = extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    swapchain_image_count: &mut u32,
    swapchain_images: *mut Image,
) -> Result;

pub type FnAcquireNextImageKHR = extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    timeout: u64,
    semaphore: Semaphore,
    fence: Fence,
    image_index: &mut u32,
) -> Result;

pub type FnQueuePresentKHR =
    extern "system" fn(queue: Queue, present_info: &PresentInfoKHR) -> Result;

pub type FnAcquireNextImage2KHR = extern "system" fn(
    device: Device,
    acquire_info: &AcquireNextImageInfoKHR,
    image_index: &mut u32,
) -> Result;
