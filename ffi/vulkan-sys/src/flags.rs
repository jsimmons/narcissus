#[repr(C)]
pub struct InstanceCreateFlags(u32);

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

// Impls

// InstanceCreateFlags
// SampleCountFlags
// MemoryPropertyFlags
// MemoryHeapFlags
// MemoryMapFlags
// ImageAspectFlags
// SparseMemoryBindFlags
// SparseImageFormatFlags
// QueueFlags
// ImageUsageFlags
// ImageCreateFlags
// FormatFeatureFlags
// PipelineStageFlags
// SurfaceTransformFlagsKHR
// SwapchainCreateFlagsKHR
// CompositeAlphaFlagsKHR
// ImageViewCreateFlags
// CommandPoolCreateFlags
// CommandPoolResetFlags
// CommandBufferResetFlags
// CommandBufferUsageFlags
// QueryControlFlags
// QueryResultFlags
// QueryPipelineStatisticFlags
// AttachmentDescriptionFlags
// AccessFlags
// DependencyFlags
// SubpassDescriptionFlags
// RenderPassCreateFlags
// FramebufferCreateFlags
// FenceCreateFlags
// SemaphoreCreateFlags
// ShaderModuleCreateFlags
// ShaderStageFlags
// DescriptorSetLayoutCreateFlags
// StencilFaceFlags
// CullModeFlags
// DescriptorPoolCreateFlags
// DescriptorPoolResetFlags
// SamplerCreateFlags
// PipelineLayoutCreateFlags
// PipelineCacheCreateFlags
// PipelineDepthStencilStateCreateFlags
// PipelineDynamicStateCreateFlags
// PipelineColorBlendStateCreateFlags
// PipelineMultisampleStateCreateFlags
// PipelineRasterizationStateCreateFlags
// PipelineViewportStateCreateFlags
// PipelineTessellationStateCreateFlags
// PipelineInputAssemblyStateCreateFlags
// PipelineVertexInputStateCreateFlags
// PipelineShaderStageCreateFlags
// PipelineCreateFlags
// ColorComponentFlags
// BufferCreateFlags
// BufferUsageFlags
// BufferViewCreateFlags
// SemaphoreWaitFlags
// ResolveModeFlags
// RenderingFlags
// SubgroupFeatureFlags
// SubmitFlags
// AccessFlags2
// PipelineStageFlags2

// Reference Implementation For Flags.

// impl Flags {
//     #[inline]
//     pub fn from_raw(value: u32) -> Self {
//         Self(value)
//     }

//     #[inline]
//     pub fn as_raw(self) -> u32 {
//         self.0
//     }

//     #[inline]
//     pub fn intersects(self, rhs: Self) -> bool {
//         self.0 & rhs.0 != 0
//     }

//     #[inline]
//     pub fn contains(self, rhs: Self) -> bool {
//         self.0 & rhs.0 == rhs.0
//     }

//     #[inline]
//     pub fn cardinality(self) -> u32 {
//         self.0.count_ones()
//     }
// }

// impl Clone for Flags {
//     fn clone(&self) -> Self {
//         Self(self.0)
//     }
// }

// impl Copy for Flags {}

// impl Default for Flags {
//     fn default() -> Self {
//         Self(0)
//     }
// }

// impl PartialEq for Flags {
//     fn eq(&self, rhs: &Self) -> bool {
//         self.0 == rhs.0
//     }
// }

// impl Eq for Flags {}

// impl std::ops::BitOr for Flags {
//     type Output = Self;
//     fn bitor(self, rhs: Self) -> Self::Output {
//         Self(self.0 | rhs.0)
//     }
// }

// impl std::ops::BitOrAssign for Flags {
//     fn bitor_assign(&mut self, rhs: Self) {
//         self.0 |= rhs.0
//     }
// }

// impl std::ops::BitAnd for Flags {
//     type Output = Self;
//     fn bitand(self, rhs: Self) -> Self::Output {
//         Self(self.0 & rhs.0)
//     }
// }

// impl std::ops::BitAndAssign for Flags {
// fn bitand_assign(&mut self, rhs: Self) {
//     self.0 &= rhs.0
// }
// }

// impl std::ops::BitXor for Flags {
//     type Output = Self;
//     fn bitxor(self, rhs: Self) -> Self::Output {
//         Self(self.0 ^ rhs.0)
//     }
// }

