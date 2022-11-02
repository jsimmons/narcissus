#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Instance(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct PhysicalDevice(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Device(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Queue(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct CommandBuffer(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct DeviceMemory(pub u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct CommandPool(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Buffer(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct BufferView(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Image(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct ImageView(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct ShaderModule(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Pipeline(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct PipelineLayout(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Sampler(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct DescriptorSet(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct DescriptorSetLayout(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct DescriptorPool(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Fence(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Semaphore(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Event(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct QueryPool(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Framebuffer(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct RenderPass(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct PipelineCache(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct DescriptorUpdateTemplate(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct DisplayKHR(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct DisplayModeKHR(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct SurfaceKHR(u64);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct SwapchainKHR(u64);

// impl Handle {
//     #[inline]
//     pub const fn null() -> Self {
//         Self(0)
//     }

//     #[inline]
//     pub fn as_raw(self) -> u64 {
//         self.0
//     }

//     #[inline]
//     pub fn from_raw(value: u64) -> Self {
//         Self(value)
//     }
// }

// Instance
// PhysicalDevice
// Device
// Queue
// CommandBuffer
// DeviceMemory
// CommandPool
// Buffer
// BufferView
// Image
// ImageView
// ShaderModule
// Pipeline
// PipelineLayout
// Sampler
// DescriptorSet
// DescriptorSetLayout
// DescriptorPool
// Fence
// Semaphore
// Event
// QueryPool
// Framebuffer
// RenderPass
// PipelineCache
// DescriptorUpdateTemplate
// DisplayKHR
// DisplayModeKHR
// SurfaceKHR
// SwapchainKHR

impl Instance {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl PhysicalDevice {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Device {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Queue {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl CommandBuffer {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl DeviceMemory {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl CommandPool {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Buffer {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl BufferView {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Image {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl ImageView {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl ShaderModule {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Pipeline {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl PipelineLayout {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Sampler {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl DescriptorSet {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl DescriptorSetLayout {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl DescriptorPool {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Fence {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Semaphore {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Event {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl QueryPool {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl Framebuffer {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl RenderPass {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl PipelineCache {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl DescriptorUpdateTemplate {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl DisplayKHR {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl DisplayModeKHR {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl SurfaceKHR {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
impl SwapchainKHR {
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}
