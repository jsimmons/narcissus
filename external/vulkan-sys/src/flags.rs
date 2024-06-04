#[repr(C)]
pub struct InstanceCreateFlags(u32);

#[repr(C)]
pub struct XcbSurfaceCreateFlagsKHR(u32);

#[repr(C)]
pub struct XlibSurfaceCreateFlagsKHR(u32);

#[repr(C)]
pub struct WaylandSurfaceCreateFlagsKHR(u32);

#[repr(C)]
#[derive(Debug)]
pub struct DeviceCreateFlags(u32);

#[repr(C)]
#[derive(Debug)]
pub struct DeviceQueueCreateFlags(u32);
impl DeviceQueueCreateFlags {
    pub const PROTECTED: Self = Self(1);
}

#[repr(C)]
#[derive(Debug)]
pub struct SurfaceTransformFlagsKHR(u32);
impl SurfaceTransformFlagsKHR {
    pub const IDENTITY: Self = Self(1);
    pub const ROTATE_90: Self = Self(2);
    pub const ROTATE_180: Self = Self(4);
    pub const ROTATE_270: Self = Self(8);
    pub const HORIZONTAL_MIRROR: Self = Self(16);
    pub const HORIZONTAL_MIRROR_ROTATE_90: Self = Self(32);
    pub const HORIZONTAL_MIRROR_ROTATE_180: Self = Self(64);
    pub const HORIZONTAL_MIRROR_ROTATE_270: Self = Self(128);
    pub const INHERIT: Self = Self(256);
}

#[repr(C)]
#[derive(Debug)]
pub struct SwapchainCreateFlagsKHR(u32);
impl SwapchainCreateFlagsKHR {
    /// Allow images with VK_IMAGE_CREATE_SPLIT_INSTANCE_BIND_REGIONS
    pub const SPLIT_INSTANCE_BIND_REGIONS: Self = Self(1);
    /// Swapchain is protected
    pub const PROTECTED: Self = Self(2);
    pub const MUTABLE_FORMAT: Self = Self(4);
}

#[repr(C)]
#[derive(Debug)]
pub struct CompositeAlphaFlagsKHR(u32);
impl CompositeAlphaFlagsKHR {
    pub const OPAQUE: Self = Self(1);
    pub const PRE_MULTIPLIED: Self = Self(2);
    pub const POST_MULTIPLIED: Self = Self(4);
    pub const INHERIT: Self = Self(8);
}

#[repr(C)]
#[derive(Debug)]
pub struct SampleCountFlags(u32);
impl SampleCountFlags {
    /// Sample count 1 supported
    pub const SAMPLE_COUNT_1: Self = Self(1);
    /// Sample count 2 supported
    pub const SAMPLE_COUNT_2: Self = Self(2);
    /// Sample count 4 supported
    pub const SAMPLE_COUNT_4: Self = Self(4);
    /// Sample count 8 supported
    pub const SAMPLE_COUNT_8: Self = Self(8);
    /// Sample count 16 supported
    pub const SAMPLE_COUNT_16: Self = Self(16);
    /// Sample count 32 supported
    pub const SAMPLE_COUNT_32: Self = Self(32);
    /// Sample count 64 supported
    pub const SAMPLE_COUNT_64: Self = Self(64);
}

#[repr(C)]
pub struct MemoryPropertyFlags(u32);
impl MemoryPropertyFlags {
    /// If otherwise stated, then allocate memory on device
    pub const DEVICE_LOCAL: Self = Self(1);
    /// Memory is mappable by host
    pub const HOST_VISIBLE: Self = Self(2);
    /// Memory will have i/o coherency. If not set, application may need to use vkFlushMappedMemoryRanges and vkInvalidateMappedMemoryRanges to flush/invalidate host cache
    pub const HOST_COHERENT: Self = Self(4);
    /// Memory will be cached by the host
    pub const HOST_CACHED: Self = Self(8);
    /// Memory may be allocated by the driver when it is required
    pub const LAZILY_ALLOCATED: Self = Self(16);
    /// Memory is protected
    pub const PROTECTED: Self = Self(32);
    pub const DEVICE_COHERENT_AMD: Self = Self(64);
    pub const DEVICE_UNCACHED_AMD: Self = Self(128);
    pub const RDMA_CAPABLE_NV: Self = Self(256);
}

#[repr(C)]
pub struct MemoryHeapFlags(u32);
impl MemoryHeapFlags {
    /// If set, heap represents device memory
    pub const DEVICE_LOCAL: Self = Self(1);
    /// If set, heap allocations allocate multiple instances by default
    pub const MULTI_INSTANCE: Self = Self(2);
}

#[repr(C)]
pub struct MemoryMapFlags(u32);

#[repr(C)]
pub struct MemoryAllocateFlags(u32);

impl MemoryAllocateFlags {
    pub const DEVICE_MASK: Self = Self(0x00000001);
    pub const DEVICE_ADDRESS_BIT: Self = Self(0x00000002);
    pub const DEVICE_ADDRESS_CAPTURE_REPLAY_BIT: Self = Self(0x00000004);
}

#[repr(C)]
pub struct ImageAspectFlags(u32);
impl ImageAspectFlags {
    pub const COLOR: Self = Self(1);
    pub const DEPTH: Self = Self(2);
    pub const STENCIL: Self = Self(4);
    pub const METADATA: Self = Self(8);
    pub const PLANE_0: Self = Self(16);
    pub const PLANE_1: Self = Self(32);
    pub const PLANE_2: Self = Self(64);
    pub const MEMORY_PLANE_0_EXT: Self = Self(128);
    pub const MEMORY_PLANE_1_EXT: Self = Self(256);
    pub const MEMORY_PLANE_2_EXT: Self = Self(512);
    pub const MEMORY_PLANE_3_EXT: Self = Self(1024);
}

#[repr(C)]
pub struct ImageViewCreateFlags(u32);
impl ImageViewCreateFlags {
    pub const FRAGMENT_DENSITY_MAP_DYNAMIC_EXT: Self = Self(1);
    pub const FRAGMENT_DENSITY_MAP_DEFERRED_EXT: Self = Self(2);
}

#[repr(C)]
pub struct SparseMemoryBindFlags(u32);
impl SparseMemoryBindFlags {
    /// Operation binds resource metadata to memory
    pub const METADATA: Self = Self(1);
}

#[repr(C)]
pub struct SparseImageFormatFlags(u32);
impl SparseImageFormatFlags {
    /// Image uses a single mip tail region for all array layers
    pub const SINGLE_MIPTAIL: Self = Self(1);
    /// Image requires mip level dimensions to be an integer multiple of the sparse image block dimensions for non-tail mip levels.
    pub const ALIGNED_MIP_SIZE: Self = Self(2);
    /// Image uses a non-standard sparse image block dimensions
    pub const NONSTANDARD_BLOCK_SIZE: Self = Self(4);
}

