#![allow(unused)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::missing_transmute_annotations)]

mod enums;
mod flags;
mod functions;
mod handles;
mod structs;

pub use enums::*;
pub use flags::*;
pub use functions::*;
pub use handles::*;
pub use structs::*;

use std::{
    convert::{TryFrom, TryInto},
    ffi::{c_void, CStr},
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
    os::raw::c_char,
};

pub const fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    (major << 22) | (minor << 12) | patch
}

pub const fn get_version(ver: u32) -> (u32, u32, u32) {
    (ver >> 22, (ver >> 12) & 0x3ff, ver & 0xfff)
}

pub const VERSION_1_0: u32 = make_version(1, 0, 0);
pub const VERSION_1_1: u32 = make_version(1, 1, 0);
pub const VERSION_1_2: u32 = make_version(1, 2, 0);
pub const VERSION_1_3: u32 = make_version(1, 3, 0);

pub const MAX_PHYSICAL_DEVICE_NAME_SIZE: u32 = 256;
pub const UUID_SIZE: u32 = 16;
pub const LUID_SIZE: u32 = 8;
pub const MAX_EXTENSION_NAME_SIZE: u32 = 256;
pub const MAX_DESCRIPTION_SIZE: u32 = 256;
pub const MAX_MEMORY_TYPES: u32 = 32;
pub const MAX_MEMORY_HEAPS: u32 = 16;
pub const LOD_CLAMP_NONE: f32 = 1000.0;
pub const REMAINING_MIP_LEVELS: u32 = !0u32;
pub const REMAINING_ARRAY_LAYERS: u32 = !0u32;
pub const WHOLE_SIZE: u64 = !0u64;
pub const ATTACHMENT_UNUSED: u32 = !0u32;
pub const TRUE: u32 = 1;
pub const FALSE: u32 = 0;
pub const QUEUE_FAMILY_IGNORED: u32 = !0u32;
pub const QUEUE_FAMILY_EXTERNAL: u32 = !1u32;
pub const QUEUE_FAMILY_EXTERNAL_KHR: u32 = QUEUE_FAMILY_EXTERNAL;
pub const QUEUE_FAMILY_FOREIGN_EXT: u32 = !2u32;
pub const SUBPASS_EXTERNAL: u32 = !0u32;
pub const MAX_DEVICE_GROUP_SIZE: u32 = 32;
pub const MAX_DEVICE_GROUP_SIZE_KHR: u32 = MAX_DEVICE_GROUP_SIZE;
pub const MAX_DRIVER_NAME_SIZE: u32 = 256;
pub const MAX_DRIVER_NAME_SIZE_KHR: u32 = MAX_DRIVER_NAME_SIZE;
pub const MAX_DRIVER_INFO_SIZE: u32 = 256;
pub const MAX_DRIVER_INFO_SIZE_KHR: u32 = MAX_DRIVER_INFO_SIZE;
pub const SHADER_UNUSED_KHR: u32 = !0u32;
pub const SHADER_UNUSED_NV: u32 = SHADER_UNUSED_KHR;
pub const MAX_GLOBAL_PRIORITY_SIZE_EXT: u32 = 16;

pub type SampleMask = u32;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bool32 {
    False = 0,
    True = 1,
}

impl From<bool> for Bool32 {
    fn from(value: bool) -> Self {
        match value {
            true => Bool32::True,
            false => Bool32::False,
        }
    }
}

impl Default for Bool32 {
    fn default() -> Self {
        Bool32::False
    }
}

pub type DeviceSize = u64;
pub type DeviceAddress = u64;

#[repr(C)]
#[repr(packed(4))]
pub struct VulkanSlice1<'a, I, T, const PAD: usize> {
    len: I,
    #[doc(hidden)]
    _pad: MaybeUninit<[u8; PAD]>,
    ptr: *const T,
    phantom: PhantomData<&'a T>,
}

impl<I, T, const PAD: usize> std::fmt::Debug for VulkanSlice1<'_, I, T, PAD>
where
    I: TryInto<usize> + Copy,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.len.try_into().unwrap_or(0);
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, len) };
        f.debug_list().entries(slice).finish()
    }
}

impl<I: Default, T, const PAD: usize> Default for VulkanSlice1<'_, I, T, PAD> {
    fn default() -> Self {
        Self {
            len: Default::default(),
            _pad: MaybeUninit::uninit(),
            ptr: std::ptr::null(),
            phantom: PhantomData,
        }
    }
}

impl<I, T, const PAD: usize> VulkanSlice1<'_, I, T, PAD> {
    pub const fn dangling(len: I) -> Self {
        Self {
            len,
            _pad: MaybeUninit::uninit(),
            ptr: std::ptr::null(),
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T, const PAD: usize> From<&'a [T]> for VulkanSlice1<'a, I, T, PAD>
where
    I: TryFrom<usize>,
{
    fn from(x: &'a [T]) -> Self {
        let Ok(len) = I::try_from(x.len()) else {
            panic!("invalid slice length")
        };
        let ptr = x.as_ptr();
        Self {
            len,
            _pad: MaybeUninit::uninit(),
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T, const N: usize, const PAD: usize> From<&'a [T; N]> for VulkanSlice1<'a, I, T, PAD>
where
    I: TryFrom<usize>,
{
    fn from(x: &'a [T; N]) -> Self {
        let Ok(len) = I::try_from(N) else {
            panic!("invalid slice length")
        };
        let ptr = x.as_ptr();
        Self {
            len,
            _pad: MaybeUninit::uninit(),
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T, const PAD: usize> From<&'a mut [T]> for VulkanSlice1<'a, I, T, PAD>
where
    I: TryFrom<usize>,
{
    #[inline(always)]
    fn from(x: &'a mut [T]) -> Self {
        (x as &[_]).into()
    }
}

impl<'a, I, T, const N: usize, const PAD: usize> From<&'a mut [T; N]>
    for VulkanSlice1<'a, I, T, PAD>
where
    I: TryFrom<usize>,
{
    #[inline(always)]
    fn from(x: &'a mut [T; N]) -> Self {
        (x as &[T; N]).into()
    }
}

#[repr(C)]
#[repr(packed(4))]
pub struct VulkanSlice2<'a, I, T0, T1, const PAD: usize> {
    len: I,
    #[doc(hidden)]
    _pad: MaybeUninit<[u8; PAD]>,
    ptr0: *const T0,
    ptr1: *const T1,
    phantom0: PhantomData<&'a T0>,
    phantom1: PhantomData<&'a T1>,
}

impl<I, T0, T1, const PAD: usize> std::fmt::Debug for VulkanSlice2<'_, I, T0, T1, PAD>
where
    I: TryInto<usize> + Copy,
    T0: std::fmt::Debug,
    T1: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.len.try_into().unwrap_or(0);
        let slice = unsafe { std::slice::from_raw_parts(self.ptr0, len) };
        f.debug_list().entries(slice).finish()?;
        let slice = unsafe { std::slice::from_raw_parts(self.ptr1, len) };
        f.debug_list().entries(slice).finish()
    }
}

impl<I: Default, T0, T1, const PAD: usize> Default for VulkanSlice2<'_, I, T0, T1, PAD> {
    fn default() -> Self {
        Self {
            len: Default::default(),
            _pad: MaybeUninit::uninit(),
            ptr0: std::ptr::null(),
            ptr1: std::ptr::null(),
            phantom0: PhantomData,
            phantom1: PhantomData,
        }
    }
}

impl<'a, I, T0, T1, const PAD: usize> From<(&'a [T0], &'a [T1])>
    for VulkanSlice2<'a, I, T0, T1, PAD>
where
    I: TryFrom<usize>,
{
    fn from(x: (&'a [T0], &'a [T1])) -> Self {
        assert_eq!(x.0.len(), x.1.len());
        let Ok(len) = I::try_from(x.0.len()) else {
            panic!("invalid slice length")
        };
        let ptr0 = x.0.as_ptr();
        let ptr1 = x.1.as_ptr();
        Self {
            len,
            _pad: MaybeUninit::uninit(),
            ptr0,
            ptr1,
            phantom0: PhantomData,
            phantom1: PhantomData,
        }
    }
}

impl<'a, I, T0, T1, const N: usize, const PAD: usize> From<(&'a [T0; N], &'a [T1; N])>
    for VulkanSlice2<'a, I, T0, T1, PAD>
where
    I: TryFrom<usize>,
{
    fn from(x: (&'a [T0; N], &'a [T1; N])) -> Self {
        let Ok(len) = I::try_from(N) else {
            panic!("invalid slice length")
        };
        let ptr0 = x.0.as_ptr();
        let ptr1 = x.1.as_ptr();
        Self {
            len,
            _pad: MaybeUninit::uninit(),
            ptr0,
            ptr1,
            phantom0: PhantomData,
            phantom1: PhantomData,
        }
    }
}

impl<'a, I, T0, T1, const PAD: usize> From<(&'a mut [T0], &'a mut [T1])>
    for VulkanSlice2<'a, I, T0, T1, PAD>
where
    I: TryFrom<usize>,
{
    #[inline(always)]
    fn from(x: (&'a mut [T0], &'a mut [T1])) -> Self {
        (x.0 as &[T0], x.1 as &[T1]).into()
    }
}

impl<'a, I, T0, T1, const N: usize, const PAD: usize> From<(&'a mut [T0; N], &'a mut [T1; N])>
    for VulkanSlice2<'a, I, T0, T1, PAD>
where
    I: TryFrom<usize>,
{
    #[inline(always)]
    fn from(x: (&'a mut [T0; N], &'a mut [T1; N])) -> Self {
        (x.0 as &[T0; N], x.1 as &[T1; N]).into()
    }
}

fn vulkan_instance_version_not_supported() {
    panic!("calling an instance function not supported by the version requested in `InstanceFunctions::new`")
}

fn vulkan_device_version_not_supported() {
    panic!("calling a device function not supported by the version requested in `DeviceFunctions::new`")
}

pub struct GlobalFunctions {
    get_instance_proc_addr: FnGetInstanceProcAddr,
    enumerate_instance_version: Option<FnEnumerateInstanceVersion>,
    enumerate_instance_extension_properties: FnEnumerateInstanceExtensionProperties,
    enumerate_instance_layer_properties: FnEnumerateInstanceLayerProperties,
    create_instance: FnCreateInstance,
}

impl GlobalFunctions {
    pub unsafe fn new(get_proc_addr: *mut c_void) -> Self {
        let get_instance_proc_addr = transmute::<_, FnGetInstanceProcAddr>(get_proc_addr);
        Self {
            get_instance_proc_addr,
            enumerate_instance_version: transmute::<_, _>(get_instance_proc_addr(
                Instance::null(),
                c"vkEnumerateInstanceVersion".as_ptr(),
            )),
            enumerate_instance_extension_properties: transmute::<_, _>(get_instance_proc_addr(
                Instance::null(),
                c"vkEnumerateInstanceExtensionProperties".as_ptr(),
            )),
            enumerate_instance_layer_properties: transmute::<_, _>(get_instance_proc_addr(
                Instance::null(),
                c"vkEnumerateInstanceLayerProperties".as_ptr(),
            )),
            create_instance: transmute::<_, _>(
                get_instance_proc_addr(Instance::null(), c"vkCreateInstance".as_ptr())
                    .expect("failed to load vkCreateInstance"),
            ),
        }
    }

    #[inline]
    pub unsafe fn get_instance_proc_addr(
        &self,
        instance: Instance,
        name: &CStr,
    ) -> Option<FnVoidFunction> {
        (self.get_instance_proc_addr)(instance, name.as_ptr())
    }

    #[inline]
    pub fn enumerate_instance_version(&self, api_version: &mut u32) -> Result {
        if let Some(enumerate_instance_version) = self.enumerate_instance_version {
            enumerate_instance_version(api_version)
        } else {
            *api_version = VERSION_1_0;
            Result::Success
        }
    }

    #[inline]
    pub unsafe fn enumerate_instance_extension_properties(
        &self,
        layer_name: *const c_char,
        property_count: &mut u32,
        properties: *mut ExtensionProperties,
    ) -> Result {
        (self.enumerate_instance_extension_properties)(layer_name, property_count, properties)
    }

    #[inline]
    pub unsafe fn enumerate_instance_layer_properties(
        &self,
        property_count: &mut u32,
        properties: *mut LayerProperties,
    ) -> Result {
        (self.enumerate_instance_layer_properties)(property_count, properties)
    }

    #[inline]
    pub unsafe fn create_instance(
        &self,
        create_info: &InstanceCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        instance: &mut Instance,
    ) -> Result {
        (self.create_instance)(create_info, allocator, instance)
    }
}

pub struct InstanceFunctions {
    destroy_instance: FnDestroyInstance,
    enumerate_physical_devices: FnEnumeratePhysicalDevices,
    enumerate_device_extension_properties: FnEnumerateDeviceExtensionProperties,
    get_physical_device_features: FnGetPhysicalDeviceFeatures,
    get_physical_device_properties: FnGetPhysicalDeviceProperties,
    get_physical_device_queue_family_properties: FnGetPhysicalDeviceQueueFamilyProperties,
    get_physical_device_memory_properties: FnGetPhysicalDeviceMemoryProperties,
    create_device: FnCreateDevice,
    get_device_proc_addr: FnGetDeviceProcAddr,

    // VERSION_1_1
    get_physical_device_features2: FnGetPhysicalDeviceFeatures2,
    get_physical_device_properties2: FnGetPhysicalDeviceProperties2,
}

impl InstanceFunctions {
    pub fn new(global_functions: &GlobalFunctions, instance: Instance, api_version: u32) -> Self {
        unsafe {
            let load = |name: &CStr, function_version| {
                if api_version >= function_version {
                    global_functions
                        .get_instance_proc_addr(instance, name)
                        .unwrap_or_else(
                            #[cold]
                            || {
                                panic!(
                                    "failed to load instance function {}",
                                    name.to_string_lossy()
                                )
                            },
                        )
                } else {
                    transmute::<_, _>(vulkan_instance_version_not_supported as fn())
                }
            };

            Self {
                destroy_instance: transmute::<_, _>(load(c"vkDestroyInstance", VERSION_1_0)),
                enumerate_physical_devices: transmute::<_, _>(load(
                    c"vkEnumeratePhysicalDevices",
                    VERSION_1_0,
                )),
                enumerate_device_extension_properties: transmute::<_, _>(load(
                    c"vkEnumerateDeviceExtensionProperties",
                    VERSION_1_0,
                )),
                get_physical_device_features: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceFeatures",
                    VERSION_1_0,
                )),
                get_physical_device_properties: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceProperties",
                    VERSION_1_0,
                )),
                get_physical_device_queue_family_properties: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceQueueFamilyProperties",
                    VERSION_1_0,
                )),
                get_physical_device_memory_properties: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceMemoryProperties",
                    VERSION_1_0,
                )),
                create_device: transmute::<_, _>(load(c"vkCreateDevice", VERSION_1_0)),
                get_device_proc_addr: transmute::<_, _>(load(c"vkGetDeviceProcAddr", VERSION_1_0)),

                // VERSION_1_1
                get_physical_device_features2: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceFeatures2",
                    VERSION_1_1,
                )),
                get_physical_device_properties2: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceProperties2",
                    VERSION_1_1,
                )),
            }
        }
    }

    #[inline]
    pub unsafe fn destroy_instance(
        &self,
        instance: Instance,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_instance)(instance, allocator)
    }

    #[inline]
    pub unsafe fn enumerate_physical_devices(
        &self,
        instance: Instance,
        physical_device_count: &mut u32,
        physical_devices: *mut PhysicalDevice,
    ) -> Result {
        (self.enumerate_physical_devices)(instance, physical_device_count, physical_devices)
    }

    #[inline]
    pub unsafe fn enumerate_device_extension_properties(
        &self,
        physical_device: PhysicalDevice,
        layer_name: *const c_char,
        property_count: &mut u32,
        properties: *mut ExtensionProperties,
    ) -> Result {
        (self.enumerate_device_extension_properties)(
            physical_device,
            layer_name,
            property_count,
            properties,
        )
    }

    #[inline]
    pub unsafe fn get_physical_device_features(
        &self,
        physical_device: PhysicalDevice,
        features: *mut PhysicalDeviceFeatures,
    ) {
        (self.get_physical_device_features)(physical_device, features)
    }

    #[inline]
    pub unsafe fn get_physical_device_features2(
        &self,
        physical_device: PhysicalDevice,
        features: *mut PhysicalDeviceFeatures2,
    ) {
        (self.get_physical_device_features2)(physical_device, features)
    }

    #[inline]
    pub unsafe fn get_physical_device_properties(
        &self,
        physical_device: PhysicalDevice,
        properties: *mut PhysicalDeviceProperties,
    ) {
        (self.get_physical_device_properties)(physical_device, properties)
    }

    #[inline]
    pub unsafe fn get_physical_device_properties2(
        &self,
        physical_device: PhysicalDevice,
        properties: *mut PhysicalDeviceProperties2,
    ) {
        (self.get_physical_device_properties2)(physical_device, properties)
    }

    #[inline]
    pub unsafe fn get_physical_device_queue_family_properties(
        &self,
        physical_device: PhysicalDevice,
        queue_family_property_count: &mut u32,
        queue_family_properties: *mut QueueFamilyProperties,
    ) {
        (self.get_physical_device_queue_family_properties)(
            physical_device,
            queue_family_property_count,
            queue_family_properties,
        )
    }

    #[inline]
    pub unsafe fn get_physical_device_memory_properties(
        &self,
        physical_device: PhysicalDevice,
        memory_properties: *mut PhysicalDeviceMemoryProperties,
    ) {
        (self.get_physical_device_memory_properties)(physical_device, memory_properties)
    }

    #[inline]
    pub unsafe fn create_device(
        &self,
        physical_device: PhysicalDevice,
        create_info: &DeviceCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        device: &mut Device,
    ) -> Result {
        (self.create_device)(physical_device, create_info, allocator, device)
    }

    #[inline]
    pub unsafe fn get_device_proc_addr(
        &self,
        device: Device,
        name: &CStr,
    ) -> Option<FnVoidFunction> {
        (self.get_device_proc_addr)(device, name.as_ptr())
    }
}