// impl std::ops::BitXorAssign for Flags {
//     fn bitxor_assign(&mut self, rhs: Self) {
//         self.0 ^= rhs.0
//     }
// }

impl PipelineStageFlags {
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

impl Clone for PipelineStageFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineStageFlags {}

impl Default for PipelineStageFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineStageFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineStageFlags {}

impl std::ops::BitOr for PipelineStageFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineStageFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineStageFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineStageFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineStageFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineStageFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl InstanceCreateFlags {
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

impl Clone for InstanceCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for InstanceCreateFlags {}

impl Default for InstanceCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for InstanceCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for InstanceCreateFlags {}

impl std::ops::BitOr for InstanceCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for InstanceCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for InstanceCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for InstanceCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for InstanceCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for InstanceCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl DeviceCreateFlags {
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

impl Clone for DeviceCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for DeviceCreateFlags {}

impl Default for DeviceCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for DeviceCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for DeviceCreateFlags {}

impl std::ops::BitOr for DeviceCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DeviceCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for DeviceCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for DeviceCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for DeviceCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for DeviceCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl DeviceQueueCreateFlags {
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

impl Clone for DeviceQueueCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for DeviceQueueCreateFlags {}

impl Default for DeviceQueueCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for DeviceQueueCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for DeviceQueueCreateFlags {}

impl std::ops::BitOr for DeviceQueueCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DeviceQueueCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for DeviceQueueCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for DeviceQueueCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for DeviceQueueCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for DeviceQueueCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl SampleCountFlags {
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

impl Clone for SampleCountFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SampleCountFlags {}

impl Default for SampleCountFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SampleCountFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SampleCountFlags {}

impl std::ops::BitOr for SampleCountFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SampleCountFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SampleCountFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SampleCountFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SampleCountFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SampleCountFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl MemoryPropertyFlags {
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

impl Clone for MemoryPropertyFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for MemoryPropertyFlags {}

impl Default for MemoryPropertyFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for MemoryPropertyFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for MemoryPropertyFlags {}

impl std::ops::BitOr for MemoryPropertyFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for MemoryPropertyFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for MemoryPropertyFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for MemoryPropertyFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for MemoryPropertyFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for MemoryPropertyFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl MemoryHeapFlags {
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

impl Clone for MemoryHeapFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for MemoryHeapFlags {}

impl Default for MemoryHeapFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for MemoryHeapFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for MemoryHeapFlags {}

impl std::ops::BitOr for MemoryHeapFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for MemoryHeapFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for MemoryHeapFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for MemoryHeapFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for MemoryHeapFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for MemoryHeapFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl MemoryMapFlags {
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

impl Clone for MemoryMapFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for MemoryMapFlags {}

impl Default for MemoryMapFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for MemoryMapFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for MemoryMapFlags {}

impl std::ops::BitOr for MemoryMapFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for MemoryMapFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for MemoryMapFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for MemoryMapFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for MemoryMapFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for MemoryMapFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl ImageAspectFlags {
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

impl Clone for ImageAspectFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for ImageAspectFlags {}

impl Default for ImageAspectFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for ImageAspectFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for ImageAspectFlags {}

impl std::ops::BitOr for ImageAspectFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ImageAspectFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for ImageAspectFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ImageAspectFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for ImageAspectFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for ImageAspectFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl SparseMemoryBindFlags {
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

impl Clone for SparseMemoryBindFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SparseMemoryBindFlags {}

impl Default for SparseMemoryBindFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SparseMemoryBindFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SparseMemoryBindFlags {}

impl std::ops::BitOr for SparseMemoryBindFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SparseMemoryBindFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SparseMemoryBindFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SparseMemoryBindFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SparseMemoryBindFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SparseMemoryBindFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl SparseImageFormatFlags {
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

impl Clone for SparseImageFormatFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SparseImageFormatFlags {}

impl Default for SparseImageFormatFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SparseImageFormatFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SparseImageFormatFlags {}

impl std::ops::BitOr for SparseImageFormatFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SparseImageFormatFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SparseImageFormatFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SparseImageFormatFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SparseImageFormatFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SparseImageFormatFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl QueueFlags {
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

impl Clone for QueueFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for QueueFlags {}

impl Default for QueueFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for QueueFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for QueueFlags {}

impl std::ops::BitOr for QueueFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for QueueFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for QueueFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for QueueFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for QueueFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for QueueFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl ImageUsageFlags {
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

impl Clone for ImageUsageFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for ImageUsageFlags {}

impl Default for ImageUsageFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for ImageUsageFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for ImageUsageFlags {}

impl std::ops::BitOr for ImageUsageFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ImageUsageFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for ImageUsageFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ImageUsageFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for ImageUsageFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for ImageUsageFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl ImageCreateFlags {
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

impl Clone for ImageCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for ImageCreateFlags {}

impl Default for ImageCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for ImageCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for ImageCreateFlags {}

impl std::ops::BitOr for ImageCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ImageCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for ImageCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ImageCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for ImageCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for ImageCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl FormatFeatureFlags {
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

impl Clone for FormatFeatureFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for FormatFeatureFlags {}

impl Default for FormatFeatureFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for FormatFeatureFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for FormatFeatureFlags {}

impl std::ops::BitOr for FormatFeatureFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for FormatFeatureFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for FormatFeatureFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for FormatFeatureFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for FormatFeatureFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for FormatFeatureFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl SurfaceTransformFlagsKHR {
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

impl Clone for SurfaceTransformFlagsKHR {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SurfaceTransformFlagsKHR {}

impl Default for SurfaceTransformFlagsKHR {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SurfaceTransformFlagsKHR {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SurfaceTransformFlagsKHR {}

impl std::ops::BitOr for SurfaceTransformFlagsKHR {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SurfaceTransformFlagsKHR {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SurfaceTransformFlagsKHR {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SurfaceTransformFlagsKHR {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SurfaceTransformFlagsKHR {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SurfaceTransformFlagsKHR {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl SwapchainCreateFlagsKHR {
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

impl Clone for SwapchainCreateFlagsKHR {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SwapchainCreateFlagsKHR {}

impl Default for SwapchainCreateFlagsKHR {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SwapchainCreateFlagsKHR {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SwapchainCreateFlagsKHR {}

impl std::ops::BitOr for SwapchainCreateFlagsKHR {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SwapchainCreateFlagsKHR {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SwapchainCreateFlagsKHR {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SwapchainCreateFlagsKHR {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SwapchainCreateFlagsKHR {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SwapchainCreateFlagsKHR {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl CompositeAlphaFlagsKHR {
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

impl Clone for CompositeAlphaFlagsKHR {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for CompositeAlphaFlagsKHR {}

impl Default for CompositeAlphaFlagsKHR {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for CompositeAlphaFlagsKHR {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for CompositeAlphaFlagsKHR {}

impl std::ops::BitOr for CompositeAlphaFlagsKHR {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CompositeAlphaFlagsKHR {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for CompositeAlphaFlagsKHR {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for CompositeAlphaFlagsKHR {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for CompositeAlphaFlagsKHR {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for CompositeAlphaFlagsKHR {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl ImageViewCreateFlags {
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

impl Clone for ImageViewCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for ImageViewCreateFlags {}

impl Default for ImageViewCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for ImageViewCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for ImageViewCreateFlags {}

impl std::ops::BitOr for ImageViewCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ImageViewCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for ImageViewCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ImageViewCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for ImageViewCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for ImageViewCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl CommandPoolCreateFlags {
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

impl Clone for CommandPoolCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for CommandPoolCreateFlags {}

impl Default for CommandPoolCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for CommandPoolCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for CommandPoolCreateFlags {}

impl std::ops::BitOr for CommandPoolCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CommandPoolCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for CommandPoolCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for CommandPoolCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for CommandPoolCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for CommandPoolCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl CommandPoolResetFlags {
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

impl Clone for CommandPoolResetFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for CommandPoolResetFlags {}

impl Default for CommandPoolResetFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for CommandPoolResetFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for CommandPoolResetFlags {}

impl std::ops::BitOr for CommandPoolResetFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CommandPoolResetFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for CommandPoolResetFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for CommandPoolResetFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for CommandPoolResetFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for CommandPoolResetFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl CommandBufferResetFlags {
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

impl Clone for CommandBufferResetFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for CommandBufferResetFlags {}

impl Default for CommandBufferResetFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for CommandBufferResetFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for CommandBufferResetFlags {}

impl std::ops::BitOr for CommandBufferResetFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CommandBufferResetFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for CommandBufferResetFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for CommandBufferResetFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for CommandBufferResetFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for CommandBufferResetFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl CommandBufferUsageFlags {
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

impl Clone for CommandBufferUsageFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for CommandBufferUsageFlags {}

impl Default for CommandBufferUsageFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for CommandBufferUsageFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for CommandBufferUsageFlags {}

impl std::ops::BitOr for CommandBufferUsageFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CommandBufferUsageFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for CommandBufferUsageFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for CommandBufferUsageFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for CommandBufferUsageFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for CommandBufferUsageFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl QueryControlFlags {
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

impl Clone for QueryControlFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for QueryControlFlags {}

impl Default for QueryControlFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for QueryControlFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for QueryControlFlags {}

impl std::ops::BitOr for QueryControlFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for QueryControlFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for QueryControlFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for QueryControlFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for QueryControlFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for QueryControlFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl QueryResultFlags {
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

impl Clone for QueryResultFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for QueryResultFlags {}

impl Default for QueryResultFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for QueryResultFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for QueryResultFlags {}

impl std::ops::BitOr for QueryResultFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for QueryResultFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for QueryResultFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for QueryResultFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for QueryResultFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for QueryResultFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl QueryPipelineStatisticFlags {
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

impl Clone for QueryPipelineStatisticFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for QueryPipelineStatisticFlags {}

impl Default for QueryPipelineStatisticFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for QueryPipelineStatisticFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for QueryPipelineStatisticFlags {}

impl std::ops::BitOr for QueryPipelineStatisticFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for QueryPipelineStatisticFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for QueryPipelineStatisticFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for QueryPipelineStatisticFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for QueryPipelineStatisticFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for QueryPipelineStatisticFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl AttachmentDescriptionFlags {
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

impl Clone for AttachmentDescriptionFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for AttachmentDescriptionFlags {}

impl Default for AttachmentDescriptionFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for AttachmentDescriptionFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for AttachmentDescriptionFlags {}

impl std::ops::BitOr for AttachmentDescriptionFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for AttachmentDescriptionFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for AttachmentDescriptionFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for AttachmentDescriptionFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for AttachmentDescriptionFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for AttachmentDescriptionFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl AccessFlags {
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

impl Clone for AccessFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for AccessFlags {}

impl Default for AccessFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for AccessFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for AccessFlags {}

impl std::ops::BitOr for AccessFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for AccessFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for AccessFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for AccessFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for AccessFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for AccessFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl DependencyFlags {
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

impl Clone for DependencyFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for DependencyFlags {}

impl Default for DependencyFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for DependencyFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for DependencyFlags {}

impl std::ops::BitOr for DependencyFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DependencyFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for DependencyFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for DependencyFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for DependencyFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for DependencyFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl SubpassDescriptionFlags {
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

impl Clone for SubpassDescriptionFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SubpassDescriptionFlags {}

impl Default for SubpassDescriptionFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SubpassDescriptionFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SubpassDescriptionFlags {}

impl std::ops::BitOr for SubpassDescriptionFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SubpassDescriptionFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SubpassDescriptionFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SubpassDescriptionFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SubpassDescriptionFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SubpassDescriptionFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl RenderPassCreateFlags {
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

impl Clone for RenderPassCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for RenderPassCreateFlags {}

impl Default for RenderPassCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for RenderPassCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for RenderPassCreateFlags {}

impl std::ops::BitOr for RenderPassCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for RenderPassCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for RenderPassCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for RenderPassCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for RenderPassCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for RenderPassCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl FramebufferCreateFlags {
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

impl Clone for FramebufferCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for FramebufferCreateFlags {}

impl Default for FramebufferCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for FramebufferCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for FramebufferCreateFlags {}

impl std::ops::BitOr for FramebufferCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for FramebufferCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for FramebufferCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for FramebufferCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for FramebufferCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for FramebufferCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl FenceCreateFlags {
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

impl Clone for FenceCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for FenceCreateFlags {}

impl Default for FenceCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for FenceCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for FenceCreateFlags {}

impl std::ops::BitOr for FenceCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for FenceCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for FenceCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for FenceCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for FenceCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for FenceCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl SemaphoreCreateFlags {
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

impl Clone for SemaphoreCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SemaphoreCreateFlags {}

impl Default for SemaphoreCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SemaphoreCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SemaphoreCreateFlags {}

impl std::ops::BitOr for SemaphoreCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SemaphoreCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SemaphoreCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SemaphoreCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SemaphoreCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SemaphoreCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl ShaderModuleCreateFlags {
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

impl Clone for ShaderModuleCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for ShaderModuleCreateFlags {}

impl Default for ShaderModuleCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for ShaderModuleCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for ShaderModuleCreateFlags {}

impl std::ops::BitOr for ShaderModuleCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ShaderModuleCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for ShaderModuleCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ShaderModuleCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for ShaderModuleCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for ShaderModuleCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl ShaderStageFlags {
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

impl Clone for ShaderStageFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for ShaderStageFlags {}

impl Default for ShaderStageFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for ShaderStageFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for ShaderStageFlags {}

impl std::ops::BitOr for ShaderStageFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ShaderStageFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for ShaderStageFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ShaderStageFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for ShaderStageFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for ShaderStageFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl DescriptorSetLayoutCreateFlags {
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

impl Clone for DescriptorSetLayoutCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for DescriptorSetLayoutCreateFlags {}

impl Default for DescriptorSetLayoutCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for DescriptorSetLayoutCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for DescriptorSetLayoutCreateFlags {}

impl std::ops::BitOr for DescriptorSetLayoutCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DescriptorSetLayoutCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for DescriptorSetLayoutCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for DescriptorSetLayoutCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for DescriptorSetLayoutCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for DescriptorSetLayoutCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl StencilFaceFlags {
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

impl Clone for StencilFaceFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for StencilFaceFlags {}

impl Default for StencilFaceFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for StencilFaceFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for StencilFaceFlags {}

impl std::ops::BitOr for StencilFaceFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for StencilFaceFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for StencilFaceFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for StencilFaceFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for StencilFaceFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for StencilFaceFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl CullModeFlags {
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

impl Clone for CullModeFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for CullModeFlags {}

impl Default for CullModeFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for CullModeFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for CullModeFlags {}

impl std::ops::BitOr for CullModeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CullModeFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for CullModeFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for CullModeFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for CullModeFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for CullModeFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl DescriptorPoolCreateFlags {
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

impl Clone for DescriptorPoolCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for DescriptorPoolCreateFlags {}

impl Default for DescriptorPoolCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for DescriptorPoolCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for DescriptorPoolCreateFlags {}

impl std::ops::BitOr for DescriptorPoolCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DescriptorPoolCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for DescriptorPoolCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for DescriptorPoolCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for DescriptorPoolCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for DescriptorPoolCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl DescriptorPoolResetFlags {
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

impl Clone for DescriptorPoolResetFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for DescriptorPoolResetFlags {}

impl Default for DescriptorPoolResetFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for DescriptorPoolResetFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for DescriptorPoolResetFlags {}

impl std::ops::BitOr for DescriptorPoolResetFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DescriptorPoolResetFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for DescriptorPoolResetFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for DescriptorPoolResetFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for DescriptorPoolResetFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for DescriptorPoolResetFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl SamplerCreateFlags {
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

impl Clone for SamplerCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SamplerCreateFlags {}

impl Default for SamplerCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SamplerCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SamplerCreateFlags {}

impl std::ops::BitOr for SamplerCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SamplerCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SamplerCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SamplerCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SamplerCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SamplerCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineLayoutCreateFlags {
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

impl Clone for PipelineLayoutCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineLayoutCreateFlags {}

impl Default for PipelineLayoutCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineLayoutCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineLayoutCreateFlags {}

impl std::ops::BitOr for PipelineLayoutCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineLayoutCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineLayoutCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineLayoutCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineLayoutCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineLayoutCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineCacheCreateFlags {
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

impl Clone for PipelineCacheCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineCacheCreateFlags {}

impl Default for PipelineCacheCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineCacheCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineCacheCreateFlags {}

impl std::ops::BitOr for PipelineCacheCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineCacheCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineCacheCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineCacheCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineCacheCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineCacheCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineDepthStencilStateCreateFlags {
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

impl Clone for PipelineDepthStencilStateCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineDepthStencilStateCreateFlags {}

impl Default for PipelineDepthStencilStateCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineDepthStencilStateCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineDepthStencilStateCreateFlags {}

impl std::ops::BitOr for PipelineDepthStencilStateCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineDepthStencilStateCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineDepthStencilStateCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineDepthStencilStateCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineDepthStencilStateCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineDepthStencilStateCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineDynamicStateCreateFlags {
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

impl Clone for PipelineDynamicStateCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineDynamicStateCreateFlags {}

impl Default for PipelineDynamicStateCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineDynamicStateCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineDynamicStateCreateFlags {}

impl std::ops::BitOr for PipelineDynamicStateCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineDynamicStateCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineDynamicStateCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineDynamicStateCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineDynamicStateCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineDynamicStateCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineColorBlendStateCreateFlags {
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

impl Clone for PipelineColorBlendStateCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineColorBlendStateCreateFlags {}

impl Default for PipelineColorBlendStateCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineColorBlendStateCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineColorBlendStateCreateFlags {}

impl std::ops::BitOr for PipelineColorBlendStateCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineColorBlendStateCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineColorBlendStateCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineColorBlendStateCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineColorBlendStateCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineColorBlendStateCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineMultisampleStateCreateFlags {
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

impl Clone for PipelineMultisampleStateCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineMultisampleStateCreateFlags {}

impl Default for PipelineMultisampleStateCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineMultisampleStateCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineMultisampleStateCreateFlags {}

impl std::ops::BitOr for PipelineMultisampleStateCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineMultisampleStateCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineMultisampleStateCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineMultisampleStateCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineMultisampleStateCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineMultisampleStateCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineRasterizationStateCreateFlags {
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

impl Clone for PipelineRasterizationStateCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineRasterizationStateCreateFlags {}

impl Default for PipelineRasterizationStateCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineRasterizationStateCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineRasterizationStateCreateFlags {}

impl std::ops::BitOr for PipelineRasterizationStateCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineRasterizationStateCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineRasterizationStateCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineRasterizationStateCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineRasterizationStateCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineRasterizationStateCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineViewportStateCreateFlags {
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

impl Clone for PipelineViewportStateCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineViewportStateCreateFlags {}

impl Default for PipelineViewportStateCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineViewportStateCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineViewportStateCreateFlags {}

impl std::ops::BitOr for PipelineViewportStateCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineViewportStateCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineViewportStateCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineViewportStateCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineViewportStateCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineViewportStateCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineTessellationStateCreateFlags {
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

impl Clone for PipelineTessellationStateCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineTessellationStateCreateFlags {}

impl Default for PipelineTessellationStateCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineTessellationStateCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineTessellationStateCreateFlags {}

impl std::ops::BitOr for PipelineTessellationStateCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineTessellationStateCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineTessellationStateCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineTessellationStateCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineTessellationStateCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineTessellationStateCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineInputAssemblyStateCreateFlags {
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

impl Clone for PipelineInputAssemblyStateCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineInputAssemblyStateCreateFlags {}

impl Default for PipelineInputAssemblyStateCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineInputAssemblyStateCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineInputAssemblyStateCreateFlags {}

impl std::ops::BitOr for PipelineInputAssemblyStateCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineInputAssemblyStateCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineInputAssemblyStateCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineInputAssemblyStateCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineInputAssemblyStateCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineInputAssemblyStateCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineVertexInputStateCreateFlags {
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

impl Clone for PipelineVertexInputStateCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineVertexInputStateCreateFlags {}

impl Default for PipelineVertexInputStateCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineVertexInputStateCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineVertexInputStateCreateFlags {}

impl std::ops::BitOr for PipelineVertexInputStateCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineVertexInputStateCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineVertexInputStateCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineVertexInputStateCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineVertexInputStateCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineVertexInputStateCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineShaderStageCreateFlags {
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

impl Clone for PipelineShaderStageCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineShaderStageCreateFlags {}

impl Default for PipelineShaderStageCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineShaderStageCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineShaderStageCreateFlags {}

impl std::ops::BitOr for PipelineShaderStageCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineShaderStageCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineShaderStageCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineShaderStageCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineShaderStageCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineShaderStageCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl PipelineCreateFlags {
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

impl Clone for PipelineCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineCreateFlags {}

impl Default for PipelineCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineCreateFlags {}

impl std::ops::BitOr for PipelineCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl ColorComponentFlags {
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

impl Clone for ColorComponentFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for ColorComponentFlags {}

impl Default for ColorComponentFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for ColorComponentFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for ColorComponentFlags {}

impl std::ops::BitOr for ColorComponentFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ColorComponentFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for ColorComponentFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ColorComponentFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for ColorComponentFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for ColorComponentFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl BufferCreateFlags {
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

impl Clone for BufferCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for BufferCreateFlags {}

impl Default for BufferCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for BufferCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for BufferCreateFlags {}

impl std::ops::BitOr for BufferCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for BufferCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for BufferCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for BufferCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for BufferCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for BufferCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl BufferUsageFlags {
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

impl Clone for BufferUsageFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for BufferUsageFlags {}

impl Default for BufferUsageFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for BufferUsageFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for BufferUsageFlags {}

impl std::ops::BitOr for BufferUsageFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for BufferUsageFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for BufferUsageFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for BufferUsageFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for BufferUsageFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for BufferUsageFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl BufferViewCreateFlags {
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

impl Clone for BufferViewCreateFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for BufferViewCreateFlags {}

impl Default for BufferViewCreateFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for BufferViewCreateFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for BufferViewCreateFlags {}

impl std::ops::BitOr for BufferViewCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for BufferViewCreateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for BufferViewCreateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for BufferViewCreateFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for BufferViewCreateFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for BufferViewCreateFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl SemaphoreWaitFlags {
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

impl Clone for SemaphoreWaitFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SemaphoreWaitFlags {}

impl Default for SemaphoreWaitFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SemaphoreWaitFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SemaphoreWaitFlags {}

impl std::ops::BitOr for SemaphoreWaitFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SemaphoreWaitFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SemaphoreWaitFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SemaphoreWaitFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SemaphoreWaitFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SemaphoreWaitFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl ResolveModeFlags {
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

impl Clone for ResolveModeFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for ResolveModeFlags {}

impl Default for ResolveModeFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for ResolveModeFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for ResolveModeFlags {}

impl std::ops::BitOr for ResolveModeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ResolveModeFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for ResolveModeFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ResolveModeFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for ResolveModeFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for ResolveModeFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl RenderingFlags {
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

impl Clone for RenderingFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for RenderingFlags {}

impl Default for RenderingFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for RenderingFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for RenderingFlags {}

impl std::ops::BitOr for RenderingFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for RenderingFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for RenderingFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for RenderingFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for RenderingFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for RenderingFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl SubgroupFeatureFlags {
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

impl Clone for SubgroupFeatureFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SubgroupFeatureFlags {}

impl Default for SubgroupFeatureFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SubgroupFeatureFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SubgroupFeatureFlags {}

impl std::ops::BitOr for SubgroupFeatureFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SubgroupFeatureFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SubgroupFeatureFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SubgroupFeatureFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SubgroupFeatureFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SubgroupFeatureFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
impl SubmitFlags {
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

impl Clone for SubmitFlags {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for SubmitFlags {}

impl Default for SubmitFlags {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for SubmitFlags {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for SubmitFlags {}

impl std::ops::BitOr for SubmitFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SubmitFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for SubmitFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for SubmitFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for SubmitFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for SubmitFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl PipelineStageFlags2 {
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

impl Clone for PipelineStageFlags2 {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for PipelineStageFlags2 {}

impl Default for PipelineStageFlags2 {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for PipelineStageFlags2 {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for PipelineStageFlags2 {}

impl std::ops::BitOr for PipelineStageFlags2 {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PipelineStageFlags2 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for PipelineStageFlags2 {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for PipelineStageFlags2 {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for PipelineStageFlags2 {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for PipelineStageFlags2 {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl AccessFlags2 {
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

impl Clone for AccessFlags2 {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Copy for AccessFlags2 {}

impl Default for AccessFlags2 {
    fn default() -> Self {
        Self(0)
    }
}

impl PartialEq for AccessFlags2 {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl Eq for AccessFlags2 {}

impl std::ops::BitOr for AccessFlags2 {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for AccessFlags2 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitAnd for AccessFlags2 {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for AccessFlags2 {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitXor for AccessFlags2 {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for AccessFlags2 {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