#[repr(C)]
#[derive(Debug)]
pub struct QueueFlags(u32);
impl QueueFlags {
    /// Queue supports graphics operations
    pub const GRAPHICS: Self = Self(1);
    /// Queue supports compute operations
    pub const COMPUTE: Self = Self(2);
    /// Queue supports transfer operations
    pub const TRANSFER: Self = Self(4);
    /// Queue supports sparse resource memory management operations
    pub const SPARSE_BINDING: Self = Self(8);
    /// Queues may support protected operations
    pub const PROTECTED: Self = Self(16);
    pub const VIDEO_DECODE_KHR: Self = Self(32);
    pub const VIDEO_ENCODE_KHR: Self = Self(64);
}

#[repr(C)]
pub struct ImageUsageFlags(u32);
impl ImageUsageFlags {
    /// Can be used as a source of transfer operations
    pub const TRANSFER_SRC: Self = Self(1);
    /// Can be used as a destination of transfer operations
    pub const TRANSFER_DST: Self = Self(2);
    /// Can be sampled from (SAMPLED_IMAGE and COMBINED_IMAGE_SAMPLER descriptor types)
    pub const SAMPLED: Self = Self(4);
    /// Can be used as storage image (STORAGE_IMAGE descriptor type)
    pub const STORAGE: Self = Self(8);
    /// Can be used as framebuffer color attachment
    pub const COLOR_ATTACHMENT: Self = Self(16);
    /// Can be used as framebuffer depth/stencil attachment
    pub const DEPTH_STENCIL_ATTACHMENT: Self = Self(32);
    /// Image data not needed outside of rendering
    pub const TRANSIENT_ATTACHMENT: Self = Self(64);
    /// Can be used as framebuffer input attachment
    pub const INPUT_ATTACHMENT: Self = Self(128);
    pub const VIDEO_DECODE_DST_KHR: Self = Self(1024);
    pub const VIDEO_DECODE_SRC_KHR: Self = Self(2048);
    pub const VIDEO_DECODE_DPB_KHR: Self = Self(4096);
    pub const FRAGMENT_DENSITY_MAP_EXT: Self = Self(512);
    pub const FRAGMENT_SHADING_RATE_ATTACHMENT_KHR: Self = Self(256);
    pub const VIDEO_ENCODE_DST_KHR: Self = Self(8192);
    pub const VIDEO_ENCODE_SRC_KHR: Self = Self(16384);
    pub const VIDEO_ENCODE_DPB_KHR: Self = Self(32768);
    pub const INVOCATION_MASK_HUAWEI: Self = Self(262144);
}

#[repr(C)]
pub struct ImageCreateFlags(u32);

impl ImageCreateFlags {
    /// Image should support sparse backing
    pub const SPARSE_BINDING: Self = Self(1);
    /// Image should support sparse backing with partial residency
    pub const SPARSE_RESIDENCY: Self = Self(2);
    /// Image should support constant data access to physical memory ranges mapped into multiple locations of sparse images
    pub const SPARSE_ALIASED: Self = Self(4);
    /// Allows image views to have different format than the base image
    pub const MUTABLE_FORMAT: Self = Self(8);
    /// Allows creating image views with cube type from the created image
    pub const CUBE_COMPATIBLE: Self = Self(16);
    pub const ALIAS: Self = Self(1024);
    /// Allows using VkBindImageMemoryDeviceGroupInfo::pSplitInstanceBindRegions when binding memory to the image
    pub const SPLIT_INSTANCE_BIND_REGIONS: Self = Self(64);
    /// The 3D image can be viewed as a 2D or 2D array image
    pub const IMAGE_CREATE_2D_ARRAY_COMPATIBLE: Self = Self(32);
    pub const BLOCK_TEXEL_VIEW_COMPATIBLE: Self = Self(128);
    pub const EXTENDED_USAGE: Self = Self(256);
    /// Image requires protected memory
    pub const PROTECTED: Self = Self(2048);
    pub const DISJOINT: Self = Self(512);
    pub const CORNER_SAMPLED_NV: Self = Self(8192);
    pub const SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_EXT: Self = Self(4096);
    pub const SUBSAMPLED_EXT: Self = Self(16384);
}

#[repr(C)]
pub struct FormatFeatureFlags(u32);

impl FormatFeatureFlags {
    ///   Format can be used for sampled images (SAMPLED_IMAGE and COMBINED_IMAGE_SAMPLER descriptor types)
    pub const SAMPLED_IMAGE: Self = Self(1);
    ///   Format can be used for storage images (STORAGE_IMAGE descriptor type)
    pub const STORAGE_IMAGE: Self = Self(2);
    ///   Format supports atomic operations in case it is used for storage images
    pub const STORAGE_IMAGE_ATOMIC: Self = Self(4);
    ///   Format can be used for uniform texel buffers (TBOs)
    pub const UNIFORM_TEXEL_BUFFER: Self = Self(8);
    ///   Format can be used for storage texel buffers (IBOs)
    pub const STORAGE_TEXEL_BUFFER: Self = Self(16);
    ///   Format supports atomic operations in case it is used for storage texel buffers
    pub const STORAGE_TEXEL_BUFFER_ATOMIC: Self = Self(32);
    ///   Format can be used for vertex buffers (VBOs)
    pub const VERTEX_BUFFER: Self = Self(64);
    ///   Format can be used for color attachment images
    pub const COLOR_ATTACHMENT: Self = Self(128);
    ///   Format supports blending in case it is used for color attachment images
    pub const COLOR_ATTACHMENT_BLEND: Self = Self(256);
    ///   Format can be used for depth/stencil attachment images
    pub const DEPTH_STENCIL_ATTACHMENT: Self = Self(512);
    ///   Format can be used as the source image of blits with vkCmdBlitImage
    pub const BLIT_SRC: Self = Self(1024);
    ///   Format can be used as the destination image of blits with vkCmdBlitImage
    pub const BLIT_DST: Self = Self(2048);
    ///   Format can be filtered with VK_FILTER_LINEAR when being sampled
    pub const SAMPLED_IMAGE_FILTER_LINEAR: Self = Self(4096);
    ///   Format can be used as the source image of image transfer commands
    pub const TRANSFER_SRC: Self = Self(16384);
    ///   Format can be used as the destination image of image transfer commands
    pub const TRANSFER_DST: Self = Self(32768);
    ///   Format can have midpoint rather than cosited chroma samples
    pub const MIDPOINT_CHROMA_SAMPLES: Self = Self(131072);
    ///   Format can be used with linear filtering whilst color conversion is enabled
    pub const SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER: Self = Self(262144);
    ///   Format can have different chroma, min and mag filters
    pub const SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER: Self = Self(524288);
    pub const SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT: Self = Self(1048576);
    pub const SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE: Self =
        Self(2097152);
    ///   Format supports disjoint planes
    pub const DISJOINT: Self = Self(4194304);
    ///   Format can have cosited rather than midpoint chroma samples
    pub const COSITED_CHROMA_SAMPLES: Self = Self(8388608);
    ///   Format can be used with min/max reduction filtering
    pub const SAMPLED_IMAGE_FILTER_MINMAX: Self = Self(65536);
    ///   Format can be filtered with VK_FILTER_CUBIC_IMG when being sampled
    pub const SAMPLED_IMAGE_FILTER_CUBIC_IMG: Self = Self(8192);
    pub const VIDEO_DECODE_OUTPUT_KHR: Self = Self(33554432);
    pub const VIDEO_DECODE_DPB_KHR: Self = Self(67108864);
    pub const ACCELERATION_STRUCTURE_VERTEX_BUFFER_KHR: Self = Self(536870912);
    pub const DISJOINT_KHR: Self = Self::DISJOINT;
    pub const FRAGMENT_DENSITY_MAP_EXT: Self = Self(16777216);
    pub const FRAGMENT_SHADING_RATE_ATTACHMENT_KHR: Self = Self(1073741824);
    pub const VIDEO_ENCODE_INPUT_KHR: Self = Self(134217728);
    pub const VIDEO_ENCODE_DPB_KHR: Self = Self(268435456);
}