pub struct DeviceFunctions {
    destroy_device: FnDestroyDevice,
    get_device_queue: FnGetDeviceQueue,
    queue_submit: FnQueueSubmit,
    queue_wait_idle: FnQueueWaitIdle,
    device_wait_idle: FnDeviceWaitIdle,
    allocate_memory: FnAllocateMemory,
    free_memory: FnFreeMemory,
    map_memory: FnMapMemory,
    unmap_memory: FnUnmapMemory,
    create_buffer: FnCreateBuffer,
    destroy_buffer: FnDestroyBuffer,
    create_buffer_view: FnCreateBufferView,
    destroy_buffer_view: FnDestroyBufferView,
    create_image: FnCreateImage,
    destroy_image: FnDestroyImage,
    get_image_subresource_layout: FnGetImageSubresourceLayout,
    create_image_view: FnCreateImageView,
    destroy_image_view: FnDestroyImageView,
    create_command_pool: FnCreateCommandPool,
    destroy_command_pool: FnDestroyCommandPool,
    reset_command_pool: FnResetCommandPool,
    allocate_command_buffers: FnAllocateCommandBuffers,
    free_command_buffers: FnFreeCommandBuffers,
    begin_command_buffer: FnBeginCommandBuffer,
    end_command_buffer: FnEndCommandBuffer,
    reset_command_buffer: FnResetCommandBuffer,
    create_framebuffer: FnCreateFramebuffer,
    destroy_framebuffer: FnDestroyFramebuffer,
    create_render_pass: FnCreateRenderPass,
    destroy_render_pass: FnDestroyRenderPass,
    create_semaphore: FnCreateSemaphore,
    destroy_semaphore: FnDestroySemaphore,
    wait_semaphores: FnWaitSemaphores,
    signal_semaphore: FnSignalSemaphore,
    create_fence: FnCreateFence,
    destroy_fence: FnDestroyFence,
    reset_fences: FnResetFences,
    get_fence_status: FnGetFenceStatus,
    wait_for_fences: FnWaitForFences,
    invalidate_mapped_memory_ranges: FnInvalidateMappedMemoryRanges,
    create_shader_module: FnCreateShaderModule,
    destroy_shader_module: FnDestroyShaderModule,
    create_sampler: FnCreateSampler,
    destroy_sampler: FnDestroySampler,
    create_descriptor_set_layout: FnCreateDescriptorSetLayout,
    destroy_descriptor_set_layout: FnDestroyDescriptorSetLayout,
    create_descriptor_pool: FnCreateDescriptorPool,
    destroy_descriptor_pool: FnDestroyDescriptorPool,
    reset_descriptor_pool: FnResetDescriptorPool,
    allocate_descriptor_sets: FnAllocateDescriptorSets,
    free_descriptor_sets: FnFreeDescriptorSets,
    update_descriptor_sets: FnUpdateDescriptorSets,
    create_pipeline_layout: FnCreatePipelineLayout,
    destroy_pipeline_layout: FnDestroyPipelineLayout,
    create_graphics_pipelines: FnCreateGraphicsPipelines,
    create_compute_pipelines: FnCreateComputePipelines,
    destroy_pipeline: FnDestroyPipeline,
    cmd_bind_pipeline: FnCmdBindPipeline,
    cmd_set_viewport: FnCmdSetViewport,
    cmd_set_scissor: FnCmdSetScissor,
    cmd_set_line_width: FnCmdSetLineWidth,
    cmd_set_depth_bias: FnCmdSetDepthBias,
    cmd_set_blend_constants: FnCmdSetBlendConstants,
    cmd_set_depth_bounds: FnCmdSetDepthBounds,
    cmd_set_stencil_compare_mask: FnCmdSetStencilCompareMask,
    cmd_set_stencil_write_mask: FnCmdSetStencilWriteMask,
    cmd_set_stencil_reference: FnCmdSetStencilReference,
    cmd_bind_descriptor_sets: FnCmdBindDescriptorSets,
    cmd_bind_index_buffer: FnCmdBindIndexBuffer,
    cmd_bind_vertex_buffers: FnCmdBindVertexBuffers,
    cmd_draw: FnCmdDraw,
    cmd_draw_indexed: FnCmdDrawIndexed,
    cmd_draw_indirect: FnCmdDrawIndirect,
    cmd_draw_indexed_indirect: FnCmdDrawIndexedIndirect,
    cmd_dispatch: FnCmdDispatch,
    cmd_dispatch_indirect: FnCmdDispatchIndirect,
    cmd_copy_buffer: FnCmdCopyBuffer,
    cmd_copy_image: FnCmdCopyImage,
    cmd_blit_image: FnCmdBlitImage,
    cmd_copy_buffer_to_image: FnCmdCopyBufferToImage,
    cmd_copy_image_to_buffer: FnCmdCopyImageToBuffer,
    cmd_update_buffer: FnCmdUpdateBuffer,
    cmd_fill_buffer: FnCmdFillBuffer,
    cmd_clear_color_image: FnCmdClearColorImage,
    cmd_clear_depth_stencil_image: FnCmdClearDepthStencilImage,
    cmd_clear_attachments: FnCmdClearAttachments,
    cmd_resolve_image: FnCmdResolveImage,
    cmd_set_event: FnCmdSetEvent,
    cmd_reset_event: FnCmdResetEvent,
    cmd_wait_events: FnCmdWaitEvents,
    cmd_pipeline_barrier: FnCmdPipelineBarrier,
    cmd_begin_query: FnCmdBeginQuery,
    cmd_end_query: FnCmdEndQuery,
    cmd_reset_query_pool: FnCmdResetQueryPool,
    cmd_write_timestamp: FnCmdWriteTimestamp,
    cmd_copy_query_pool_results: FnCmdCopyQueryPoolResults,
    cmd_push_constants: FnCmdPushConstants,
    cmd_begin_render_pass: FnCmdBeginRenderPass,
    cmd_next_subpass: FnCmdNextSubpass,
    cmd_end_render_pass: FnCmdEndRenderPass,
    cmd_execute_commands: FnCmdExecuteCommands,