#[repr(C)]
pub struct PipelineStageFlags(u32);
impl PipelineStageFlags {
    /// Before subsequent commands are processed
    pub const TOP_OF_PIPE: Self = Self(1);
    /// Draw/DispatchIndirect command fetch
    pub const DRAW_INDIRECT: Self = Self(2);
    /// Vertex/index fetch
    pub const VERTEX_INPUT: Self = Self(4);
    /// Vertex shading
    pub const VERTEX_SHADER: Self = Self(8);
    /// Tessellation control shading
    pub const TESSELLATION_CONTROL_SHADER: Self = Self(16);
    /// Tessellation evaluation shading
    pub const TESSELLATION_EVALUATION_SHADER: Self = Self(32);
    /// Geometry shading
    pub const GEOMETRY_SHADER: Self = Self(64);
    /// Fragment shading
    pub const FRAGMENT_SHADER: Self = Self(128);
    /// Early fragment (depth and stencil) tests
    pub const EARLY_FRAGMENT_TESTS: Self = Self(256);
    /// Late fragment (depth and stencil) tests
    pub const LATE_FRAGMENT_TESTS: Self = Self(512);
    /// Color attachment writes
    pub const COLOR_ATTACHMENT_OUTPUT: Self = Self(1024);
    /// Compute shading
    pub const COMPUTE_SHADER: Self = Self(2048);
    /// Transfer/copy operations
    pub const TRANSFER: Self = Self(4096);
    /// After previous commands have completed
    pub const BOTTOM_OF_PIPE: Self = Self(8192);
    /// Indicates host (CPU) is a source/sink of the dependency
    pub const HOST: Self = Self(16384);
    /// All stages of the graphics pipeline
    pub const ALL_GRAPHICS: Self = Self(32768);
    /// All stages supported on the queue
    pub const ALL_COMMANDS: Self = Self(65536);
    pub const TRANSFORM_FEEDBACK_EXT: Self = Self(16777216);
    /// A pipeline stage for conditional rendering predicate fetch
    pub const CONDITIONAL_RENDERING_EXT: Self = Self(262144);
    pub const ACCELERATION_STRUCTURE_BUILD_KHR: Self = Self(33554432);
    pub const RAY_TRACING_SHADER_KHR: Self = Self(2097152);
    pub const TASK_SHADER_NV: Self = Self(524288);
    pub const MESH_SHADER_NV: Self = Self(1048576);
    pub const FRAGMENT_DENSITY_PROCESS_EXT: Self = Self(8388608);
    pub const FRAGMENT_SHADING_RATE_ATTACHMENT_KHR: Self = Self(4194304);
    pub const COMMAND_PREPROCESS_NV: Self = Self(131072);
}

#[repr(C)]
pub struct CommandPoolCreateFlags(u32);
impl CommandPoolCreateFlags {
    /// Command buffers have a short lifetime
    pub const TRANSIENT: Self = Self(1);
    /// Command buffers may release their memory individually
    pub const RESET_COMMAND_BUFFER: Self = Self(2);
    /// Command buffers allocated from pool are protected command buffers
    pub const PROTECTED: Self = Self(4);
}

#[repr(C)]
pub struct CommandPoolResetFlags(u32);
impl CommandPoolResetFlags {
    /// Release resources owned by the pool
    pub const RELEASE_RESOURCES: Self = Self(1);
}

#[repr(C)]
pub struct CommandBufferResetFlags(u32);
impl CommandBufferResetFlags {
    /// Release resources owned by the buffer
    pub const RELEASE_RESOURCES: Self = Self(1);
}

#[repr(C)]
pub struct CommandBufferUsageFlags(u32);
impl CommandBufferUsageFlags {
    pub const ONE_TIME_SUBMIT: Self = Self(1);
    pub const RENDER_PASS_CONTINUE: Self = Self(2);
    /// Command buffer may be submitted/executed more than once simultaneously
    pub const SIMULTANEOUS_USE: Self = Self(4);
}

#[repr(C)]
pub struct QueryControlFlags(u32);
impl QueryControlFlags {
    /// Require precise results to be collected by the query
    pub const PRECISE: Self = Self(1);
}

#[repr(C)]
pub struct QueryResultFlags(u32);
impl QueryResultFlags {
    /// Results of the queries are written to the destination buffer as 64-bit values
    pub const QUERY_RESULT_64: Self = Self(1);
    /// Results of the queries are waited on before proceeding with the result copy
    pub const WAIT: Self = Self(2);
    /// Besides the results of the query, the availability of the results is also written
    pub const WITH_AVAILABILITY: Self = Self(4);
    /// Copy the partial results of the query even if the final results are not available
    pub const PARTIAL: Self = Self(8);
    pub const WITH_STATUS_KHR: Self = Self(16);
}