    // VERSION_1_1
    get_image_memory_requirements2: FnGetImageMemoryRequirements2,
    bind_image_memory2: FnBindImageMemory2,
    get_buffer_memory_requirements2: FnGetBufferMemoryRequirements2,
    bind_buffer_memory2: FnBindBufferMemory2,

    // VERSION_1_2
    get_buffer_device_address: FnGetBufferDeviceAddress,
    get_semaphore_counter_value: FnGetSemaphoreCounterValue,

    // VERSION_1_3
    cmd_pipeline_barrier2: FnCmdPipelineBarrier2,
    cmd_wait_events2: FnCmdWaitEvents2,
    cmd_set_event2: FnCmdSetEvent2,
    cmd_begin_rendering: FnCmdBeginRendering,
    cmd_end_rendering: FnCmdEndRendering,
    cmd_set_viewport_with_count: FnCmdSetViewportWithCount,
    cmd_set_scissor_with_count: FnCmdSetScissorWithCount,
    queue_submit2: FnQueueSubmit2,
}

impl DeviceFunctions {
    pub fn new(instance_functons: &InstanceFunctions, device: Device, api_version: u32) -> Self {
        unsafe {
            let load = |name: &CStr, function_version| {
                if api_version >= function_version {
                    instance_functons
                        .get_device_proc_addr(device, name)
                        .unwrap_or_else(
                            #[cold]
                            || panic!("failed to load device function {}", name.to_string_lossy()),
                        )
                } else {
                    transmute::<_, _>(vulkan_device_version_not_supported as fn())
                }
            };

            Self {
                destroy_device: transmute::<_, _>(load(c"vkDestroyDevice", VERSION_1_0)),
                get_device_queue: transmute::<_, _>(load(c"vkGetDeviceQueue", VERSION_1_0)),
                queue_submit: transmute::<_, _>(load(c"vkQueueSubmit", VERSION_1_0)),
                queue_wait_idle: transmute::<_, _>(load(c"vkQueueWaitIdle", VERSION_1_0)),
                device_wait_idle: transmute::<_, _>(load(c"vkDeviceWaitIdle", VERSION_1_0)),
                allocate_memory: transmute::<_, _>(load(c"vkAllocateMemory", VERSION_1_0)),
                free_memory: transmute::<_, _>(load(c"vkFreeMemory", VERSION_1_0)),
                map_memory: transmute::<_, _>(load(c"vkMapMemory", VERSION_1_0)),
                unmap_memory: transmute::<_, _>(load(c"vkUnmapMemory", VERSION_1_0)),
                create_buffer: transmute::<_, _>(load(c"vkCreateBuffer", VERSION_1_0)),
                destroy_buffer: transmute::<_, _>(load(c"vkDestroyBuffer", VERSION_1_0)),
                create_buffer_view: transmute::<_, _>(load(c"vkCreateBufferView", VERSION_1_0)),
                destroy_buffer_view: transmute::<_, _>(load(c"vkDestroyBufferView", VERSION_1_0)),
                create_image: transmute::<_, _>(load(c"vkCreateImage", VERSION_1_0)),
                destroy_image: transmute::<_, _>(load(c"vkDestroyImage", VERSION_1_0)),
                get_image_subresource_layout: transmute::<_, _>(load(
                    c"vkGetImageSubresourceLayout",
                    VERSION_1_0,
                )),
                create_image_view: transmute::<_, _>(load(c"vkCreateImageView", VERSION_1_0)),
                destroy_image_view: transmute::<_, _>(load(c"vkDestroyImageView", VERSION_1_0)),
                create_command_pool: transmute::<_, _>(load(c"vkCreateCommandPool", VERSION_1_0)),
                destroy_command_pool: transmute::<_, _>(load(c"vkDestroyCommandPool", VERSION_1_0)),
                reset_command_pool: transmute::<_, _>(load(c"vkResetCommandPool", VERSION_1_0)),
                allocate_command_buffers: transmute::<_, _>(load(
                    c"vkAllocateCommandBuffers",
                    VERSION_1_0,
                )),
                free_command_buffers: transmute::<_, _>(load(c"vkFreeCommandBuffers", VERSION_1_0)),
                begin_command_buffer: transmute::<_, _>(load(c"vkBeginCommandBuffer", VERSION_1_0)),
                end_command_buffer: transmute::<_, _>(load(c"vkEndCommandBuffer", VERSION_1_0)),
                reset_command_buffer: transmute::<_, _>(load(c"vkResetCommandBuffer", VERSION_1_0)),
                create_framebuffer: transmute::<_, _>(load(c"vkCreateFramebuffer", VERSION_1_0)),
                destroy_framebuffer: transmute::<_, _>(load(c"vkDestroyFramebuffer", VERSION_1_0)),
                create_render_pass: transmute::<_, _>(load(c"vkCreateRenderPass", VERSION_1_0)),
                destroy_render_pass: transmute::<_, _>(load(c"vkDestroyRenderPass", VERSION_1_0)),
                create_semaphore: transmute::<_, _>(load(c"vkCreateSemaphore", VERSION_1_0)),
                destroy_semaphore: transmute::<_, _>(load(c"vkDestroySemaphore", VERSION_1_0)),

                wait_semaphores: transmute::<_, _>(load(c"vkWaitSemaphores", VERSION_1_0)),
                signal_semaphore: transmute::<_, _>(load(c"vkSignalSemaphore", VERSION_1_0)),
                create_fence: transmute::<_, _>(load(c"vkCreateFence", VERSION_1_0)),
                destroy_fence: transmute::<_, _>(load(c"vkDestroyFence", VERSION_1_0)),
                reset_fences: transmute::<_, _>(load(c"vkResetFences", VERSION_1_0)),
                get_fence_status: transmute::<_, _>(load(c"vkGetFenceStatus", VERSION_1_0)),
                wait_for_fences: transmute::<_, _>(load(c"vkWaitForFences", VERSION_1_0)),
                invalidate_mapped_memory_ranges: transmute::<_, _>(load(
                    c"vkInvalidateMappedMemoryRanges",
                    VERSION_1_0,
                )),
                create_shader_module: transmute::<_, _>(load(c"vkCreateShaderModule", VERSION_1_0)),
                destroy_shader_module: transmute::<_, _>(load(
                    c"vkDestroyShaderModule",
                    VERSION_1_0,
                )),
                create_sampler: transmute::<_, _>(load(c"vkCreateSampler", VERSION_1_0)),
                destroy_sampler: transmute::<_, _>(load(c"vkDestroySampler", VERSION_1_0)),
                create_descriptor_set_layout: transmute::<_, _>(load(
                    c"vkCreateDescriptorSetLayout",
                    VERSION_1_0,
                )),
                destroy_descriptor_set_layout: transmute::<_, _>(load(
                    c"vkDestroyDescriptorSetLayout",
                    VERSION_1_0,
                )),
                create_descriptor_pool: transmute::<_, _>(load(
                    c"vkCreateDescriptorPool",
                    VERSION_1_0,
                )),
                destroy_descriptor_pool: transmute::<_, _>(load(
                    c"vkDestroyDescriptorPool",
                    VERSION_1_0,
                )),
                reset_descriptor_pool: transmute::<_, _>(load(
                    c"vkResetDescriptorPool",
                    VERSION_1_0,
                )),
                allocate_descriptor_sets: transmute::<_, _>(load(
                    c"vkAllocateDescriptorSets",
                    VERSION_1_0,
                )),
                free_descriptor_sets: transmute::<_, _>(load(c"vkFreeDescriptorSets", VERSION_1_0)),
                update_descriptor_sets: transmute::<_, _>(load(
                    c"vkUpdateDescriptorSets",
                    VERSION_1_0,
                )),
                create_pipeline_layout: transmute::<_, _>(load(
                    c"vkCreatePipelineLayout",
                    VERSION_1_0,
                )),
                destroy_pipeline_layout: transmute::<_, _>(load(
                    c"vkDestroyPipelineLayout",
                    VERSION_1_0,
                )),
                create_graphics_pipelines: transmute::<_, _>(load(
                    c"vkCreateGraphicsPipelines",
                    VERSION_1_0,
                )),
                create_compute_pipelines: transmute::<_, _>(load(
                    c"vkCreateComputePipelines",
                    VERSION_1_0,
                )),
                destroy_pipeline: transmute::<_, _>(load(c"vkDestroyPipeline", VERSION_1_0)),
                cmd_bind_pipeline: transmute::<_, _>(load(c"vkCmdBindPipeline", VERSION_1_0)),
                cmd_set_viewport: transmute::<_, _>(load(c"vkCmdSetViewport", VERSION_1_0)),
                cmd_set_scissor: transmute::<_, _>(load(c"vkCmdSetScissor", VERSION_1_0)),
                cmd_set_line_width: transmute::<_, _>(load(c"vkCmdSetLineWidth", VERSION_1_0)),
                cmd_set_depth_bias: transmute::<_, _>(load(c"vkCmdSetDepthBias", VERSION_1_0)),
                cmd_set_blend_constants: transmute::<_, _>(load(
                    c"vkCmdSetBlendConstants",
                    VERSION_1_0,
                )),
                cmd_set_depth_bounds: transmute::<_, _>(load(c"vkCmdSetDepthBounds", VERSION_1_0)),
                cmd_set_stencil_compare_mask: transmute::<_, _>(load(
                    c"vkCmdSetStencilCompareMask",
                    VERSION_1_0,
                )),
                cmd_set_stencil_write_mask: transmute::<_, _>(load(
                    c"vkCmdSetStencilWriteMask",
                    VERSION_1_0,
                )),
                cmd_set_stencil_reference: transmute::<_, _>(load(
                    c"vkCmdSetStencilReference",
                    VERSION_1_0,
                )),
                cmd_bind_descriptor_sets: transmute::<_, _>(load(
                    c"vkCmdBindDescriptorSets",
                    VERSION_1_0,
                )),
                cmd_bind_index_buffer: transmute::<_, _>(load(
                    c"vkCmdBindIndexBuffer",
                    VERSION_1_0,
                )),
                cmd_bind_vertex_buffers: transmute::<_, _>(load(
                    c"vkCmdBindVertexBuffers",
                    VERSION_1_0,
                )),
                cmd_draw: transmute::<_, _>(load(c"vkCmdDraw", VERSION_1_0)),
                cmd_draw_indexed: transmute::<_, _>(load(c"vkCmdDrawIndexed", VERSION_1_0)),
                cmd_draw_indirect: transmute::<_, _>(load(c"vkCmdDrawIndirect", VERSION_1_0)),
                cmd_draw_indexed_indirect: transmute::<_, _>(load(
                    c"vkCmdDrawIndexedIndirect",
                    VERSION_1_0,
                )),
                cmd_dispatch: transmute::<_, _>(load(c"vkCmdDispatch", VERSION_1_0)),
                cmd_dispatch_indirect: transmute::<_, _>(load(
                    c"vkCmdDispatchIndirect",
                    VERSION_1_0,
                )),
                cmd_copy_buffer: transmute::<_, _>(load(c"vkCmdCopyBuffer", VERSION_1_0)),
                cmd_copy_image: transmute::<_, _>(load(c"vkCmdCopyImage", VERSION_1_0)),
                cmd_blit_image: transmute::<_, _>(load(c"vkCmdBlitImage", VERSION_1_0)),
                cmd_copy_buffer_to_image: transmute::<_, _>(load(
                    c"vkCmdCopyBufferToImage",
                    VERSION_1_0,
                )),
                cmd_copy_image_to_buffer: transmute::<_, _>(load(
                    c"vkCmdCopyImageToBuffer",
                    VERSION_1_0,
                )),
                cmd_update_buffer: transmute::<_, _>(load(c"vkCmdUpdateBuffer", VERSION_1_0)),
                cmd_fill_buffer: transmute::<_, _>(load(c"vkCmdFillBuffer", VERSION_1_0)),
                cmd_clear_color_image: transmute::<_, _>(load(
                    c"vkCmdClearColorImage",
                    VERSION_1_0,
                )),
                cmd_clear_depth_stencil_image: transmute::<_, _>(load(
                    c"vkCmdClearDepthStencilImage",
                    VERSION_1_0,
                )),
                cmd_clear_attachments: transmute::<_, _>(load(
                    c"vkCmdClearAttachments",
                    VERSION_1_0,
                )),
                cmd_resolve_image: transmute::<_, _>(load(c"vkCmdResolveImage", VERSION_1_0)),
                cmd_set_event: transmute::<_, _>(load(c"vkCmdSetEvent", VERSION_1_0)),
                cmd_reset_event: transmute::<_, _>(load(c"vkCmdResetEvent", VERSION_1_0)),
                cmd_wait_events: transmute::<_, _>(load(c"vkCmdWaitEvents", VERSION_1_0)),
                cmd_pipeline_barrier: transmute::<_, _>(load(c"vkCmdPipelineBarrier", VERSION_1_0)),
                cmd_begin_query: transmute::<_, _>(load(c"vkCmdBeginQuery", VERSION_1_0)),
                cmd_end_query: transmute::<_, _>(load(c"vkCmdEndQuery", VERSION_1_0)),
                cmd_reset_query_pool: transmute::<_, _>(load(c"vkCmdResetQueryPool", VERSION_1_0)),
                cmd_write_timestamp: transmute::<_, _>(load(c"vkCmdWriteTimestamp", VERSION_1_0)),
                cmd_copy_query_pool_results: transmute::<_, _>(load(
                    c"vkCmdCopyQueryPoolResults",
                    VERSION_1_0,
                )),
                cmd_push_constants: transmute::<_, _>(load(c"vkCmdPushConstants", VERSION_1_0)),
                cmd_begin_render_pass: transmute::<_, _>(load(
                    c"vkCmdBeginRenderPass",
                    VERSION_1_0,
                )),
                cmd_next_subpass: transmute::<_, _>(load(c"vkCmdNextSubpass", VERSION_1_0)),
                cmd_end_render_pass: transmute::<_, _>(load(c"vkCmdEndRenderPass", VERSION_1_0)),
                cmd_execute_commands: transmute::<_, _>(load(c"vkCmdExecuteCommands", VERSION_1_0)),

                // VERSION_1_1
                get_image_memory_requirements2: transmute::<_, _>(load(
                    c"vkGetImageMemoryRequirements2",
                    VERSION_1_1,
                )),
                bind_image_memory2: transmute::<_, _>(load(c"vkBindImageMemory2", VERSION_1_1)),
                get_buffer_memory_requirements2: transmute::<_, _>(load(
                    c"vkGetBufferMemoryRequirements2",
                    VERSION_1_1,
                )),
                bind_buffer_memory2: transmute::<_, _>(load(c"vkBindBufferMemory2", VERSION_1_1)),

                // VERSION_1_2
                get_buffer_device_address: transmute::<_, _>(load(
                    c"vkGetBufferDeviceAddress",
                    VERSION_1_2,
                )),

                get_semaphore_counter_value: transmute::<_, _>(load(
                    c"vkGetSemaphoreCounterValue",
                    VERSION_1_2,
                )),

                // VERSION_1_3
                cmd_pipeline_barrier2: transmute::<_, _>(load(
                    c"vkCmdPipelineBarrier2",
                    VERSION_1_3,
                )),
                cmd_wait_events2: transmute::<_, _>(load(c"vkCmdWaitEvents2", VERSION_1_3)),
                cmd_set_event2: transmute::<_, _>(load(c"vkCmdSetEvent2", VERSION_1_3)),

                cmd_begin_rendering: transmute::<_, _>(load(c"vkCmdBeginRendering", VERSION_1_3)),
                cmd_end_rendering: transmute::<_, _>(load(c"vkCmdEndRendering", VERSION_1_3)),
                cmd_set_viewport_with_count: transmute::<_, _>(load(
                    c"vkCmdSetViewportWithCount",
                    VERSION_1_3,
                )),
                cmd_set_scissor_with_count: transmute::<_, _>(load(
                    c"vkCmdSetScissorWithCount",
                    VERSION_1_3,
                )),
                queue_submit2: transmute::<_, _>(load(c"vkQueueSubmit2", VERSION_1_3)),
            }
        }
    }