#[repr(C)]
pub struct QueryPipelineStatisticFlags(u32);
impl QueryPipelineStatisticFlags {
    pub const INPUT_ASSEMBLY_VERTICES: Self = Self(1);
    pub const INPUT_ASSEMBLY_PRIMITIVES: Self = Self(2);
    pub const VERTEX_SHADER_INVOCATIONS: Self = Self(4);
    pub const GEOMETRY_SHADER_INVOCATIONS: Self = Self(8);
    pub const GEOMETRY_SHADER_PRIMITIVES: Self = Self(16);
    pub const CLIPPING_INVOCATIONS: Self = Self(32);
    pub const CLIPPING_PRIMITIVES: Self = Self(64);
    pub const FRAGMENT_SHADER_INVOCATIONS: Self = Self(128);
    pub const TESSELLATION_CONTROL_SHADER_PATCHES: Self = Self(256);
    pub const TESSELLATION_EVALUATION_SHADER_INVOCATIONS: Self = Self(512);
    pub const COMPUTE_SHADER_INVOCATIONS: Self = Self(1024);
}

#[repr(C)]
pub struct AttachmentDescriptionFlags(u32);
impl AttachmentDescriptionFlags {
    /// The attachment may alias physical memory of another attachment in the same render pass
    pub const MAY_ALIAS: Self = Self(1);
}

#[repr(C)]
pub struct AccessFlags(u32);
impl AccessFlags {
    /// Controls coherency of indirect command reads
    pub const INDIRECT_COMMAND_READ: Self = Self(1);
    /// Controls coherency of index reads
    pub const INDEX_READ: Self = Self(2);
    /// Controls coherency of vertex attribute reads
    pub const VERTEX_ATTRIBUTE_READ: Self = Self(4);
    /// Controls coherency of uniform buffer reads
    pub const UNIFORM_READ: Self = Self(8);
    /// Controls coherency of input attachment reads
    pub const INPUT_ATTACHMENT_READ: Self = Self(16);
    /// Controls coherency of shader reads
    pub const SHADER_READ: Self = Self(32);
    /// Controls coherency of shader writes
    pub const SHADER_WRITE: Self = Self(64);
    /// Controls coherency of color attachment reads
    pub const COLOR_ATTACHMENT_READ: Self = Self(128);
    /// Controls coherency of color attachment writes
    pub const COLOR_ATTACHMENT_WRITE: Self = Self(256);
    /// Controls coherency of depth/stencil attachment reads
    pub const DEPTH_STENCIL_ATTACHMENT_READ: Self = Self(512);
    /// Controls coherency of depth/stencil attachment writes
    pub const DEPTH_STENCIL_ATTACHMENT_WRITE: Self = Self(1024);
    /// Controls coherency of transfer reads
    pub const TRANSFER_READ: Self = Self(2048);
    /// Controls coherency of transfer writes
    pub const TRANSFER_WRITE: Self = Self(4096);
    /// Controls coherency of host reads
    pub const HOST_READ: Self = Self(8192);
    /// Controls coherency of host writes
    pub const HOST_WRITE: Self = Self(16384);
    /// Controls coherency of memory reads
    pub const MEMORY_READ: Self = Self(32768);
    /// Controls coherency of memory writes
    pub const MEMORY_WRITE: Self = Self(65536);
    pub const TRANSFORM_FEEDBACK_WRITE_EXT: Self = Self(33554432);
    pub const TRANSFORM_FEEDBACK_COUNTER_READ_EXT: Self = Self(67108864);
    pub const TRANSFORM_FEEDBACK_COUNTER_WRITE_EXT: Self = Self(134217728);
    /// read access flag for reading conditional rendering predicate
    pub const CONDITIONAL_RENDERING_READ_EXT: Self = Self(1048576);
    pub const COLOR_ATTACHMENT_READ_NONCOHERENT_EXT: Self = Self(524288);
    pub const ACCELERATION_STRUCTURE_READ_KHR: Self = Self(2097152);
    pub const ACCELERATION_STRUCTURE_WRITE_KHR: Self = Self(4194304);
    pub const FRAGMENT_DENSITY_MAP_READ_EXT: Self = Self(16777216);
    pub const FRAGMENT_SHADING_RATE_ATTACHMENT_READ_KHR: Self = Self(8388608);
    pub const COMMAND_PREPROCESS_READ_NV: Self = Self(131072);
    pub const COMMAND_PREPROCESS_WRITE_NV: Self = Self(262144);
}

#[repr(C)]
pub struct DependencyFlags(u32);
impl DependencyFlags {
    /// Dependency is per pixel region
    pub const BY_REGION: Self = Self(1);
    /// Dependency is across devices
    pub const DEVICE_GROUP: Self = Self(4);
    pub const VIEW_LOCAL: Self = Self(2);
}

#[repr(C)]
pub struct SubpassDescriptionFlags(u32);
impl SubpassDescriptionFlags {
    pub const PER_VIEW_ATTRIBUTES_NVX: Self = Self(1);
    pub const PER_VIEW_POSITION_X_ONLY_NVX: Self = Self(2);
    pub const FRAGMENT_REGION_QCOM: Self = Self(4);
    pub const SHADER_RESOLVE_QCOM: Self = Self(8);
}

#[repr(C)]
pub struct RenderPassCreateFlags(u32);

#[repr(C)]
pub struct FramebufferCreateFlags(u32);

impl FramebufferCreateFlags {
    pub const IMAGELESS: Self = Self(1);
}

#[repr(C)]
pub struct FenceCreateFlags(u32);
impl FenceCreateFlags {
    pub const SIGNALED: Self = Self(1);
}

#[repr(C)]
pub struct SemaphoreCreateFlags(u32);

#[repr(C)]
pub struct ShaderModuleCreateFlags(u32);

#[repr(C)]
pub struct ShaderStageFlags(u32);

impl ShaderStageFlags {
    pub const VERTEX: Self = Self(1);
    pub const TESSELLATION_CONTROL: Self = Self(2);
    pub const TESSELLATION_EVALUATION: Self = Self(4);
    pub const GEOMETRY: Self = Self(8);
    pub const FRAGMENT: Self = Self(16);
    pub const COMPUTE: Self = Self(32);
    pub const ALL_GRAPHICS: Self = Self(0x0000001F);
    pub const ALL: Self = Self(0x7FFFFFFF);
    pub const RAYGEN_KHR: Self = Self(256);
    pub const ANY_HIT_KHR: Self = Self(512);
    pub const CLOSEST_HIT_KHR: Self = Self(1024);
    pub const MISS_KHR: Self = Self(2048);
    pub const INTERSECTION_KHR: Self = Self(4096);
    pub const CALLABLE_KHR: Self = Self(8192);
    pub const TASK_NV: Self = Self(64);
    pub const MESH_NV: Self = Self(128);
    pub const SUBPASS_SHADING_HUAWEI: Self = Self(16384);
}

#[repr(C)]
pub struct DescriptorSetLayoutCreateFlags(u32);
impl DescriptorSetLayoutCreateFlags {
    pub const UPDATE_AFTER_BIND_POOL: Self = Self(2);
    /// Descriptors are pushed via flink:vkCmdPushDescriptorSetKHR
    pub const PUSH_DESCRIPTOR_KHR: Self = Self(1);
    pub const HOST_ONLY_POOL_VALVE: Self = Self(4);
}

#[repr(C)]
pub struct StencilFaceFlags(u32);
impl StencilFaceFlags {
    pub const FRONT: Self = Self(1); // Front face
    pub const BACK: Self = Self(2); // Back face
    pub const FRONT_AND_BACK: Self = Self(0x00000003); // Front and back faces
}

#[repr(C)]
pub struct CullModeFlags(u32);
impl CullModeFlags {
    pub const NONE: Self = Self(0);
    pub const FRONT: Self = Self(1);
    pub const BACK: Self = Self(2);
    pub const FRONT_AND_BACK: Self = Self(0x00000003);
}

#[repr(C)]
pub struct DescriptorPoolCreateFlags(u32);
impl DescriptorPoolCreateFlags {
    pub const FREE_DESCRIPTOR_SET: Self = Self(1); // Descriptor sets may be freed individually
    pub const UPDATE_AFTER_BIND: Self = Self(2);
    pub const HOST_ONLY_VALVE: Self = Self(4);
}

#[repr(C)]
pub struct DescriptorPoolResetFlags(u32);

#[repr(C)]
pub struct SamplerCreateFlags(u32);
impl SamplerCreateFlags {
    pub const SUBSAMPLED_EXT: Self = Self(1);
    pub const SUBSAMPLED_COARSE_RECONSTRUCTION_EXT: Self = Self(2);
}

#[repr(C)]
pub struct PipelineLayoutCreateFlags(u32);

#[repr(C)]
pub struct PipelineCacheCreateFlags(u32);
impl PipelineCacheCreateFlags {
    pub const EXTERNALLY_SYNCHRONIZED_EXT: Self = Self(1);
}

#[repr(C)]
pub struct PipelineDepthStencilStateCreateFlags(u32);

#[repr(C)]
pub struct PipelineDynamicStateCreateFlags(u32);

#[repr(C)]
pub struct PipelineColorBlendStateCreateFlags(u32);

#[repr(C)]
pub struct PipelineMultisampleStateCreateFlags(u32);

#[repr(C)]
pub struct PipelineRasterizationStateCreateFlags(u32);

#[repr(C)]
pub struct PipelineViewportStateCreateFlags(u32);

#[repr(C)]
pub struct PipelineTessellationStateCreateFlags(u32);

#[repr(C)]
pub struct PipelineInputAssemblyStateCreateFlags(u32);

#[repr(C)]
pub struct PipelineVertexInputStateCreateFlags(u32);

#[repr(C)]
pub struct PipelineShaderStageCreateFlags(u32);
impl PipelineShaderStageCreateFlags {
    pub const ALLOW_VARYING_SUBGROUP_SIZE_EXT: Self = Self(1);
    pub const REQUIRE_F_SUBGROUPS_EXT: Self = Self(2);
}

#[repr(C)]
pub struct PipelineCreateFlags(u32);
impl PipelineCreateFlags {
    pub const DISABLE_OPTIMIZATION: Self = Self(1);
    pub const ALLOW_DERIVATIVES: Self = Self(2);
    pub const DERIVATIVE: Self = Self(4);
    pub const VIEW_INDEX_FROM_DEVICE_INDEX: Self = Self(8);
    pub const DISPATCH_BASE: Self = Self(16);
    pub const RAY_TRACING_NO_NULL_ANY_HIT_SHADERS_KHR: Self = Self(16384);
    pub const RAY_TRACING_NO_NULL_CLOSEST_HIT_SHADERS_KHR: Self = Self(32768);
    pub const RAY_TRACING_NO_NULL_MISS_SHADERS_KHR: Self = Self(65536);
    pub const RAY_TRACING_NO_NULL_INTERSECTION_SHADERS_KHR: Self = Self(131072);
    pub const RAY_TRACING_SKIP_TRIANGLES_KHR: Self = Self(4096);
    pub const RAY_TRACING_SKIP_AABBS_KHR: Self = Self(8192);
    pub const RAY_TRACING_SHADER_GROUP_HANDLE_CAPTURE_REPLAY_KHR: Self = Self(524288);
    pub const DEFER_COMPILE_NV: Self = Self(32);
    pub const CAPTURE_STATISTICS_KHR: Self = Self(64);
    pub const CAPTURE_INTERNAL_REPRESENTATIONS_KHR: Self = Self(128);
    pub const INDIRECT_BINDABLE_NV: Self = Self(262144);
    pub const LIBRARY_KHR: Self = Self(2048);
    pub const FAIL_ON_PIPELINE_COMPILE_REQUIRED_EXT: Self = Self(256);
    pub const EARLY_RETURN_ON_FAILURE_EXT: Self = Self(512);
    pub const RAY_TRACING_ALLOW_MOTION_NV: Self = Self(1048576);
}

#[repr(C)]
pub struct ColorComponentFlags(u32);
impl ColorComponentFlags {
    pub const R: Self = Self(1);
    pub const G: Self = Self(2);
    pub const B: Self = Self(4);
    pub const A: Self = Self(8);
}

#[repr(C)]
pub struct BufferUsageFlags(u32);
impl BufferUsageFlags {
    /// Can be used as a source of transfer operations
    pub const TRANSFER_SRC: Self = Self(1);
    /// Can be used as a destination of transfer operations
    pub const TRANSFER_DST: Self = Self(2);
    /// Can be used as TBO
    pub const UNIFORM_TEXEL_BUFFER: Self = Self(4);
    /// Can be used as IBO
    pub const STORAGE_TEXEL_BUFFER: Self = Self(8);
    /// Can be used as UBO
    pub const UNIFORM_BUFFER: Self = Self(16);
    /// Can be used as SSBO
    pub const STORAGE_BUFFER: Self = Self(32);
    /// Can be used as source of fixed-function index fetch (index buffer)
    pub const INDEX_BUFFER: Self = Self(64);
    /// Can be used as source of fixed-function vertex fetch (VBO)
    pub const VERTEX_BUFFER: Self = Self(128);
    /// Can be the source of indirect parameters (e.g. indirect buffer, parameter buffer)
    pub const INDIRECT_BUFFER: Self = Self(256);
    pub const SHADER_DEVICE_ADDRESS: Self = Self(131072);
    pub const VIDEO_DECODE_SRC_KHR: Self = Self(8192);
    pub const VIDEO_DECODE_DST_KHR: Self = Self(16384);
    pub const TRANSFORM_FEEDBACK_BUFFER_EXT: Self = Self(2048);
    pub const TRANSFORM_FEEDBACK_COUNTER_BUFFER_EXT: Self = Self(4096);
    /// Specifies the buffer can be used as predicate in conditional rendering
    pub const CONDITIONAL_RENDERING_EXT: Self = Self(512);
    pub const ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR: Self = Self(524288);
    pub const ACCELERATION_STRUCTURE_STORAGE_KHR: Self = Self(1048576);
    pub const SHADER_BINDING_TABLE_KHR: Self = Self(1024);
    pub const VIDEO_ENCODE_DST_KHR: Self = Self(32768);
    pub const VIDEO_ENCODE_SRC_KHR: Self = Self(65536);
}