    #[inline]
    pub unsafe fn destroy_device(&self, device: Device, allocator: Option<&AllocationCallbacks>) {
        (self.destroy_device)(device, allocator)
    }

    #[inline]
    pub unsafe fn get_device_queue(
        &self,
        device: Device,
        queue_family_index: u32,
        queue_index: u32,
        queue: &mut Queue,
    ) {
        (self.get_device_queue)(device, queue_family_index, queue_index, queue)
    }

    #[inline]
    pub unsafe fn queue_submit(
        &self,
        queue: Queue,
        submits: &[SubmitInfo],
        fence: Fence,
    ) -> Result {
        (self.queue_submit)(queue, submits.len() as u32, submits.as_ptr(), fence)
    }

    #[inline]
    pub unsafe fn queue_submit2(
        &self,
        queue: Queue,
        submits: &[SubmitInfo2],
        fence: Fence,
    ) -> Result {
        (self.queue_submit2)(queue, submits.len() as u32, submits.as_ptr(), fence)
    }

    #[inline]
    pub unsafe fn allocate_memory(
        &self,
        device: Device,
        allocate_info: &MemoryAllocateInfo,
        allocator: Option<&AllocationCallbacks>,
        memory: &mut DeviceMemory,
    ) -> Result {
        (self.allocate_memory)(device, allocate_info, allocator, memory)
    }