#[repr(C)]
pub struct BufferCreateFlags(u32);
impl BufferCreateFlags {
    /// Buffer should support sparse backing
    pub const SPARSE_BINDING: Self = Self(1);
    /// Buffer should support sparse backing with partial residency
    pub const SPARSE_RESIDENCY: Self = Self(2);
    /// Buffer should support constant data access to physical memory ranges mapped into multiple locations of sparse buffers
    pub const SPARSE_ALIASED: Self = Self(4);
    /// Buffer requires protected memory
    pub const PROTECTED: Self = Self(8);
    pub const DEVICE_ADDRESS_CAPTURE_REPLAY: Self = Self(16);
}

#[repr(C)]
pub struct BufferViewCreateFlags(u32);

#[repr(C)]
pub struct SemaphoreWaitFlags(u32);

impl SemaphoreWaitFlags {
    pub const ANY: Self = Self(1);
}

#[repr(C)]
pub struct ResolveModeFlags(u32);

impl ResolveModeFlags {
    pub const RESOLVE_MODE_NONE: Self = Self(0);
    pub const RESOLVE_MODE_SAMPLE_ZERO: Self = Self(0x00000001);
    pub const RESOLVE_MODE_AVERAGE: Self = Self(0x00000002);
    pub const RESOLVE_MODE_MIN: Self = Self(0x00000004);
    pub const RESOLVE_MODE_MAX: Self = Self(0x00000008);
}

#[repr(C)]
pub struct RenderingFlags(u32);

impl RenderingFlags {
    pub const RENDERING_CONTENTS_SECONDARY_COMMAND_BUFFERS: Self = Self(0x00000001);
    pub const RENDERING_SUSPENDING: Self = Self(0x00000002);
    pub const RENDERING_RESUMING: Self = Self(0x00000004);
}

#[repr(C)]
pub struct SubgroupFeatureFlags(u32);

impl SubgroupFeatureFlags {
    pub const BASIC: Self = Self(0x00000001);
    pub const VOTE: Self = Self(0x00000002);
    pub const ARITHMETIC: Self = Self(0x00000004);
    pub const BALLOT: Self = Self(0x00000008);
    pub const SHUFFLE: Self = Self(0x00000010);
    pub const SHUFFLE_RELATIVE: Self = Self(0x00000020);
    pub const CLUSTERED: Self = Self(0x00000040);
    pub const QUAD: Self = Self(0x00000080);
    // Provided by VK_NV_shader_subgroup_partitioned
    pub const PARTITIONED_NV: Self = Self(0x00000100);
}

#[repr(C)]
pub struct SubmitFlags(u32);

impl SubmitFlags {
    pub const PROTECTED: Self = Self(1);
}

#[repr(C)]
pub struct AccessFlags2(u64);

impl AccessFlags2 {
    pub const NONE: Self = Self(0);
    pub const INDIRECT_COMMAND_READ: Self = Self(0x00000001);
    pub const INDEX_READ: Self = Self(0x00000002);
    pub const VERTEX_ATTRIBUTE_READ: Self = Self(0x00000004);
    pub const UNIFORM_READ: Self = Self(0x00000008);
    pub const INPUT_ATTACHMENT_READ: Self = Self(0x00000010);
    pub const SHADER_READ: Self = Self(0x00000020);
    pub const SHADER_WRITE: Self = Self(0x00000040);
    pub const COLOR_ATTACHMENT_READ: Self = Self(0x00000080);
    pub const COLOR_ATTACHMENT_WRITE: Self = Self(0x00000100);
    pub const DEPTH_STENCIL_ATTACHMENT_READ: Self = Self(0x00000200);
    pub const DEPTH_STENCIL_ATTACHMENT_WRITE: Self = Self(0x00000400);
    pub const TRANSFER_READ: Self = Self(0x00000800);
    pub const TRANSFER_WRITE: Self = Self(0x00001000);
    pub const HOST_READ: Self = Self(0x00002000);
    pub const HOST_WRITE: Self = Self(0x00004000);
    pub const MEMORY_READ: Self = Self(0x00008000);
    pub const MEMORY_WRITE: Self = Self(0x00010000);
    pub const SHADER_SAMPLED_READ: Self = Self(0x100000000);
    pub const SHADER_STORAGE_READ: Self = Self(0x200000000);
    pub const SHADER_STORAGE_WRITE: Self = Self(0x400000000);
    // Provided by VK_KHR_synchronization2 with VK_EXT_transform_feedback
    pub const TRANSFORM_FEEDBACK_WRITE_EXT: Self = Self(0x02000000);
    // Provided by VK_KHR_synchronization2 with VK_EXT_transform_feedback
    pub const TRANSFORM_FEEDBACK_COUNTER_READ_EXT: Self = Self(0x04000000);
    // Provided by VK_KHR_synchronization2 with VK_EXT_transform_feedback
    pub const TRANSFORM_FEEDBACK_COUNTER_WRITE_EXT: Self = Self(0x08000000);
    // Provided by VK_KHR_synchronization2 with VK_EXT_conditional_rendering
    pub const CONDITIONAL_RENDERING_READ_EXT: Self = Self(0x00100000);
    // Provided by VK_KHR_synchronization2 with VK_NV_device_generated_commands
    pub const COMMAND_PREPROCESS_READ_NV: Self = Self(0x00020000);
    // Provided by VK_KHR_synchronization2 with VK_NV_device_generated_commands
    pub const COMMAND_PREPROCESS_WRITE_NV: Self = Self(0x00040000);
    // Provided by VK_KHR_fragment_shading_rate with VK_KHR_synchronization2
    pub const FRAGMENT_SHADING_RATE_ATTACHMENT_READ_KHR: Self = Self(0x00800000);
    // Provided by VK_KHR_synchronization2 with VK_NV_shading_rate_image
    pub const SHADING_RATE_IMAGE_READ_NV: Self = Self(0x00800000);
    // Provided by VK_KHR_acceleration_structure with VK_KHR_synchronization2
    pub const ACCELERATION_STRUCTURE_READ_KHR: Self = Self(0x00200000);
    // Provided by VK_KHR_acceleration_structure with VK_KHR_synchronization2
    pub const ACCELERATION_STRUCTURE_WRITE_KHR: Self = Self(0x00400000);
    // Provided by VK_KHR_synchronization2 with VK_NV_ray_tracing
    pub const ACCELERATION_STRUCTURE_READ_NV: Self = Self(0x00200000);
    // Provided by VK_KHR_synchronization2 with VK_NV_ray_tracing
    pub const ACCELERATION_STRUCTURE_WRITE_NV: Self = Self(0x00400000);
    // Provided by VK_KHR_synchronization2 with VK_EXT_fragment_density_map
    pub const FRAGMENT_DENSITY_MAP_READ_EXT: Self = Self(0x01000000);
    // Provided by VK_KHR_synchronization2 with VK_EXT_blend_operation_advanced
    pub const COLOR_ATTACHMENT_READ_NONCOHERENT_EXT: Self = Self(0x00080000);
    // Provided by VK_HUAWEI_invocation_mask
    pub const INVOCATION_MASK_READ_HUAWEI: Self = Self(0x8000000000);
}