    #[inline]
    pub unsafe fn free_memory(
        &self,
        device: Device,
        memory: DeviceMemory,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.free_memory)(device, memory, allocator)
    }

    #[inline]
    pub unsafe fn map_memory(
        &self,
        device: Device,
        memory: DeviceMemory,
        offset: DeviceSize,
        size: DeviceSize,
        flags: MemoryMapFlags,
        data: &mut *mut c_void,
    ) -> Result {
        (self.map_memory)(device, memory, offset, size, flags, data)
    }

    #[inline]
    pub unsafe fn unmap_memory(&self, device: Device, memory: DeviceMemory) {
        (self.unmap_memory)(device, memory)
    }

    #[inline]
    pub unsafe fn create_buffer(
        &self,
        device: Device,
        create_info: &BufferCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        buffer: &mut Buffer,
    ) -> Result {
        (self.create_buffer)(device, create_info, allocator, buffer)
    }

    #[inline]
    pub unsafe fn destroy_buffer(
        &self,
        device: Device,
        buffer: Buffer,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_buffer)(device, buffer, allocator)
    }

    #[inline]
    pub unsafe fn create_buffer_view(
        &self,
        device: Device,
        create_info: &BufferViewCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        view: &mut BufferView,
    ) -> Result {
        (self.create_buffer_view)(device, create_info, allocator, view)
    }

    #[inline]
    pub unsafe fn destroy_buffer_view(
        &self,
        device: Device,
        buffer_view: BufferView,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_buffer_view)(device, buffer_view, allocator)
    }

    #[inline]
    pub unsafe fn create_image(
        &self,
        device: Device,
        create_info: &ImageCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        image: &mut Image,
    ) -> Result {
        (self.create_image)(device, create_info, allocator, image)
    }

    #[inline]
    pub unsafe fn destroy_image(
        &self,
        device: Device,
        image: Image,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_image)(device, image, allocator)
    }

    #[inline]
    pub unsafe fn get_image_subresource_layout(
        &self,
        device: Device,
        image: Image,
        subresource: &ImageSubresource,
        layout: &mut SubresourceLayout,
    ) {
        (self.get_image_subresource_layout)(device, image, subresource, layout)
    }

    #[inline]
    pub unsafe fn create_image_view(
        &self,
        device: Device,
        create_info: &ImageViewCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        view: &mut ImageView,
    ) -> Result {
        (self.create_image_view)(device, create_info, allocator, view)
    }

    #[inline]
    pub unsafe fn destroy_image_view(
        &self,
        device: Device,
        image_view: ImageView,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_image_view)(device, image_view, allocator)
    }

    #[inline]
    pub unsafe fn create_render_pass(
        &self,
        device: Device,
        create_info: &RenderPassCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        render_pass: &mut RenderPass,
    ) -> Result {
        (self.create_render_pass)(device, create_info, allocator, render_pass)
    }

    #[inline]
    pub unsafe fn destroy_render_pass(
        &self,
        device: Device,
        render_pass: RenderPass,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_render_pass)(device, render_pass, allocator)
    }

    #[inline]
    pub unsafe fn create_framebuffer(
        &self,
        device: Device,
        create_info: &FramebufferCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        framebuffer: &mut Framebuffer,
    ) -> Result {
        (self.create_framebuffer)(device, create_info, allocator, framebuffer)
    }

    #[inline]
    pub unsafe fn destroy_framebuffer(
        &self,
        device: Device,
        framebuffer: Framebuffer,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_framebuffer)(device, framebuffer, allocator)
    }

    #[inline]
    pub unsafe fn create_command_pool(
        &self,
        device: Device,
        create_info: &CommandPoolCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        command_pool: &mut CommandPool,
    ) -> Result {
        (self.create_command_pool)(device, create_info, allocator, command_pool)
    }

    #[inline]
    pub unsafe fn destroy_command_pool(
        &self,
        device: Device,
        command_pool: CommandPool,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_command_pool)(device, command_pool, allocator)
    }

    #[inline]
    pub unsafe fn reset_command_pool(
        &self,
        device: Device,
        command_pool: CommandPool,
        flags: CommandPoolResetFlags,
    ) -> Result {
        (self.reset_command_pool)(device, command_pool, flags)
    }

    #[inline]
    pub unsafe fn allocate_command_buffers(
        &self,
        device: Device,
        allocate_info: &CommandBufferAllocateInfo,
        command_buffers: *mut CommandBuffer,
    ) -> Result {
        (self.allocate_command_buffers)(device, allocate_info, command_buffers)
    }

    #[inline]
    pub unsafe fn free_command_buffers(
        &self,
        device: Device,
        command_pool: CommandPool,
        command_buffers: &[CommandBuffer],
    ) {
        (self.free_command_buffers)(
            device,
            command_pool,
            command_buffers.len() as u32,
            command_buffers.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn begin_command_buffer(
        &self,
        command_buffer: CommandBuffer,
        begin_info: &CommandBufferBeginInfo,
    ) -> Result {
        (self.begin_command_buffer)(command_buffer, begin_info)
    }

    #[inline]
    pub unsafe fn end_command_buffer(&self, command_buffer: CommandBuffer) -> Result {
        (self.end_command_buffer)(command_buffer)
    }

    #[inline]
    pub unsafe fn reset_command_buffer(
        &self,
        command_buffer: CommandBuffer,
        flags: CommandBufferResetFlags,
    ) -> Result {
        (self.reset_command_buffer)(command_buffer, flags)
    }

    #[inline]
    pub unsafe fn create_semaphore(
        &self,
        device: Device,
        create_info: &SemaphoreCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        semaphore: &mut Semaphore,
    ) -> Result {
        (self.create_semaphore)(device, create_info, allocator, semaphore)
    }

    #[inline]
    pub unsafe fn destroy_semaphore(
        &self,
        device: Device,
        semaphore: Semaphore,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_semaphore)(device, semaphore, allocator)
    }

    #[inline]
    pub unsafe fn get_semaphore_counter_value(
        &self,
        device: Device,
        semaphore: Semaphore,
        value: &mut u64,
    ) -> Result {
        (self.get_semaphore_counter_value)(device, semaphore, value)
    }

    #[inline]
    pub unsafe fn wait_semaphores(
        &self,
        device: Device,
        wait_info: &SemaphoreWaitInfo,
        timeout: u64,
    ) -> Result {
        (self.wait_semaphores)(device, wait_info, timeout)
    }

    #[inline]
    pub unsafe fn signal_semaphore(
        &self,
        device: Device,
        signal_info: &SemaphoreSignalInfo,
    ) -> Result {
        (self.signal_semaphore)(device, signal_info)
    }

    #[inline]
    pub unsafe fn create_fence(
        &self,
        device: Device,
        create_info: &FenceCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        fence: &mut Fence,
    ) -> Result {
        (self.create_fence)(device, create_info, allocator, fence)
    }

    #[inline]
    pub unsafe fn destroy_fence(
        &self,
        device: Device,
        fence: Fence,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_fence)(device, fence, allocator)
    }

    #[inline]
    pub unsafe fn reset_fences(&self, device: Device, fences: &[Fence]) -> Result {
        (self.reset_fences)(device, fences.len() as u32, fences.as_ptr())
    }

    #[inline]
    pub unsafe fn get_fence_status(&self, device: Device, fence: Fence) -> Result {
        (self.get_fence_status)(device, fence)
    }

    #[inline]
    pub unsafe fn wait_for_fences(
        &self,
        device: Device,
        fences: &[Fence],
        wait_all: Bool32,
        timeout: u64,
    ) -> Result {
        (self.wait_for_fences)(
            device,
            fences.len() as u32,
            fences.as_ptr(),
            wait_all,
            timeout,
        )
    }

    #[inline]
    pub unsafe fn invalidate_mapped_memory_ranges(
        &self,
        device: Device,
        memory_ranges: &[MappedMemoryRange],
    ) -> Result {
        (self.invalidate_mapped_memory_ranges)(
            device,
            memory_ranges.len() as u32,
            memory_ranges.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn create_shader_module(
        &self,
        device: Device,
        create_info: &ShaderModuleCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        shader_module: &mut ShaderModule,
    ) -> Result {
        (self.create_shader_module)(device, create_info, allocator, shader_module)
    }

    #[inline]
    pub unsafe fn destroy_shader_module(
        &self,
        device: Device,
        shader_module: ShaderModule,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_shader_module)(device, shader_module, allocator)
    }

    #[inline]
    pub unsafe fn create_sampler(
        &self,
        device: Device,
        create_info: &SamplerCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        sampler: &mut Sampler,
    ) -> Result {
        (self.create_sampler)(device, create_info, allocator, sampler)
    }

    #[inline]
    pub unsafe fn destroy_sampler(
        &self,
        device: Device,
        sampler: Sampler,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_sampler)(device, sampler, allocator)
    }

    #[inline]
    pub unsafe fn create_descriptor_set_layout(
        &self,
        device: Device,
        create_info: &DescriptorSetLayoutCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        set_layout: &mut DescriptorSetLayout,
    ) -> Result {
        (self.create_descriptor_set_layout)(device, create_info, allocator, set_layout)
    }

    #[inline]
    pub unsafe fn destroy_descriptor_set_layout(
        &self,
        device: Device,
        descriptor_set_layout: DescriptorSetLayout,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_descriptor_set_layout)(device, descriptor_set_layout, allocator)
    }

    #[inline]
    pub unsafe fn create_descriptor_pool(
        &self,
        device: Device,
        create_info: &DescriptorPoolCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        descriptor_pool: &mut DescriptorPool,
    ) -> Result {
        (self.create_descriptor_pool)(device, create_info, allocator, descriptor_pool)
    }

    #[inline]
    pub unsafe fn destroy_descriptor_pool(
        &self,
        device: Device,
        descriptor_pool: DescriptorPool,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_descriptor_pool)(device, descriptor_pool, allocator)
    }

    #[inline]
    pub unsafe fn reset_descriptor_pool(
        &self,
        device: Device,
        descriptor_pool: DescriptorPool,
        flags: DescriptorPoolResetFlags,
    ) -> Result {
        (self.reset_descriptor_pool)(device, descriptor_pool, flags)
    }

    #[inline]
    pub unsafe fn allocate_descriptor_sets(
        &self,
        device: Device,
        allocate_info: &DescriptorSetAllocateInfo,
        descriptor_sets: *mut DescriptorSet,
    ) -> Result {
        (self.allocate_descriptor_sets)(device, allocate_info, descriptor_sets)
    }

    #[inline]
    pub unsafe fn free_descriptor_sets(
        &self,
        device: Device,
        descriptor_pool: DescriptorPool,
        descriptor_set_count: u32,
        descriptor_sets: *const DescriptorSet,
    ) -> Result {
        (self.free_descriptor_sets)(
            device,
            descriptor_pool,
            descriptor_set_count,
            descriptor_sets,
        )
    }

    #[inline]
    pub unsafe fn update_descriptor_sets(
        &self,
        device: Device,
        descriptor_writes: &[WriteDescriptorSet],
        descriptor_copies: &[CopyDescriptorSet],
    ) {
        (self.update_descriptor_sets)(
            device,
            descriptor_writes.len() as u32,
            descriptor_writes.as_ptr(),
            descriptor_copies.len() as u32,
            descriptor_copies.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn create_pipeline_layout(
        &self,
        device: Device,
        create_info: &PipelineLayoutCreateInfo,
        allocator: Option<&AllocationCallbacks>,
        pipeline_layout: &mut PipelineLayout,
    ) -> Result {
        (self.create_pipeline_layout)(device, create_info, allocator, pipeline_layout)
    }

    #[inline]
    pub unsafe fn destroy_pipeline_layout(
        &self,
        device: Device,
        pipeline_layout: PipelineLayout,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_pipeline_layout)(device, pipeline_layout, allocator)
    }

    #[inline]
    pub unsafe fn create_graphics_pipelines(
        &self,
        device: Device,
        pipeline_cache: PipelineCache,
        create_infos: &[GraphicsPipelineCreateInfo],
        allocator: Option<&AllocationCallbacks>,
        pipelines: &mut [Pipeline],
    ) -> Result {
        (self.create_graphics_pipelines)(
            device,
            pipeline_cache,
            create_infos.len() as u32,
            create_infos.as_ptr(),
            allocator,
            pipelines.as_mut_ptr(),
        )
    }

    #[inline]
    pub unsafe fn create_compute_pipelines(
        &self,
        device: Device,
        pipeline_cache: PipelineCache,
        create_infos: &[ComputePipelineCreateInfo],
        allocator: Option<&AllocationCallbacks>,
        pipelines: &mut [Pipeline],
    ) -> Result {
        debug_assert_eq!(create_infos.len(), pipelines.len());
        (self.create_compute_pipelines)(
            device,
            pipeline_cache,
            create_infos.len() as u32,
            create_infos.as_ptr(),
            allocator,
            pipelines.as_mut_ptr(),
        )
    }

    #[inline]
    pub unsafe fn destroy_pipeline(
        &self,
        device: Device,
        pipeline: Pipeline,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_pipeline)(device, pipeline, allocator)
    }

    #[inline]
    pub unsafe fn cmd_bind_pipeline(
        &self,
        command_buffer: CommandBuffer,
        pipeline_bind_point: PipelineBindPoint,
        pipeline: Pipeline,
    ) {
        (self.cmd_bind_pipeline)(command_buffer, pipeline_bind_point, pipeline)
    }

    #[inline]
    pub unsafe fn cmd_set_viewport(
        &self,
        command_buffer: CommandBuffer,
        first_viewport: u32,
        viewports: &[Viewport],
    ) {
        (self.cmd_set_viewport)(
            command_buffer,
            first_viewport,
            viewports.len() as u32,
            viewports.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_set_scissor(
        &self,
        command_buffer: CommandBuffer,
        first_scissor: u32,
        scissors: &[Rect2d],
    ) {
        (self.cmd_set_scissor)(
            command_buffer,
            first_scissor,
            scissors.len() as u32,
            scissors.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_set_line_width(&self, command_buffer: CommandBuffer, line_width: f32) {
        (self.cmd_set_line_width)(command_buffer, line_width)
    }

    #[inline]
    pub unsafe fn cmd_set_depth_bias(
        &self,
        command_buffer: CommandBuffer,
        depth_bias_constant_factor: f32,
        depth_bias_clamp: f32,
        depth_bias_slope_factor: f32,
    ) {
        (self.cmd_set_depth_bias)(
            command_buffer,
            depth_bias_constant_factor,
            depth_bias_clamp,
            depth_bias_slope_factor,
        )
    }

    #[inline]
    pub unsafe fn cmd_set_blend_constants(
        &self,
        command_buffer: CommandBuffer,
        blend_constants: [f32; 4],
    ) {
        (self.cmd_set_blend_constants)(command_buffer, blend_constants)
    }

    #[inline]
    pub unsafe fn cmd_set_depth_bounds(
        &self,
        command_buffer: CommandBuffer,
        min_depth_bounds: f32,
        max_depth_bounds: f32,
    ) {
        (self.cmd_set_depth_bounds)(command_buffer, min_depth_bounds, max_depth_bounds)
    }

    #[inline]
    pub unsafe fn cmd_set_stencil_compare_mask(
        &self,
        command_buffer: CommandBuffer,
        face_mask: StencilFaceFlags,
        compare_mask: u32,
    ) {
        (self.cmd_set_stencil_compare_mask)(command_buffer, face_mask, compare_mask)
    }

    #[inline]
    pub unsafe fn cmd_set_stencil_write_mask(
        &self,
        command_buffer: CommandBuffer,
        face_mask: StencilFaceFlags,
        write_mask: u32,
    ) {
        (self.cmd_set_stencil_write_mask)(command_buffer, face_mask, write_mask)
    }

    #[inline]
    pub unsafe fn cmd_set_stencil_reference(
        &self,
        command_buffer: CommandBuffer,
        face_mask: StencilFaceFlags,
        reference: u32,
    ) {
        (self.cmd_set_stencil_reference)(command_buffer, face_mask, reference)
    }

    #[inline]
    pub unsafe fn cmd_bind_descriptor_sets(
        &self,
        command_buffer: CommandBuffer,
        pipeline_bind_point: PipelineBindPoint,
        layout: PipelineLayout,
        first_set: u32,
        descriptor_sets: &[DescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        (self.cmd_bind_descriptor_sets)(
            command_buffer,
            pipeline_bind_point,
            layout,
            first_set,
            descriptor_sets.len() as u32,
            descriptor_sets.as_ptr(),
            dynamic_offsets.len() as u32,
            dynamic_offsets.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_bind_index_buffer(
        &self,
        command_buffer: CommandBuffer,
        buffer: Buffer,
        offset: DeviceSize,
        index_type: IndexType,
    ) {
        (self.cmd_bind_index_buffer)(command_buffer, buffer, offset, index_type)
    }

    #[inline]
    pub unsafe fn cmd_bind_vertex_buffers(
        &self,
        command_buffer: CommandBuffer,
        first_binding: u32,
        binding_count: u32,
        buffers: *const Buffer,
        offsets: *const DeviceSize,
    ) {
        (self.cmd_bind_vertex_buffers)(
            command_buffer,
            first_binding,
            binding_count,
            buffers,
            offsets,
        )
    }

    #[inline]
    pub unsafe fn cmd_draw(
        &self,
        command_buffer: CommandBuffer,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        (self.cmd_draw)(
            command_buffer,
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        )
    }

    #[inline]
    pub unsafe fn cmd_draw_indexed(
        &self,
        command_buffer: CommandBuffer,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        (self.cmd_draw_indexed)(
            command_buffer,
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        )
    }

    #[inline]
    pub unsafe fn cmd_draw_indirect(
        &self,
        command_buffer: CommandBuffer,
        buffer: Buffer,
        offset: DeviceSize,
        draw_count: u32,
        stride: u32,
    ) {
        (self.cmd_draw_indirect)(command_buffer, buffer, offset, draw_count, stride)
    }

    #[inline]
    pub unsafe fn cmd_draw_indexed_indirect(
        &self,
        command_buffer: CommandBuffer,
        buffer: Buffer,
        offset: DeviceSize,
        draw_count: u32,
        stride: u32,
    ) {
        (self.cmd_draw_indexed_indirect)(command_buffer, buffer, offset, draw_count, stride)
    }

    #[inline]
    pub unsafe fn cmd_dispatch(
        &self,
        command_buffer: CommandBuffer,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        (self.cmd_dispatch)(command_buffer, group_count_x, group_count_y, group_count_z)
    }

    #[inline]
    pub unsafe fn cmd_dispatch_indirect(
        &self,
        command_buffer: CommandBuffer,
        buffer: Buffer,
        offset: DeviceSize,
    ) {
        (self.cmd_dispatch_indirect)(command_buffer, buffer, offset)
    }

    #[inline]
    pub unsafe fn cmd_copy_buffer(
        &self,
        command_buffer: CommandBuffer,
        src_buffer: Buffer,
        dst_buffer: Buffer,
        regions: &[BufferCopy],
    ) {
        (self.cmd_copy_buffer)(
            command_buffer,
            src_buffer,
            dst_buffer,
            regions.len() as u32,
            regions.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_copy_image(
        &self,
        command_buffer: CommandBuffer,
        src_image: Image,
        src_image_layout: ImageLayout,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        regions: &[ImageCopy],
    ) {
        (self.cmd_copy_image)(
            command_buffer,
            src_image,
            src_image_layout,
            dst_image,
            dst_image_layout,
            regions.len() as u32,
            regions.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_blit_image(
        &self,
        command_buffer: CommandBuffer,
        src_image: Image,
        src_image_layout: ImageLayout,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        regions: &[ImageBlit],
        filter: Filter,
    ) {
        (self.cmd_blit_image)(
            command_buffer,
            src_image,
            src_image_layout,
            dst_image,
            dst_image_layout,
            regions.len() as u32,
            regions.as_ptr(),
            filter,
        )
    }

    #[inline]
    pub unsafe fn cmd_copy_buffer_to_image(
        &self,
        command_buffer: CommandBuffer,
        src_buffer: Buffer,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        regions: &[BufferImageCopy],
    ) {
        (self.cmd_copy_buffer_to_image)(
            command_buffer,
            src_buffer,
            dst_image,
            dst_image_layout,
            regions.len() as u32,
            regions.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_copy_image_to_buffer(
        &self,
        command_buffer: CommandBuffer,
        src_image: Image,
        src_image_layout: ImageLayout,
        dst_buffer: Buffer,
        regions: &[BufferImageCopy],
    ) {
        (self.cmd_copy_image_to_buffer)(
            command_buffer,
            src_image,
            src_image_layout,
            dst_buffer,
            regions.len() as u32,
            regions.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_update_buffer(
        &self,
        command_buffer: CommandBuffer,
        dst_buffer: Buffer,
        dst_offset: DeviceSize,
        data_size: DeviceSize,
        data: *const c_void,
    ) {
        (self.cmd_update_buffer)(command_buffer, dst_buffer, dst_offset, data_size, data)
    }

    #[inline]
    pub unsafe fn cmd_fill_buffer(
        &self,
        command_buffer: CommandBuffer,
        dst_buffer: Buffer,
        dst_offset: DeviceSize,
        size: DeviceSize,
        data: u32,
    ) {
        (self.cmd_fill_buffer)(command_buffer, dst_buffer, dst_offset, size, data)
    }

    #[inline]
    pub unsafe fn cmd_clear_color_image(
        &self,
        command_buffer: CommandBuffer,
        image: Image,
        image_layout: ImageLayout,
        color: &ClearColorValue,
        ranges: &[ImageSubresourceRange],
    ) {
        (self.cmd_clear_color_image)(
            command_buffer,
            image,
            image_layout,
            color,
            ranges.len() as u32,
            ranges.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_clear_depth_stencil_image(
        &self,
        command_buffer: CommandBuffer,
        image: Image,
        image_layout: ImageLayout,
        depth_stencil: &ClearDepthStencilValue,
        ranges: &[ImageSubresourceRange],
    ) {
        (self.cmd_clear_depth_stencil_image)(
            command_buffer,
            image,
            image_layout,
            depth_stencil,
            ranges.len() as u32,
            ranges.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_clear_attachments(
        &self,
        command_buffer: CommandBuffer,
        attachments: &[ClearAttachment],
        rects: &[ClearRect],
    ) {
        (self.cmd_clear_attachments)(
            command_buffer,
            attachments.len() as u32,
            attachments.as_ptr(),
            rects.len() as u32,
            rects.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_resolve_image(
        &self,
        command_buffer: CommandBuffer,
        src_image: Image,
        src_image_layout: ImageLayout,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        regions: &[ImageResolve],
    ) {
        (self.cmd_resolve_image)(
            command_buffer,
            src_image,
            src_image_layout,
            dst_image,
            dst_image_layout,
            regions.len() as u32,
            regions.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_set_event(
        &self,
        command_buffer: CommandBuffer,
        event: Event,
        stage_mask: PipelineStageFlags,
    ) {
        (self.cmd_set_event)(command_buffer, event, stage_mask)
    }

    #[inline]
    pub unsafe fn cmd_reset_event(
        &self,
        command_buffer: CommandBuffer,
        event: Event,
        stage_mask: PipelineStageFlags,
    ) {
        (self.cmd_reset_event)(command_buffer, event, stage_mask)
    }

    #[inline]
    pub unsafe fn cmd_wait_events(
        &self,
        command_buffer: CommandBuffer,
        events: &[Event],
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier],
        image_memory_barriers: &[ImageMemoryBarrier],
    ) {
        (self.cmd_wait_events)(
            command_buffer,
            events.len() as u32,
            events.as_ptr(),
            src_stage_mask,
            dst_stage_mask,
            memory_barriers.len() as u32,
            memory_barriers.as_ptr(),
            buffer_memory_barriers.len() as u32,
            buffer_memory_barriers.as_ptr(),
            image_memory_barriers.len() as u32,
            image_memory_barriers.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_pipeline_barrier(
        &self,
        command_buffer: CommandBuffer,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier],
        image_memory_barriers: &[ImageMemoryBarrier],
    ) {
        (self.cmd_pipeline_barrier)(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            dependency_flags,
            memory_barriers.len() as u32,
            memory_barriers.as_ptr(),
            buffer_memory_barriers.len() as u32,
            buffer_memory_barriers.as_ptr(),
            image_memory_barriers.len() as u32,
            image_memory_barriers.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_begin_query(
        &self,
        command_buffer: CommandBuffer,
        query_pool: QueryPool,
        query: u32,
        flags: QueryControlFlags,
    ) {
        (self.cmd_begin_query)(command_buffer, query_pool, query, flags)
    }

    #[inline]
    pub unsafe fn cmd_end_query(
        &self,
        command_buffer: CommandBuffer,
        query_pool: QueryPool,
        query: u32,
    ) {
        (self.cmd_end_query)(command_buffer, query_pool, query)
    }

    #[inline]
    pub unsafe fn cmd_reset_query_pool(
        &self,
        command_buffer: CommandBuffer,
        query_pool: QueryPool,
        first_query: u32,
        query_count: u32,
    ) {
        (self.cmd_reset_query_pool)(command_buffer, query_pool, first_query, query_count)
    }

    #[inline]
    pub unsafe fn cmd_write_timestamp(
        &self,
        command_buffer: CommandBuffer,
        pipeline_stage: PipelineStageFlags,
        query_pool: QueryPool,
        query: u32,
    ) {
        (self.cmd_write_timestamp)(command_buffer, pipeline_stage, query_pool, query)
    }

    #[inline]
    pub unsafe fn cmd_copy_query_pool_results(
        &self,
        command_buffer: CommandBuffer,
        query_pool: QueryPool,
        first_query: u32,
        query_count: u32,
        dst_buffer: Buffer,
        dst_offset: DeviceSize,
        stride: DeviceSize,
        flags: QueryResultFlags,
    ) {
        (self.cmd_copy_query_pool_results)(
            command_buffer,
            query_pool,
            first_query,
            query_count,
            dst_buffer,
            dst_offset,
            stride,
            flags,
        )
    }

    #[inline]
    pub unsafe fn cmd_push_constants(
        &self,
        command_buffer: CommandBuffer,
        layout: PipelineLayout,
        stage_flags: ShaderStageFlags,
        offset: u32,
        size: u32,
        values: *const c_void,
    ) {
        (self.cmd_push_constants)(command_buffer, layout, stage_flags, offset, size, values)
    }

    #[inline]
    pub unsafe fn cmd_begin_render_pass(
        &self,
        command_buffer: CommandBuffer,
        render_pass_begin: &RenderPassBeginInfo,
        contents: SubpassContents,
    ) {
        (self.cmd_begin_render_pass)(command_buffer, render_pass_begin, contents)
    }

    #[inline]
    pub unsafe fn cmd_next_subpass(
        &self,
        command_buffer: CommandBuffer,
        contents: SubpassContents,
    ) {
        (self.cmd_next_subpass)(command_buffer, contents)
    }

    #[inline]
    pub unsafe fn cmd_end_render_pass(&self, command_buffer: CommandBuffer) {
        (self.cmd_end_render_pass)(command_buffer)
    }

    #[inline]
    pub unsafe fn cmd_pipeline_barrier2(
        &self,
        command_buffer: CommandBuffer,
        dependency_info: &DependencyInfo,
    ) {
        (self.cmd_pipeline_barrier2)(command_buffer, dependency_info)
    }

    #[inline]
    pub unsafe fn cmd_wait_events2(
        &self,
        command_buffer: CommandBuffer,
        event_count: u32,
        events: *const Event,
        dependency_infos: *const DependencyInfo,
    ) {
        (self.cmd_wait_events2)(command_buffer, event_count, events, dependency_infos)
    }

    #[inline]
    pub unsafe fn cmd_set_event2(
        &self,
        command_buffer: CommandBuffer,
        event: Event,
        dependency_info: &DependencyInfo,
    ) {
        (self.cmd_set_event2)(command_buffer, event, dependency_info)
    }

    #[inline]
    pub unsafe fn cmd_begin_rendering(
        &self,
        command_buffer: CommandBuffer,
        rendering_info: &RenderingInfo,
    ) {
        (self.cmd_begin_rendering)(command_buffer, rendering_info)
    }

    #[inline]
    pub unsafe fn cmd_end_rendering(&self, command_buffer: CommandBuffer) {
        (self.cmd_end_rendering)(command_buffer)
    }

    #[inline]
    pub unsafe fn cmd_execute_commands(
        &self,
        command_buffer: CommandBuffer,
        command_buffers: &[CommandBuffer],
    ) {
        (self.cmd_execute_commands)(
            command_buffer,
            command_buffers.len() as u32,
            command_buffers.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_set_viewport_with_count(
        &self,
        command_buffer: CommandBuffer,
        viewports: &[Viewport],
    ) {
        (self.cmd_set_viewport_with_count)(
            command_buffer,
            viewports.len() as u32,
            viewports.as_ptr(),
        )
    }

    #[inline]
    pub unsafe fn cmd_set_scissor_with_count(
        &self,
        command_buffer: CommandBuffer,
        scissors: &[Rect2d],
    ) {
        (self.cmd_set_scissor_with_count)(command_buffer, scissors.len() as u32, scissors.as_ptr())
    }

    #[inline]
    pub unsafe fn get_image_memory_requirements2(
        &self,
        device: Device,
        info: &ImageMemoryRequirementsInfo2,
        memory_requirements: &mut MemoryRequirements2,
    ) {
        (self.get_image_memory_requirements2)(device, info, memory_requirements)
    }

    #[inline]
    pub unsafe fn bind_image_memory2(&self, device: Device, bind_infos: &[BindImageMemoryInfo]) {
        (self.bind_image_memory2)(device, bind_infos.len() as u32, bind_infos.as_ptr())
    }

    #[inline]
    pub unsafe fn get_buffer_memory_requirements2(
        &self,
        device: Device,
        info: &BufferMemoryRequirementsInfo2,
        memory_requirements: &mut MemoryRequirements2,
    ) {
        (self.get_buffer_memory_requirements2)(device, info, memory_requirements)
    }

    #[inline]
    pub unsafe fn bind_buffer_memory2(&self, device: Device, bind_infos: &[BindBufferMemoryInfo]) {
        (self.bind_buffer_memory2)(device, bind_infos.len() as u32, bind_infos.as_ptr())
    }

    #[inline]
    pub unsafe fn get_buffer_device_address(
        &self,
        device: Device,
        info: &BufferDeviceAddressInfo,
    ) -> DeviceAddress {
        (self.get_buffer_device_address)(device, info)
    }

    #[inline]
    pub unsafe fn queue_wait_idle(&self, queue: Queue) -> Result {
        (self.queue_wait_idle)(queue)
    }

    #[inline]
    pub unsafe fn device_wait_idle(&self, device: Device) -> Result {
        (self.device_wait_idle)(device)
    }
}

pub struct SurfaceKHRFunctions {
    destroy_surface: FnDestroySurfaceKHR,
    get_physical_device_surface_support: FnGetPhysicalDeviceSurfaceSupportKHR,
    get_physical_device_surface_capabilities: FnGetPhysicalDeviceSurfaceCapabilitiesKHR,
    get_physical_device_surface_formats: FnGetPhysicalDeviceSurfaceFormatsKHR,
    get_physical_device_surface_present_modes: FnGetPhysicalDeviceSurfacePresentModesKHR,
}

impl SurfaceKHRFunctions {
    pub fn new(global_functions: &GlobalFunctions, instance: Instance) -> Self {
        unsafe {
            let load = |name: &CStr| {
                global_functions
                    .get_instance_proc_addr(instance, name)
                    .unwrap_or_else(
                        #[cold]
                        || panic!("failed to load device function {}", name.to_string_lossy()),
                    )
            };
            Self {
                destroy_surface: transmute::<_, _>(load(c"vkDestroySurfaceKHR")),
                get_physical_device_surface_support: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceSurfaceSupportKHR",
                )),
                get_physical_device_surface_capabilities: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceSurfaceCapabilitiesKHR",
                )),
                get_physical_device_surface_formats: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceSurfaceFormatsKHR",
                )),
                get_physical_device_surface_present_modes: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceSurfacePresentModesKHR",
                )),
            }
        }
    }

    pub unsafe fn destroy_surface(
        &self,
        instance: Instance,
        surface: SurfaceKHR,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_surface)(instance, surface, allocator)
    }

    pub unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: PhysicalDevice,
        queue_family_index: u32,
        surface: SurfaceKHR,
        supported: &mut Bool32,
    ) -> Result {
        (self.get_physical_device_surface_support)(
            physical_device,
            queue_family_index,
            surface,
            supported,
        )
    }

    pub unsafe fn get_physical_device_surface_capabilities(
        &self,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
        surface_capabilities: &mut SurfaceCapabilitiesKHR,
    ) -> Result {
        (self.get_physical_device_surface_capabilities)(
            physical_device,
            surface,
            surface_capabilities,
        )
    }

    pub unsafe fn get_physical_device_surface_formats(
        &self,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
        surface_format_count: &mut u32,
        surface_formats: *mut SurfaceFormatKHR,
    ) -> Result {
        (self.get_physical_device_surface_formats)(
            physical_device,
            surface,
            surface_format_count,
            surface_formats,
        )
    }

    pub unsafe fn get_physical_device_surface_present_modes(
        &self,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
        present_mode_count: &mut u32,
        present_modes: *mut PresentModeKHR,
    ) -> Result {
        (self.get_physical_device_surface_present_modes)(
            physical_device,
            surface,
            present_mode_count,
            present_modes,
        )
    }
}

pub struct XcbSurfaceKHRFunctions {
    create_xcb_surface: FnCreateXcbSurfaceKHR,
}

impl XcbSurfaceKHRFunctions {
    pub fn new(global_functions: &GlobalFunctions, instance: Instance) -> Self {
        unsafe {
            let load = |name: &CStr| {
                global_functions
                    .get_instance_proc_addr(instance, name)
                    .unwrap_or_else(
                        #[cold]
                        || panic!("failed to load device function {}", name.to_string_lossy()),
                    )
            };
            Self {
                create_xcb_surface: transmute::<_, _>(load(c"vkCreateXcbSurfaceKHR")),
            }
        }
    }

    pub fn create_xcb_surface(
        &self,
        instance: Instance,
        create_info: &XcbSurfaceCreateInfoKHR,
        allocator: Option<&AllocationCallbacks>,
        surface: &mut SurfaceKHR,
    ) -> Result {
        (self.create_xcb_surface)(instance, create_info, allocator, surface)
    }
}

pub struct XlibSurfaceKHRFunctions {
    create_xlib_surface: FnCreateXlibSurfaceKHR,
}

impl XlibSurfaceKHRFunctions {
    pub fn new(global_functions: &GlobalFunctions, instance: Instance) -> Self {
        unsafe {
            let load = |name: &CStr| {
                global_functions
                    .get_instance_proc_addr(instance, name)
                    .unwrap_or_else(
                        #[cold]
                        || panic!("failed to load device function {}", name.to_string_lossy()),
                    )
            };
            Self {
                create_xlib_surface: transmute::<_, _>(load(c"vkCreateXlibSurfaceKHR")),
            }
        }
    }

    pub fn create_xlib_surface(
        &self,
        instance: Instance,
        create_info: &XlibSurfaceCreateInfoKHR,
        allocator: Option<&AllocationCallbacks>,
        surface: &mut SurfaceKHR,
    ) -> Result {
        (self.create_xlib_surface)(instance, create_info, allocator, surface)
    }
}

pub struct WaylandSurfaceKHRFunctions {
    create_wayland_surface: FnCreateWaylandSurfaceKHR,
}

impl WaylandSurfaceKHRFunctions {
    pub fn new(global_functions: &GlobalFunctions, instance: Instance) -> Self {
        unsafe {
            let load = |name: &CStr| {
                global_functions
                    .get_instance_proc_addr(instance, name)
                    .unwrap_or_else(
                        #[cold]
                        || panic!("failed to load device function {}", name.to_string_lossy()),
                    )
            };
            Self {
                create_wayland_surface: transmute::<_, _>(load(c"vkCreateWaylandSurfaceKHR")),
            }
        }
    }

    pub fn create_wayland_surface(
        &self,
        instance: Instance,
        create_info: &WaylandSurfaceCreateInfoKHR,
        allocator: Option<&AllocationCallbacks>,
        surface: &mut SurfaceKHR,
    ) -> Result {
        (self.create_wayland_surface)(instance, create_info, allocator, surface)
    }
}

pub struct SwapchainKHRFunctions {
    create_swapchain: FnCreateSwapchainKHR,
    destroy_swapchain: FnDestroySwapchainKHR,
    get_swapchain_images: FnGetSwapchainImagesKHR,
    acquire_next_image: FnAcquireNextImageKHR,
    queue_present: FnQueuePresentKHR,

    acquire_next_image2: FnAcquireNextImage2KHR,
}

impl SwapchainKHRFunctions {
    pub fn new(global_functions: &GlobalFunctions, instance: Instance, api_version: u32) -> Self {
        unsafe {
            let load = |name: &CStr, function_version: u32| {
                if api_version >= function_version {
                    global_functions
                        .get_instance_proc_addr(instance, name)
                        .unwrap_or_else(
                            #[cold]
                            || panic!("failed to load device function {}", name.to_string_lossy()),
                        )
                } else {
                    transmute::<_, _>(vulkan_instance_version_not_supported as fn())
                }
            };
            Self {
                create_swapchain: transmute::<_, _>(load(c"vkCreateSwapchainKHR", VERSION_1_0)),
                destroy_swapchain: transmute::<_, _>(load(c"vkDestroySwapchainKHR", VERSION_1_0)),
                get_swapchain_images: transmute::<_, _>(load(
                    c"vkGetSwapchainImagesKHR",
                    VERSION_1_0,
                )),
                acquire_next_image: transmute::<_, _>(load(c"vkAcquireNextImageKHR", VERSION_1_0)),
                queue_present: transmute::<_, _>(load(c"vkQueuePresentKHR", VERSION_1_0)),

                acquire_next_image2: transmute::<_, _>(load(
                    c"vkAcquireNextImage2KHR",
                    VERSION_1_1,
                )),
            }
        }
    }

    pub unsafe fn create_swapchain(
        &self,
        device: Device,
        create_info: &SwapchainCreateInfoKHR,
        allocator: Option<&AllocationCallbacks>,
        swapchain: &mut SwapchainKHR,
    ) -> Result {
        (self.create_swapchain)(device, create_info, allocator, swapchain)
    }

    pub unsafe fn destroy_swapchain(
        &self,
        device: Device,
        swapchain: SwapchainKHR,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_swapchain)(device, swapchain, allocator)
    }

    pub unsafe fn get_swapchain_images(
        &self,
        device: Device,
        swapchain: SwapchainKHR,
        swapchain_image_count: &mut u32,
        swapchain_images: *mut Image,
    ) -> Result {
        (self.get_swapchain_images)(device, swapchain, swapchain_image_count, swapchain_images)
    }

    pub unsafe fn acquire_next_image(
        &self,
        device: Device,
        swapchain: SwapchainKHR,
        timeout: u64,
        semaphore: Semaphore,
        fence: Fence,
        image_index: &mut u32,
    ) -> Result {
        (self.acquire_next_image)(device, swapchain, timeout, semaphore, fence, image_index)
    }

    pub unsafe fn acquire_next_image2(
        &self,
        device: Device,
        acquire_info: &AcquireNextImageInfoKHR,
        image_index: &mut u32,
    ) -> Result {
        (self.acquire_next_image2)(device, acquire_info, image_index)
    }

    pub unsafe fn queue_present(&self, queue: Queue, present_info: &PresentInfoKHR) -> Result {
        (self.queue_present)(queue, present_info)
    }
}

pub struct SurfaceCapabilities2KhrFunctions {
    get_physical_device_surface_capabilities2_khr: FnGetPhysicalDeviceSurfaceCapabilities2KHR,
    get_physical_device_surface_formats2_khr: FnGetPhysicalDeviceSurfaceFormats2KHR,
}

impl SurfaceCapabilities2KhrFunctions {
    pub fn new(global_functions: &GlobalFunctions, instance: Instance) -> Self {
        unsafe {
            let load = |name: &CStr| {
                global_functions
                    .get_instance_proc_addr(instance, name)
                    .unwrap_or_else(
                        #[cold]
                        || panic!("failed to load device function {}", name.to_string_lossy()),
                    )
            };
            Self {
                get_physical_device_surface_formats2_khr: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceSurfaceFormats2KHR",
                )),
                get_physical_device_surface_capabilities2_khr: transmute::<_, _>(load(
                    c"vkGetPhysicalDeviceSurfaceCapabilities2KHR",
                )),
            }
        }
    }

    pub unsafe fn get_physical_device_surface_capabilities2_khr(
        &self,
        physical_device: PhysicalDevice,
        surface_info: &PhysicalDeviceSurfaceInfo2KHR,
        surface_capabilities: &mut SurfaceCapabilities2KHR,
    ) -> Result {
        (self.get_physical_device_surface_capabilities2_khr)(
            physical_device,
            surface_info,
            surface_capabilities,
        )
    }

    pub unsafe fn get_physical_device_surface_formats2_khr(
        &self,
        physical_device: PhysicalDevice,
        surface_info: PhysicalDeviceSurfaceInfo2KHR,
        surface_format_count: &mut u32,
        surface_formats: *mut SurfaceFormat2KHR,
    ) -> Result {
        (self.get_physical_device_surface_formats2_khr)(
            physical_device,
            surface_info,
            surface_format_count,
            surface_formats,
        )
    }
}

pub struct SwapchainMaintenance1ExtFunctions {
    release_swapchain_images_ext: FnReleaseSwapchainImagesEXT,
}

impl SwapchainMaintenance1ExtFunctions {
    pub fn new(instance_functions: &InstanceFunctions, device: Device) -> Self {
        unsafe {
            let load = |name: &CStr| {
                instance_functions
                    .get_device_proc_addr(device, name)
                    .unwrap_or_else(
                        #[cold]
                        || panic!("failed to load device function {}", name.to_string_lossy()),
                    )
            };
            Self {
                release_swapchain_images_ext: transmute::<_, _>(load(
                    c"vkReleaseSwapchainImagesEXT",
                )),
            }
        }
    }

    pub unsafe fn release_swapchain_images_ext(
        &self,
        device: Device,
        release_info: &ReleaseSwapchainImagesInfoEXT,
    ) -> Result {
        (self.release_swapchain_images_ext)(device, release_info)
    }
}

pub struct DebugUtilsFunctions {
    cmd_begin_debug_utils_label_ext: FnCmdBeginDebugUtilsLabelExt,
    cmd_end_debug_utils_label_ext: FnCmdEndDebugUtilsLabelExt,
    cmd_insert_debug_utils_label_ext: FnCmdInsertDebugUtilsLabelExt,
    create_debug_utils_messenger_ext: FnCreateDebugUtilsMessengerExt,
    destroy_debug_utils_messenger_ext: FnDestroyDebugUtilsMessengerExt,
    queue_begin_debug_utils_label_ext: FnQueueBeginDebugUtilsLabelExt,
    queue_end_debug_utils_label_ext: FnQueueEndDebugUtilsLabelExt,
    queue_insert_debug_utils_label_ext: FnQueueInsertDebugUtilsLabelExt,
    set_debug_utils_object_name_ext: FnSetDebugUtilsObjectNameExt,
    set_debug_utils_object_tag_ext: FnSetDebugUtilsObjectTagExt,
    submit_debug_utils_message_ext: FnSubmitDebugUtilsMessageExt,
}

impl DebugUtilsFunctions {
    pub fn new(
        global_functions: &GlobalFunctions,
        instance_functions: &InstanceFunctions,
        instance: Instance,
        device: Device,
    ) -> Self {
        unsafe {
            let load_instance = |name: &CStr| {
                global_functions
                    .get_instance_proc_addr(instance, name)
                    .unwrap_or_else(
                        #[cold]
                        || {
                            panic!(
                                "failed to load instance function {}",
                                name.to_string_lossy()
                            )
                        },
                    )
            };
            let load_device = |name: &CStr| {
                instance_functions
                    .get_device_proc_addr(device, name)
                    .unwrap_or_else(
                        #[cold]
                        || panic!("failed to load device function {}", name.to_string_lossy()),
                    )
            };
            Self {
                cmd_begin_debug_utils_label_ext: transmute::<_, _>(load_device(
                    c"vkCmdBeginDebugUtilsLabelEXT",
                )),
                cmd_end_debug_utils_label_ext: transmute::<_, _>(load_device(
                    c"vkCmdEndDebugUtilsLabelEXT",
                )),
                cmd_insert_debug_utils_label_ext: transmute::<_, _>(load_device(
                    c"vkCmdInsertDebugUtilsLabelEXT",
                )),
                create_debug_utils_messenger_ext: transmute::<_, _>(load_instance(
                    c"vkCreateDebugUtilsMessengerEXT",
                )),
                destroy_debug_utils_messenger_ext: transmute::<_, _>(load_instance(
                    c"vkDestroyDebugUtilsMessengerEXT",
                )),
                queue_begin_debug_utils_label_ext: transmute::<_, _>(load_device(
                    c"vkQueueBeginDebugUtilsLabelEXT",
                )),
                queue_end_debug_utils_label_ext: transmute::<_, _>(load_device(
                    c"vkQueueEndDebugUtilsLabelEXT",
                )),
                queue_insert_debug_utils_label_ext: transmute::<_, _>(load_device(
                    c"vkQueueInsertDebugUtilsLabelEXT",
                )),
                set_debug_utils_object_name_ext: transmute::<_, _>(load_device(
                    c"vkSetDebugUtilsObjectNameEXT",
                )),
                set_debug_utils_object_tag_ext: transmute::<_, _>(load_device(
                    c"vkSetDebugUtilsObjectTagEXT",
                )),
                submit_debug_utils_message_ext: transmute::<_, _>(load_instance(
                    c"vkSubmitDebugUtilsMessageEXT",
                )),
            }
        }
    }

    pub unsafe fn cmd_begin_debug_utils_label_ext(
        &self,
        command_buffer: CommandBuffer,
        label_info: &DebugUtilsLabelExt,
    ) {
        (self.cmd_begin_debug_utils_label_ext)(command_buffer, label_info)
    }

    pub unsafe fn cmd_end_debug_utils_label_ext(&self, command_buffer: CommandBuffer) {
        (self.cmd_end_debug_utils_label_ext)(command_buffer)
    }

    pub unsafe fn cmd_insert_debug_utils_label_ext(
        &self,
        command_buffer: CommandBuffer,
        label_info: &DebugUtilsLabelExt,
    ) {
        (self.cmd_insert_debug_utils_label_ext)(command_buffer, label_info)
    }

    pub unsafe fn create_debug_utils_messenger_ext(
        &self,
        instance: Instance,
        create_info: &DebugUtilsMessengerCreateInfoExt,
        allocator: Option<&AllocationCallbacks>,
        messenger: *mut DebugUtilsMessengerExt,
    ) {
        (self.create_debug_utils_messenger_ext)(instance, create_info, allocator, messenger)
    }

    pub unsafe fn destroy_debug_utils_messenger_ext(
        &self,
        instance: Instance,
        messenger: DebugUtilsMessengerExt,
        allocator: Option<&AllocationCallbacks>,
    ) {
        (self.destroy_debug_utils_messenger_ext)(instance, messenger, allocator)
    }

    pub unsafe fn queue_begin_debug_utils_label_ext(
        &self,
        queue: Queue,
        label_info: &DebugUtilsLabelExt,
    ) {
        (self.queue_begin_debug_utils_label_ext)(queue, label_info)
    }

    pub unsafe fn queue_end_debug_utils_label_ext(&self, queue: Queue) {
        (self.queue_end_debug_utils_label_ext)(queue)
    }

    pub unsafe fn queue_insert_debug_utils_label_ext(
        &self,
        queue: Queue,
        label_info: &DebugUtilsLabelExt,
    ) {
        (self.queue_insert_debug_utils_label_ext)(queue, label_info)
    }

    pub unsafe fn set_debug_utils_object_name_ext(
        &self,
        device: Device,
        name_info: &DebugUtilsObjectNameInfoExt,
    ) {
        (self.set_debug_utils_object_name_ext)(device, name_info)
    }

    pub unsafe fn set_debug_utils_object_tag_ext(
        &self,
        device: Device,
        tag_info: &DebugUtilsObjectTagInfoExt,
    ) {
        (self.set_debug_utils_object_tag_ext)(device, tag_info)
    }

    pub unsafe fn submit_debug_utils_message_ext(
        &self,
        instance: Instance,
        message_severity: DebugUtilsMessageSeverityFlagsExt,
        message_types: DebugUtilsMessageTypeFlagsExt,
        callback_data: &DebugUtilsMessengerCallbackDataExt,
    ) {
        (self.submit_debug_utils_message_ext)(
            instance,
            message_severity,
            message_types,
            callback_data,
        )
    }
}