#[repr(C)]
pub struct PipelineStageFlags2(u64);

impl PipelineStageFlags2 {
    pub const NONE: Self = Self(0);
    pub const TOP_OF_PIPE: Self = Self(0x00000001);
    pub const DRAW_INDIRECT: Self = Self(0x00000002);
    pub const VERTEX_INPUT: Self = Self(0x00000004);
    pub const VERTEX_SHADER: Self = Self(0x00000008);
    pub const TESSELLATION_CONTROL_SHADER: Self = Self(0x00000010);
    pub const TESSELLATION_EVALUATION_SHADER: Self = Self(0x00000020);
    pub const GEOMETRY_SHADER: Self = Self(0x00000040);
    pub const FRAGMENT_SHADER: Self = Self(0x00000080);
    pub const EARLY_FRAGMENT_TESTS: Self = Self(0x00000100);
    pub const LATE_FRAGMENT_TESTS: Self = Self(0x00000200);
    pub const COLOR_ATTACHMENT_OUTPUT: Self = Self(0x00000400);
    pub const COMPUTE_SHADER: Self = Self(0x00000800);
    pub const ALL_TRANSFER: Self = Self(0x00001000);
    pub const TRANSFER: Self = Self(0x00001000);
    pub const BOTTOM_OF_PIPE: Self = Self(0x00002000);
    pub const HOST: Self = Self(0x00004000);
    pub const ALL_GRAPHICS: Self = Self(0x00008000);
    pub const ALL_COMMANDS: Self = Self(0x00010000);
    pub const COPY: Self = Self(0x100000000);
    pub const RESOLVE: Self = Self(0x200000000);
    pub const BLIT: Self = Self(0x400000000);
    pub const CLEAR: Self = Self(0x800000000);
    pub const INDEX_INPUT: Self = Self(0x1000000000);
    pub const VERTEX_ATTRIBUTE_INPUT: Self = Self(0x2000000000);
    pub const PRE_RASTERIZATION_SHADERS: Self = Self(0x4000000000);
    // Provided by VK_KHR_synchronization2 with VK_EXT_transform_feedback
    pub const TRANSFORM_FEEDBACK_EXT: Self = Self(0x01000000);
    // Provided by VK_KHR_synchronization2 with VK_EXT_conditional_rendering
    pub const CONDITIONAL_RENDERING_EXT: Self = Self(0x00040000);
    // Provided by VK_KHR_synchronization2 with VK_NV_device_generated_commands
    pub const COMMAND_PREPROCESS_NV: Self = Self(0x00020000);
    // Provided by VK_KHR_fragment_shading_rate with VK_KHR_synchronization2
    pub const FRAGMENT_SHADING_RATE_ATTACHMENT_KHR: Self = Self(0x00400000);
    // Provided by VK_KHR_synchronization2 with VK_NV_shading_rate_image
    pub const SHADING_RATE_IMAGE_NV: Self = Self(0x00400000);
    // Provided by VK_KHR_acceleration_structure with VK_KHR_synchronization2
    pub const ACCELERATION_STRUCTURE_BUILD_KHR: Self = Self(0x02000000);
    // Provided by VK_KHR_ray_tracing_pipeline with VK_KHR_synchronization2
    pub const RAY_TRACING_SHADER_KHR: Self = Self(0x00200000);
    // Provided by VK_KHR_synchronization2 with VK_NV_ray_tracing
    pub const RAY_TRACING_SHADER_NV: Self = Self(0x00200000);
    // Provided by VK_KHR_synchronization2 with VK_NV_ray_tracing
    pub const ACCELERATION_STRUCTURE_BUILD_NV: Self = Self(0x02000000);
    // Provided by VK_KHR_synchronization2 with VK_EXT_fragment_density_map
    pub const FRAGMENT_DENSITY_PROCESS_EXT: Self = Self(0x00800000);
    // Provided by VK_KHR_synchronization2 with VK_NV_mesh_shader
    pub const TASK_SHADER_NV: Self = Self(0x00080000);
    // Provided by VK_KHR_synchronization2 with VK_NV_mesh_shader
    pub const MESH_SHADER_NV: Self = Self(0x00100000);
    // Provided by VK_HUAWEI_subpass_shading
    pub const SUBPASS_SHADING_HUAWEI: Self = Self(0x8000000000);
    // Provided by VK_HUAWEI_invocation_mask
    pub const INVOCATION_MASK_HUAWEI: Self = Self(0x10000000000);
}

#[repr(C)]
pub struct PresentScalingFlagsEXT(u32);

impl PresentScalingFlagsEXT {
    pub const ONE_TO_ONE_EXT: Self = Self(0x00000001);
    pub const ASPECT_RATIO_STRETCH_EXT: Self = Self(0x00000002);
    pub const STRETCH_EXT: Self = Self(0x00000004);
}

#[repr(C)]
pub struct PresentGravityFlagsEXT(u32);

impl PresentGravityFlagsEXT {
    pub const MIN_EXT: Self = Self(0x00000001);
    pub const MAX_EXT: Self = Self(0x00000002);
    pub const CENTERED_EXT: Self = Self(0x00000004);
}

#[repr(C)]
pub struct DebugUtilsMessengerCreateFlagsExt(u32);

#[repr(C)]
pub struct DebugUtilsMessageSeverityFlagsExt(u32);

impl DebugUtilsMessageSeverityFlagsExt {
    pub const VERBOSE: Self = Self(0x00000001);
    pub const INFO: Self = Self(0x00000010);
    pub const WARNING: Self = Self(0x00000100);
    pub const ERROR: Self = Self(0x00001000);
}

#[repr(C)]
pub struct DebugUtilsMessageTypeFlagsExt(u32);

impl DebugUtilsMessageTypeFlagsExt {
    pub const GENERAL: Self = Self(0x00000001);
    pub const VALIDATION: Self = Self(0x00000002);
    pub const PERFORMANCE: Self = Self(0x00000004);
    pub const DEVICE_ADDRESS_BINDING: Self = Self(0x00000008);
}

#[repr(C)]
pub struct DebugUtilsMessengerCallbackDataFlagsExt(u32);

macro_rules! impl_flags_u32 {
    ($($t:ty),+) => {
        $(
            impl $t {
                #[inline]
                pub fn from_raw(value: u32) -> Self {
                    Self(value)
                }

                #[inline]
                pub fn as_raw(self) -> u32 {
                    self.0
                }

                #[inline]
                pub fn intersects(self, rhs: Self) -> bool {
                    self.0 & rhs.0 != 0
                }

                #[inline]
                pub fn contains(self, rhs: Self) -> bool {
                    self.0 & rhs.0 == rhs.0
                }

                #[inline]
                pub fn cardinality(self) -> u32 {
                    self.0.count_ones()
                }
            }

            impl Clone for $t {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl Copy for $t {}

            impl Default for $t {
                fn default() -> Self {
                    Self(0)
                }
            }

            impl PartialEq for $t {
                fn eq(&self, rhs: &Self) -> bool {
                    self.0 == rhs.0
                }
            }

            impl Eq for $t {}

            impl std::ops::BitOr for $t {
                type Output = Self;
                fn bitor(self, rhs: Self) -> Self::Output {
                    Self(self.0 | rhs.0)
                }
            }

            impl std::ops::BitOrAssign for $t {
                fn bitor_assign(&mut self, rhs: Self) {
                    self.0 |= rhs.0
                }
            }

            impl std::ops::BitAnd for $t {
                type Output = Self;
                fn bitand(self, rhs: Self) -> Self::Output {
                    Self(self.0 & rhs.0)
                }
            }

            impl std::ops::BitAndAssign for $t {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0
            }
            }

            impl std::ops::BitXor for $t {
                type Output = Self;
                fn bitxor(self, rhs: Self) -> Self::Output {
                    Self(self.0 ^ rhs.0)
                }
            }

            impl std::ops::BitXorAssign for $t {
                fn bitxor_assign(&mut self, rhs: Self) {
                    self.0 ^= rhs.0
                }
            }
        )*
    }
}

macro_rules! impl_flags_u64 {
    ($($t:ty),+) => {
        $(
            impl $t {
                #[inline]
                pub fn from_raw(value: u64) -> Self {
                    Self(value)
                }

                #[inline]
                pub fn as_raw(self) -> u64 {
                    self.0
                }

                #[inline]
                pub fn intersects(self, rhs: Self) -> bool {
                    self.0 & rhs.0 != 0
                }

                #[inline]
                pub fn contains(self, rhs: Self) -> bool {
                    self.0 & rhs.0 == rhs.0
                }

                #[inline]
                pub fn cardinality(self) -> u32 {
                    self.0.count_ones()
                }
            }

            impl Clone for $t {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl Copy for $t {}

            impl Default for $t {
                fn default() -> Self {
                    Self(0)
                }
            }

            impl PartialEq for $t {
                fn eq(&self, rhs: &Self) -> bool {
                    self.0 == rhs.0
                }
            }

            impl Eq for $t {}

            impl std::ops::BitOr for $t {
                type Output = Self;
                fn bitor(self, rhs: Self) -> Self::Output {
                    Self(self.0 | rhs.0)
                }
            }

            impl std::ops::BitOrAssign for $t {
                fn bitor_assign(&mut self, rhs: Self) {
                    self.0 |= rhs.0
                }
            }

            impl std::ops::BitAnd for $t {
                type Output = Self;
                fn bitand(self, rhs: Self) -> Self::Output {
                    Self(self.0 & rhs.0)
                }
            }

            impl std::ops::BitAndAssign for $t {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0
            }
            }

            impl std::ops::BitXor for $t {
                type Output = Self;
                fn bitxor(self, rhs: Self) -> Self::Output {
                    Self(self.0 ^ rhs.0)
                }
            }

            impl std::ops::BitXorAssign for $t {
                fn bitxor_assign(&mut self, rhs: Self) {
                    self.0 ^= rhs.0
                }
            }
        )*
    }
}

impl_flags_u32!(
    InstanceCreateFlags,
    XcbSurfaceCreateFlagsKHR,
    XlibSurfaceCreateFlagsKHR,
    WaylandSurfaceCreateFlagsKHR,
    SampleCountFlags,
    MemoryPropertyFlags,
    MemoryHeapFlags,
    MemoryMapFlags,
    MemoryAllocateFlags,
    ImageAspectFlags,
    SparseMemoryBindFlags,
    SparseImageFormatFlags,
    QueueFlags,
    ImageUsageFlags,
    ImageCreateFlags,
    FormatFeatureFlags,
    PipelineStageFlags,
    SurfaceTransformFlagsKHR,
    SwapchainCreateFlagsKHR,
    CompositeAlphaFlagsKHR,
    ImageViewCreateFlags,
    CommandPoolCreateFlags,
    CommandPoolResetFlags,
    CommandBufferResetFlags,
    CommandBufferUsageFlags,
    QueryControlFlags,
    QueryResultFlags,
    QueryPipelineStatisticFlags,
    AttachmentDescriptionFlags,
    AccessFlags,
    DependencyFlags,
    SubpassDescriptionFlags,
    RenderPassCreateFlags,
    FramebufferCreateFlags,
    FenceCreateFlags,
    SemaphoreCreateFlags,
    ShaderModuleCreateFlags,
    ShaderStageFlags,
    DescriptorSetLayoutCreateFlags,
    StencilFaceFlags,
    CullModeFlags,
    DescriptorPoolCreateFlags,
    DescriptorPoolResetFlags,
    SamplerCreateFlags,
    PipelineLayoutCreateFlags,
    PipelineCacheCreateFlags,
    PipelineDepthStencilStateCreateFlags,
    PipelineDynamicStateCreateFlags,
    PipelineColorBlendStateCreateFlags,
    PipelineMultisampleStateCreateFlags,
    PipelineRasterizationStateCreateFlags,
    PipelineViewportStateCreateFlags,
    PipelineTessellationStateCreateFlags,
    PipelineInputAssemblyStateCreateFlags,
    PipelineVertexInputStateCreateFlags,
    PipelineShaderStageCreateFlags,
    PipelineCreateFlags,
    ColorComponentFlags,
    BufferCreateFlags,
    BufferUsageFlags,
    BufferViewCreateFlags,
    SemaphoreWaitFlags,
    ResolveModeFlags,
    RenderingFlags,
    SubgroupFeatureFlags,
    SubmitFlags,
    PresentScalingFlagsEXT,
    PresentGravityFlagsEXT
);

impl_flags_u64!(AccessFlags2, PipelineStageFlags2);
