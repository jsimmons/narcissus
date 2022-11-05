use std::{
    cell::UnsafeCell,
    collections::{hash_map, HashMap, VecDeque},
    marker::PhantomData,
    os::raw::{c_char, c_void},
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use narcissus_app::{App, Window};
use narcissus_core::{
    cstr, default, manual_arc, manual_arc::ManualArc, HybridArena, Mutex, PhantomUnsend, Pool,
};

use vulkan_sys as vk;

use crate::{
    Bind, BindGroupLayout, BindGroupLayoutDesc, BindingType, Buffer, BufferDesc, BufferUsageFlags,
    ClearValue, CommandBufferToken, ComputePipelineDesc, Device, FrameToken, GpuConcurrent,
    GraphicsPipelineDesc, LoadOp, MemoryLocation, Pipeline, Sampler, SamplerAddressMode,
    SamplerCompareOp, SamplerDesc, SamplerFilter, ShaderStageFlags, Texture, TextureDesc,
    TextureDimension, TextureFormat, TextureUsageFlags, TextureViewDesc, ThreadToken, TypedBind,
};

const NUM_FRAMES: usize = 2;

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

#[must_use]
fn vulkan_format(format: TextureFormat) -> vk::Format {
    match format {
        TextureFormat::RGBA8_SRGB => vk::Format::R8G8B8A8_SRGB,
        TextureFormat::RGBA8_UNORM => vk::Format::R8G8B8A8_UNORM,
        TextureFormat::BGRA8_SRGB => vk::Format::B8G8R8A8_SRGB,
        TextureFormat::BGRA8_UNORM => vk::Format::B8G8R8A8_UNORM,
    }
}

#[must_use]
fn vulkan_aspect(format: TextureFormat) -> vk::ImageAspectFlags {
    match format {
        TextureFormat::BGRA8_SRGB
        | TextureFormat::BGRA8_UNORM
        | TextureFormat::RGBA8_SRGB
        | TextureFormat::RGBA8_UNORM => vk::ImageAspectFlags::COLOR,
    }
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
        BindingType::Texture => vk::DescriptorType::SampledImage,
        BindingType::UniformBuffer => vk::DescriptorType::UniformBuffer,
        BindingType::StorageBuffer => vk::DescriptorType::StorageBuffer,
        BindingType::DynamicUniformBuffer => vk::DescriptorType::UniformBufferDynamic,
        BindingType::DynamicStorageBuffer => vk::DescriptorType::StorageBufferDynamic,
    }
}

struct DelayQueue<T> {
    delay: u64,
    counter: u64,
    values: VecDeque<(u64, T)>,
}

impl<T> DelayQueue<T> {
    fn new(delay: u64) -> Self {
        Self {
            delay,
            counter: 0,
            values: VecDeque::new(),
        }
    }

    fn push(&mut self, value: T) {
        self.values.push_back((self.counter + self.delay, value))
    }

    fn expire<F: FnMut(T)>(&mut self, mut f: F) {
        self.counter += 1;

        let to_remove = self
            .values
            .iter()
            .take_while(|(expiry, _)| *expiry == self.counter)
            .count();

        for _ in 0..to_remove {
            f(self.values.pop_front().unwrap().1);
        }
    }

    pub fn drain<R>(&mut self, range: R) -> std::collections::vec_deque::Drain<'_, (u64, T)>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.values.drain(range)
    }
}

struct VulkanBuffer {
    memory: VulkanMemory,
    buffer: vk::Buffer,
}

#[derive(Clone)]
struct VulkanTexture {
    memory: VulkanMemory,
    image: vk::Image,
}

struct VulkanTextureUnique {
    texture: VulkanTexture,
    view: vk::ImageView,
}

struct VulkanTextureShared {
    texture: ManualArc<VulkanTexture>,
    view: vk::ImageView,
}

struct VulkanTextureSwapchain {
    window: Window,
    image: vk::Image,
    view: vk::ImageView,
}

enum VulkanTextureHolder {
    Unique(VulkanTextureUnique),
    Shared(VulkanTextureShared),
    Swapchain(VulkanTextureSwapchain),
}

impl VulkanTextureHolder {
    fn image_view(&self) -> vk::ImageView {
        match self {
            VulkanTextureHolder::Unique(x) => x.view,
            VulkanTextureHolder::Shared(x) => x.view,
            VulkanTextureHolder::Swapchain(x) => x.view,
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
        image_views: Box<[Texture]>,
    },
}

struct VulkanSwapchain {
    window: Window,
    surface: vk::SurfaceKHR,
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

struct VulkanCommandBuffer {
    command_buffer: vk::CommandBuffer,
    swapchains_touched: HashMap<Window, (vk::Image, vk::PipelineStageFlags2)>,
}

struct VulkanCommandBufferPool {
    command_pool: vk::CommandPool,
    next_free_index: usize,
    command_buffers: Vec<VulkanCommandBuffer>,
}

impl<'device> FrameToken<'device> {
    fn check_device(&self, device: &VulkanDevice) {
        let device_address = device as *const _ as usize;
        assert_eq!(
            self.device_address, device_address,
            "frame token device mismatch"
        )
    }

    fn check_frame_counter(&self, frame_counter_value: usize) {
        assert!(frame_counter_value & 1 == 0, "frame counter isn't acquired");
        assert_eq!(
            self.frame_index,
            frame_counter_value >> 1,
            "token does not match current frame"
        );
    }
}

struct FrameCounter {
    value: AtomicUsize,
}

impl FrameCounter {
    fn new() -> Self {
        Self {
            // Start the frame id at 1 so that the first `begin_frame` ticks us over to a new frame index.
            value: AtomicUsize::new(1),
        }
    }

    fn load(&self) -> usize {
        self.value.load(Ordering::Relaxed)
    }

    fn acquire(&self, device: &VulkanDevice) -> FrameToken {
        let old_frame_counter = self.value.fetch_add(1, Ordering::SeqCst);
        assert!(
            old_frame_counter & 1 == 1,
            "acquiring a frame token before previous frame token has been released"
        );

        let frame_counter = old_frame_counter + 1;
        let frame_index = frame_counter >> 1;

        FrameToken {
            device_address: device as *const _ as usize,
            frame_index,
            phantom: PhantomData,
        }
    }

    fn release(&self, frame_token: FrameToken) {
        let old_frame_counter = self.value.fetch_add(1, Ordering::SeqCst);
        frame_token.check_frame_counter(old_frame_counter);
    }
}

struct VulkanFrame {
    universal_queue_fence: AtomicU64,

    command_buffer_pools: GpuConcurrent<VulkanCommandBufferPool>,
    descriptor_pool_pools: GpuConcurrent<vk::DescriptorPool>,

    present_swapchains: Mutex<HashMap<Window, VulkanPresentInfo>>,

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
    fn command_buffer_mut<'a>(
        &self,
        thread_token: &'a mut ThreadToken,
        command_buffer_token: &'a CommandBufferToken,
    ) -> &'a mut VulkanCommandBuffer {
        let command_buffer_pool = self.command_buffer_pools.get_mut(thread_token);
        &mut command_buffer_pool.command_buffers[command_buffer_token.index]
    }

    fn recycle_semaphore(&self, semaphore: vk::Semaphore) {
        self.recycled_semaphores.lock().push_back(semaphore);
    }

    fn recycle_descriptor_pool(&self, descriptor_pool: vk::DescriptorPool) {
        self.recycled_descriptor_pools
            .lock()
            .push_back(descriptor_pool)
    }
}

type SwapchainDestroyQueue = DelayQueue<(
    Window,
    vk::SwapchainKHR,
    vk::SurfaceKHR,
    Box<[vk::ImageView]>,
)>;

pub(crate) struct VulkanDevice<'app> {
    app: &'app dyn App,

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

    swapchains: Mutex<HashMap<Window, VulkanSwapchain>>,
    destroyed_swapchains: Mutex<SwapchainDestroyQueue>,

    texture_pool: Mutex<Pool<VulkanTextureHolder>>,
    buffer_pool: Mutex<Pool<VulkanBuffer>>,
    sampler_pool: Mutex<Pool<VulkanSampler>>,
    bind_group_layout_pool: Mutex<Pool<VulkanBindGroupLayout>>,
    pipeline_pool: Mutex<Pool<VulkanPipeline>>,

    recycled_semaphores: Mutex<VecDeque<vk::Semaphore>>,
    recycled_descriptor_pools: Mutex<VecDeque<vk::DescriptorPool>>,

    _global_fn: vk::GlobalFunctions,
    instance_fn: vk::InstanceFunctions,
    surface_fn: vk::SurfaceKHRFunctions,
    swapchain_fn: vk::SwapchainKHRFunctions,
    device_fn: vk::DeviceFunctions,
}

impl<'app> VulkanDevice<'app> {
    pub(crate) fn new(app: &'app dyn App) -> Self {
        let get_proc_addr = app.vk_get_loader();
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

        let enabled_extensions = app.vk_instance_extensions();
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
            let command_buffer_pools = GpuConcurrent::new(|| {
                let pool = {
                    let create_info = vk::CommandPoolCreateInfo {
                        flags: vk::CommandPoolCreateFlags::TRANSIENT,
                        queue_family_index,
                        ..default()
                    };
                    let mut pool = vk::CommandPool::null();
                    vk_check!(device_fn.create_command_pool(device, &create_info, None, &mut pool));
                    pool
                };
                VulkanCommandBufferPool {
                    command_pool: pool,
                    command_buffers: Vec::new(),
                    next_free_index: 0,
                }
            });

            let descriptor_pool_pools = GpuConcurrent::new(|| vk::DescriptorPool::null());

            UnsafeCell::new(VulkanFrame {
                command_buffer_pools,
                descriptor_pool_pools,
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
            app,

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

            swapchains: Mutex::new(HashMap::new()),
            destroyed_swapchains: Mutex::new(DelayQueue::new(8)),

            texture_pool: default(),
            buffer_pool: default(),
            sampler_pool: default(),
            bind_group_layout_pool: default(),
            pipeline_pool: default(),

            recycled_semaphores: default(),
            recycled_descriptor_pools: default(),

            _global_fn: global_fn,
            instance_fn,
            surface_fn,
            swapchain_fn,
            device_fn,
        }
    }

    fn frame<'token>(&self, frame_token: &'token FrameToken) -> &'token VulkanFrame {
        frame_token.check_device(self);
        frame_token.check_frame_counter(self.frame_counter.load());
        // SAFETY: reference is bound to the frame token exposed by the API. only one frame token can be valid at a time.
        // The returned frame is only valid so long as we have a ref on the token.
        unsafe { &*self.frames[frame_token.frame_index % NUM_FRAMES].get() }
    }

    fn frame_mut<'token>(&self, frame_token: &'token mut FrameToken) -> &'token mut VulkanFrame {
        frame_token.check_device(self);
        frame_token.check_frame_counter(self.frame_counter.load());
        // SAFETY: mutable reference is bound to the frame token exposed by the API. only one frame token can be valid at a time.
        // The returned frame is only valid so long as we have a mut ref on the token.
        unsafe { &mut *self.frames[frame_token.frame_index % NUM_FRAMES].get() }
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
            MemoryLocation::Auto => vk::MemoryPropertyFlags::default(),
            MemoryLocation::PreferHost => vk::MemoryPropertyFlags::HOST_VISIBLE,
            MemoryLocation::PreferDevice => vk::MemoryPropertyFlags::DEVICE_LOCAL,
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
            let descriptor_count = 500;
            let pool_sizes = &[
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::Sampler,
                    descriptor_count,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::UniformBuffer,
                    descriptor_count,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::UniformBufferDynamic,
                    descriptor_count,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::StorageBuffer,
                    descriptor_count,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::StorageBufferDynamic,
                    descriptor_count,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::SampledImage,
                    descriptor_count: 500,
                },
            ];

            let mut descriptor_pool = vk::DescriptorPool::null();
            let create_info = vk::DescriptorPoolCreateInfo {
                max_sets: 500,
                pool_sizes: pool_sizes.into(),
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

    fn destroy_swapchain(
        &self,
        window: Window,
        surface: vk::SurfaceKHR,
        swapchain: vk::SwapchainKHR,
        image_views: &[vk::ImageView],
    ) {
        let app = self.app;
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
        if !window.is_null() {
            app.destroy_window(window);
        }
    }
}

impl<'driver> Device for VulkanDevice<'driver> {
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

        let memory = self.allocate_memory_for_buffer(buffer, desc.memory_location);

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

    fn create_texture(&self, desc: &TextureDesc) -> Texture {
        debug_assert_ne!(desc.layers, 0, "layers must be at least one");
        debug_assert_ne!(desc.width, 0, "width must be at least one");
        debug_assert_ne!(desc.height, 0, "height must be at least one");
        debug_assert_ne!(desc.depth, 0, "depth must be at least one");

        if desc.dimension == TextureDimension::Type3d {
            debug_assert_eq!(desc.layers, 1, "3d image arrays are illegal");
        }

        if desc.dimension == TextureDimension::TypeCube {
            debug_assert!(desc.layers % 6 == 0, "cubemaps must have 6 layers each");
            debug_assert_eq!(desc.depth, 1, "cubemap faces must be 2d");
        }

        let mut flags = vk::ImageCreateFlags::default();
        if desc.dimension == TextureDimension::TypeCube {
            flags |= vk::ImageCreateFlags::CUBE_COMPATIBLE
        }

        let image_type = match desc.dimension {
            TextureDimension::Type1d => vk::ImageType::Type1d,
            TextureDimension::Type2d => vk::ImageType::Type2d,
            TextureDimension::Type3d => vk::ImageType::Type3d,
            TextureDimension::TypeCube => vk::ImageType::Type2d,
        };
        let format = vulkan_format(desc.format);
        let extent = vk::Extent3d {
            width: desc.width,
            height: desc.height,
            depth: desc.depth,
        };

        let mut usage = default();
        if desc.usage.contains(TextureUsageFlags::SAMPLED) {
            usage |= vk::ImageUsageFlags::SAMPLED;
        }
        if desc.usage.contains(TextureUsageFlags::STORAGE) {
            usage |= vk::ImageUsageFlags::STORAGE;
        }
        if desc.usage.contains(TextureUsageFlags::DEPTH_STENCIL) {
            usage |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        }
        if desc.usage.contains(TextureUsageFlags::RENDER_TARGET) {
            usage |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
        }
        if desc.usage.contains(TextureUsageFlags::TRANSFER_DST) {
            usage |= vk::ImageUsageFlags::TRANSFER_DST;
        }
        if desc.usage.contains(TextureUsageFlags::TRANSFER_SRC) {
            usage |= vk::ImageUsageFlags::TRANSFER_SRC;
        }

        let queue_family_indices = &[self.universal_queue_family_index];
        let create_info = vk::ImageCreateInfo {
            flags,
            image_type,
            format,
            extent,
            mip_levels: desc.mip_levels,
            array_layers: desc.layers,
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

        let memory = self.allocate_memory_for_image(image, desc.memory_location);

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

        let view_type = match (desc.layers, desc.dimension) {
            (1, TextureDimension::Type1d) => vk::ImageViewType::Type1d,
            (1, TextureDimension::Type2d) => vk::ImageViewType::Type2d,
            (1, TextureDimension::Type3d) => vk::ImageViewType::Type3d,
            (6, TextureDimension::TypeCube) => vk::ImageViewType::TypeCube,
            (_, TextureDimension::Type1d) => vk::ImageViewType::Type1dArray,
            (_, TextureDimension::Type2d) => vk::ImageViewType::Type2dArray,
            (_, TextureDimension::TypeCube) => vk::ImageViewType::TypeCubeArray,
            _ => panic!("unsupported view type"),
        };

        let aspect_mask = vulkan_aspect(desc.format);
        let create_info = vk::ImageViewCreateInfo {
            image,
            view_type,
            format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: desc.mip_levels,
                base_array_layer: 0,
                layer_count: desc.layers,
            },
            ..default()
        };

        let mut view = vk::ImageView::null();
        vk_check!(self
            .device_fn
            .create_image_view(self.device, &create_info, None, &mut view));

        let texture = VulkanTextureUnique {
            texture: VulkanTexture { image, memory },
            view,
        };

        let handle = self
            .texture_pool
            .lock()
            .insert(VulkanTextureHolder::Unique(texture));

        Texture(handle)
    }

    fn create_texture_view(&self, desc: &TextureViewDesc) -> Texture {
        let mut texture_pool = self.texture_pool.lock();
        let texture = texture_pool.get_mut(desc.texture.0).unwrap();

        let arc_texture;
        match texture {
            VulkanTextureHolder::Shared(shared) => arc_texture = shared.texture.clone(),
            VulkanTextureHolder::Unique(unique) => {
                let unique_texture = ManualArc::new(unique.texture.clone());
                arc_texture = unique_texture.clone();
                let unique_view = unique.view;
                *texture = VulkanTextureHolder::Shared(VulkanTextureShared {
                    texture: unique_texture,
                    view: unique_view,
                })
            }
            VulkanTextureHolder::Swapchain(_) => {
                panic!("unable to create additional views of swapchain images")
            }
        }

        let view_type = match (desc.layer_count, desc.dimension) {
            (1, TextureDimension::Type1d) => vk::ImageViewType::Type1d,
            (1, TextureDimension::Type2d) => vk::ImageViewType::Type2d,
            (1, TextureDimension::Type3d) => vk::ImageViewType::Type3d,
            (6, TextureDimension::TypeCube) => vk::ImageViewType::TypeCube,
            (_, TextureDimension::Type1d) => vk::ImageViewType::Type1dArray,
            (_, TextureDimension::Type2d) => vk::ImageViewType::Type2dArray,
            (_, TextureDimension::TypeCube) => vk::ImageViewType::TypeCubeArray,
            _ => panic!("unsupported view type"),
        };

        let format = vulkan_format(desc.format);
        let aspect_mask = vulkan_aspect(desc.format);

        let create_info = vk::ImageViewCreateInfo {
            image: arc_texture.image,
            view_type,
            format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: desc.base_mip,
                level_count: desc.mip_count,
                base_array_layer: desc.base_layer,
                layer_count: desc.layer_count,
            },
            ..default()
        };

        let mut view = vk::ImageView::null();
        vk_check!(self
            .device_fn
            .create_image_view(self.device, &create_info, None, &mut view));

        let handle = texture_pool.insert(VulkanTextureHolder::Shared(VulkanTextureShared {
            texture: arc_texture,
            view,
        }));

        Texture(handle)
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
            SamplerCompareOp::None => (vk::Bool32::False, vk::CompareOp::Always),
            SamplerCompareOp::Less => (vk::Bool32::True, vk::CompareOp::Less),
            SamplerCompareOp::LessEq => (vk::Bool32::True, vk::CompareOp::LessOrEqual),
            SamplerCompareOp::Greater => (vk::Bool32::True, vk::CompareOp::Greater),
            SamplerCompareOp::GreaterEq => (vk::Bool32::True, vk::CompareOp::GreaterOrEqual),
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
        let layout_bindings = desc
            .entries
            .iter()
            .map(|x| vk::DescriptorSetLayoutBinding {
                binding: x.slot,
                descriptor_type: vulkan_descriptor_type(x.binding_type),
                descriptor_count: x.count,
                stage_flags: vulkan_shader_stage_flags(x.stages),
                immutable_samplers: std::ptr::null(),
            })
            .collect::<Vec<_>>();

        let create_info = &vk::DescriptorSetLayoutCreateInfo {
            bindings: layout_bindings.as_slice().into(),
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
                name: desc.vertex_shader.entrypoint_name.as_ptr(),
                module: vertex_module,
                ..default()
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::FRAGMENT,
                name: desc.fragment_shader.entrypoint_name.as_ptr(),
                module: fragment_module,
                ..default()
            },
        ];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TriangleList,
            ..default()
        };
        let viewport_state = vk::PipelineViewportStateCreateInfo::default();
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo {
            line_width: 1.0,
            ..default()
        };
        let multisample_state = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::SAMPLE_COUNT_1,
            ..default()
        };
        let color_blend_attachments = &[vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
            ..default()
        }];
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
        let color_attachment_formats = desc
            .layout
            .color_attachment_formats
            .iter()
            .copied()
            .map(vulkan_format)
            .collect::<Vec<_>>();

        let pipeline_rendering_create_info = vk::PipelineRenderingCreateInfo {
            view_mask: 0,
            color_attachment_formats: color_attachment_formats.as_slice().into(),
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

    fn destroy_buffer(&self, frame_token: &FrameToken, buffer: Buffer) {
        if let Some(buffer) = self.buffer_pool.lock().remove(buffer.0) {
            let frame = self.frame(frame_token);
            frame.destroyed_buffers.lock().push_back(buffer.buffer);
            frame.destroyed_allocations.lock().push_back(buffer.memory);
        }
    }

    fn destroy_texture(&self, frame_token: &FrameToken, texture: Texture) {
        if let Some(texture) = self.texture_pool.lock().remove(texture.0) {
            let frame = self.frame(frame_token);

            match texture {
                // The texture is unique, we've never allocated a reference counted object for it.
                VulkanTextureHolder::Unique(texture) => {
                    frame.destroyed_image_views.lock().push_back(texture.view);
                    frame
                        .destroyed_images
                        .lock()
                        .push_back(texture.texture.image);
                    frame
                        .destroyed_allocations
                        .lock()
                        .push_back(texture.texture.memory);
                }
                // The texture was at one point shared, we may or may not have the last reference.
                VulkanTextureHolder::Shared(texture) => {
                    frame.destroyed_image_views.lock().push_back(texture.view);
                    // If we had the last reference we need to destroy the image and memory too
                    if let manual_arc::Release::Unique(texture) = texture.texture.release() {
                        frame.destroyed_images.lock().push_back(texture.image);
                        frame.destroyed_allocations.lock().push_back(texture.memory);
                    }
                }
                VulkanTextureHolder::Swapchain(_) => {
                    panic!("cannot directly destroy swapchain images")
                }
            }
        }
    }

    fn destroy_sampler(&self, frame_token: &FrameToken, sampler: Sampler) {
        if let Some(sampler) = self.sampler_pool.lock().remove(sampler.0) {
            self.frame(frame_token)
                .destroyed_samplers
                .lock()
                .push_back(sampler.0)
        }
    }

    fn destroy_bind_group_layout(
        &self,
        frame_token: &FrameToken,
        bind_group_layout: BindGroupLayout,
    ) {
        if let Some(bind_group_layout) = self
            .bind_group_layout_pool
            .lock()
            .remove(bind_group_layout.0)
        {
            self.frame(frame_token)
                .destroyed_descriptor_set_layouts
                .lock()
                .push_back(bind_group_layout.0)
        }
    }

    fn destroy_pipeline(&self, frame_token: &FrameToken, pipeline: Pipeline) {
        if let Some(pipeline) = self.pipeline_pool.lock().remove(pipeline.0) {
            let frame = self.frame(frame_token);
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

    fn destroy_window(&self, window: Window) {
        if let Some(VulkanSwapchain {
            window: _,
            surface,
            surface_format: _,
            state,
            _formats: _,
            _present_modes: _,
            capabilities: _,
        }) = self.swapchains.lock().remove(&window)
        {
            let mut texture_pool = self.texture_pool.lock();

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
                    match texture_pool.remove(image_view.0) {
                        Some(VulkanTextureHolder::Swapchain(VulkanTextureSwapchain {
                            window: _,
                            image: _,
                            view,
                        })) => vulkan_image_views.push(view),
                        _ => panic!("swapchain texture in wrong state"),
                    }
                }

                self.destroyed_swapchains.lock().push((
                    window,
                    swapchain,
                    surface,
                    vulkan_image_views.into_boxed_slice(),
                ));
            }
        }
    }

    fn acquire_swapchain(
        &self,
        frame_token: &FrameToken,
        window: Window,
        format: TextureFormat,
    ) -> (u32, u32, Texture) {
        let format = vulkan_format(format);

        let mut swapchains = self.swapchains.lock();
        let mut vulkan_swapchain = swapchains.entry(window).or_insert_with(|| {
            let surface = self.app.vk_create_surface(window, self.instance.as_raw());
            let surface = vk::SurfaceKHR::from_raw(surface);

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
                window,
                surface,
                surface_format,
                state: VulkanSwapchainState::Vacant,
                _formats: formats,
                _present_modes: present_modes,
                capabilities,
            }
        });

        assert_eq!(format, vulkan_swapchain.surface_format.format);

        let frame = self.frame(frame_token);
        let mut texture_pool = self.texture_pool.lock();

        let mut present_swapchains = frame.present_swapchains.lock();
        let present_info = match present_swapchains.entry(window) {
            hash_map::Entry::Occupied(_) => {
                panic!("attempting to acquire the same swapchain multiple times in a frame")
            }
            hash_map::Entry::Vacant(entry) => entry.insert(default()),
        };

        let mut old_swapchain = vk::SwapchainKHR::null();
        let mut iters = 0;

        loop {
            iters += 1;
            if iters > 10 {
                panic!("acquiring swapchain image took more than 10 tries");
            }

            let (desired_width, desired_height) =
                self.app.vk_get_surface_extent(vulkan_swapchain.window);

            vk_check!(self.surface_fn.get_physical_device_surface_capabilities(
                self.physical_device,
                vulkan_swapchain.surface,
                &mut vulkan_swapchain.capabilities
            ));

            let desired_width = desired_width.clamp(
                vulkan_swapchain.capabilities.min_image_extent.width,
                vulkan_swapchain.capabilities.max_image_extent.width,
            );
            let desired_height = desired_height.clamp(
                vulkan_swapchain.capabilities.min_image_extent.height,
                vulkan_swapchain.capabilities.max_image_extent.height,
            );

            match &mut vulkan_swapchain.state {
                VulkanSwapchainState::Vacant => {
                    let image_extent = vk::Extent2d {
                        width: desired_width,
                        height: desired_height,
                    };
                    let mut new_swapchain = vk::SwapchainKHR::null();
                    let create_info = vk::SwapchainCreateInfoKHR {
                        surface: vulkan_swapchain.surface,
                        min_image_count: vulkan_swapchain.capabilities.min_image_count,
                        image_format: vulkan_swapchain.surface_format.format,
                        image_color_space: vulkan_swapchain.surface_format.color_space,
                        image_extent,
                        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
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

                            let handle = texture_pool.insert(VulkanTextureHolder::Swapchain(
                                VulkanTextureSwapchain {
                                    window,
                                    image,
                                    view,
                                },
                            ));
                            Texture(handle)
                        })
                        .collect::<Box<_>>();

                    vulkan_swapchain.state = VulkanSwapchainState::Occupied {
                        width: image_extent.width,
                        height: image_extent.height,
                        suboptimal: false,
                        swapchain: new_swapchain,
                        image_views,
                    };

                    continue;
                }
                VulkanSwapchainState::Occupied {
                    width,
                    height,
                    suboptimal,
                    swapchain,
                    image_views,
                } => {
                    let destroy_image_views =
                        |textures: &mut Pool<VulkanTextureHolder>| -> Box<[vk::ImageView]> {
                            let mut vulkan_image_views = Vec::new();
                            for &image_view in image_views.iter() {
                                match textures.remove(image_view.0) {
                                    Some(VulkanTextureHolder::Swapchain(
                                        VulkanTextureSwapchain {
                                            window: _,
                                            image: _,
                                            view,
                                        },
                                    )) => vulkan_image_views.push(view),
                                    _ => panic!("swapchain texture in wrong state"),
                                }
                            }
                            vulkan_image_views.into_boxed_slice()
                        };

                    if *width != desired_width || *height != desired_height || *suboptimal {
                        let image_views = destroy_image_views(&mut texture_pool);
                        old_swapchain = *swapchain;
                        if !old_swapchain.is_null() {
                            self.destroyed_swapchains.lock().push((
                                Window::default(),
                                old_swapchain,
                                vk::SurfaceKHR::null(),
                                image_views,
                            ));
                        }
                        vulkan_swapchain.state = VulkanSwapchainState::Vacant;
                        continue;
                    }

                    let acquire = self.request_transient_semaphore(frame);
                    let mut image_index = 0;
                    match unsafe {
                        self.swapchain_fn.acquire_next_image2(
                            self.device,
                            &vk::AcquireNextImageInfoKHR {
                                swapchain: *swapchain,
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
                            old_swapchain = *swapchain;
                            let image_views = destroy_image_views(&mut texture_pool);
                            if !old_swapchain.is_null() {
                                self.destroyed_swapchains.lock().push((
                                    Window::default(),
                                    old_swapchain,
                                    vk::SurfaceKHR::null(),
                                    image_views,
                                ));
                            }
                            vulkan_swapchain.state = VulkanSwapchainState::Vacant;
                            continue;
                        }
                        result => vk_check!(result),
                    }

                    present_info.acquire = acquire;
                    present_info.image_index = image_index;
                    present_info.swapchain = *swapchain;
                    let view = image_views[image_index as usize];

                    return (*width, *height, view);
                }
            }
        }
    }

    fn create_command_buffer(
        &self,
        frame_token: &FrameToken,
        thread_token: &mut ThreadToken,
    ) -> CommandBufferToken {
        let command_buffer_pool = self
            .frame(frame_token)
            .command_buffer_pools
            .get_mut(thread_token);

        // We have consumed all available command buffers, need to allocate a new one.
        if command_buffer_pool.next_free_index >= command_buffer_pool.command_buffers.len() {
            let mut command_buffers = [vk::CommandBuffer::null(); 4];
            let allocate_info = vk::CommandBufferAllocateInfo {
                command_pool: command_buffer_pool.command_pool,
                level: vk::CommandBufferLevel::Primary,
                command_buffer_count: command_buffers.len() as u32,
                ..default()
            };
            vk_check!(self.device_fn.allocate_command_buffers(
                self.device,
                &allocate_info,
                command_buffers.as_mut_ptr()
            ));
            command_buffer_pool
                .command_buffers
                .extend(command_buffers.iter().copied().map(|command_buffer| {
                    VulkanCommandBuffer {
                        command_buffer,
                        swapchains_touched: HashMap::new(),
                    }
                }));
        }

        let index = command_buffer_pool.next_free_index;
        command_buffer_pool.next_free_index += 1;

        let command_buffer = command_buffer_pool.command_buffers[index].command_buffer;

        vk_check!(self.device_fn.begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo {
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..default()
            }
        ));

        CommandBufferToken {
            index,
            raw: command_buffer.as_raw(),
            phantom_unsend: PhantomUnsend {},
        }
    }

    fn cmd_set_bind_group(
        &self,
        frame_token: &FrameToken,
        thread_token: &mut ThreadToken,
        command_buffer_token: &CommandBufferToken,
        pipeline: Pipeline,
        layout: BindGroupLayout,
        bind_group_index: u32,
        bindings: &[Bind],
    ) {
        let arena = HybridArena::<4096>::new();

        let frame = self.frame(frame_token);

        let descriptor_set_layout = self.bind_group_layout_pool.lock().get(layout.0).unwrap().0;

        let mut descriptor_pool = *frame.descriptor_pool_pools.get(thread_token);
        let mut allocated_pool = false;
        let descriptor_set = loop {
            if descriptor_pool.is_null() {
                // Need to fetch a new descriptor pool
                descriptor_pool = self.request_descriptor_pool();
                frame.recycle_descriptor_pool(descriptor_pool);
                *frame.descriptor_pool_pools.get_mut(thread_token) = descriptor_pool;
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
            TypedBind::Texture(textures) => {
                let image_infos_iter = textures.iter().map(|texture| {
                    let image_view = self
                        .texture_pool
                        .lock()
                        .get(texture.0)
                        .unwrap()
                        .image_view();
                    vk::DescriptorImageInfo {
                        image_layout: vk::ImageLayout::ColorAttachmentOptimal,
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
            TypedBind::Buffer(buffers) => {
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
        });
        let write_descriptors = arena.alloc_slice_fill_iter(write_descriptors_iter);

        unsafe {
            self.device_fn
                .update_descriptor_sets(self.device, write_descriptors, &[])
        };

        let command_buffer = vk::CommandBuffer::from_raw(command_buffer_token.raw);

        let VulkanPipeline {
            pipeline: _,
            pipeline_layout,
            pipeline_bind_point,
        } = *self.pipeline_pool.lock().get(pipeline.0).unwrap();

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

    fn cmd_set_pipeline(&self, command_buffer_token: &CommandBufferToken, pipeline: Pipeline) {
        let command_buffer = vk::CommandBuffer::from_raw(command_buffer_token.raw);
        let VulkanPipeline {
            pipeline,
            pipeline_layout: _,
            pipeline_bind_point,
        } = *self.pipeline_pool.lock().get(pipeline.0).unwrap();
        unsafe {
            self.device_fn
                .cmd_bind_pipeline(command_buffer, pipeline_bind_point, pipeline)
        };
    }

    fn cmd_begin_rendering(
        &self,
        frame_token: &FrameToken,
        thread_token: &mut ThreadToken,
        command_buffer_token: &CommandBufferToken,
        desc: &crate::RenderingDesc,
    ) {
        let frame = self.frame(frame_token);
        let command_buffer = frame.command_buffer_mut(thread_token, command_buffer_token);

        let color_attachments = desc
            .color_attachments
            .iter()
            .map(|attachment| {
                let image_view = match self.texture_pool.lock().get(attachment.texture.0).unwrap() {
                    VulkanTextureHolder::Unique(texture) => texture.view,
                    VulkanTextureHolder::Shared(texture) => texture.view,
                    VulkanTextureHolder::Swapchain(texture) => {
                        assert!(
                            !command_buffer
                                .swapchains_touched
                                .contains_key(&texture.window),
                            "swapchain attached multiple times in a command buffer"
                        );
                        command_buffer.swapchains_touched.insert(
                            texture.window,
                            (
                                texture.image,
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
                            image: texture.image,
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
                            self.device_fn.cmd_pipeline_barrier2(
                                command_buffer.command_buffer,
                                &dependency_info,
                            )
                        };

                        texture.view
                    }
                };

                let (load_op, clear_value) = match attachment.load_op {
                    LoadOp::Load => (vk::AttachmentLoadOp::Load, vk::ClearValue::default()),
                    LoadOp::Clear(clear_value) => {
                        (vk::AttachmentLoadOp::Clear, vulkan_clear_value(clear_value))
                    }
                    LoadOp::DontCare => (vk::AttachmentLoadOp::DontCare, vk::ClearValue::default()),
                };

                let store_op = match attachment.store_op {
                    crate::StoreOp::Store => vk::AttachmentStoreOp::Store,
                    crate::StoreOp::DontCare => vk::AttachmentStoreOp::DontCare,
                };

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
            depth_attachment: None,
            stencil_attachment: None,
            ..default()
        };
        unsafe {
            self.device_fn
                .cmd_begin_rendering(command_buffer.command_buffer, &rendering_info)
        }
    }

    fn cmd_end_rendering(&self, command_buffer_token: &CommandBufferToken) {
        let command_buffer = vk::CommandBuffer::from_raw(command_buffer_token.raw);
        unsafe { self.device_fn.cmd_end_rendering(command_buffer) }
    }

    fn cmd_set_viewports(
        &self,
        command_buffer_token: &CommandBufferToken,
        viewports: &[crate::Viewport],
    ) {
        let command_buffer = vk::CommandBuffer::from_raw(command_buffer_token.raw);
        unsafe {
            self.device_fn.cmd_set_viewport_with_count(
                command_buffer,
                std::mem::transmute::<_, &[vk::Viewport]>(viewports), // yolo
            );
        }
    }

    fn cmd_set_scissors(
        &self,
        command_buffer_token: &CommandBufferToken,
        scissors: &[crate::Scissor],
    ) {
        let command_buffer = vk::CommandBuffer::from_raw(command_buffer_token.raw);
        unsafe {
            self.device_fn.cmd_set_scissor_with_count(
                command_buffer,
                std::mem::transmute::<_, &[vk::Rect2d]>(scissors), // yolo
            );
        }
    }

    fn cmd_draw(
        &self,
        command_buffer_token: &CommandBufferToken,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        let command_buffer = vk::CommandBuffer::from_raw(command_buffer_token.raw);
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

    fn submit(
        &self,
        frame_token: &FrameToken,
        thread_token: &mut ThreadToken,
        command_buffer_token: CommandBufferToken,
    ) {
        let fence = self.universal_queue_fence.fetch_add(1, Ordering::SeqCst) + 1;

        let frame = self.frame(frame_token);
        frame.universal_queue_fence.store(fence, Ordering::Relaxed);

        let command_buffer = frame.command_buffer_mut(thread_token, &command_buffer_token);

        for &(image, _) in command_buffer.swapchains_touched.values() {
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
                    .cmd_pipeline_barrier2(command_buffer.command_buffer, &dependency_info)
            };
        }

        vk_check!(self
            .device_fn
            .end_command_buffer(command_buffer.command_buffer));

        let mut wait_semaphores = Vec::new();
        let mut signal_semaphores = Vec::new();

        if !command_buffer.swapchains_touched.is_empty() {
            let mut present_swapchains = frame.present_swapchains.lock();

            for (swapchain, (_, stage_mask)) in command_buffer.swapchains_touched.drain() {
                let present_swapchain = present_swapchains
                    .get_mut(&swapchain)
                    .expect("presenting a swapchain that hasn't been acquired this frame");

                assert!(!present_swapchain.acquire.is_null());
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

        let command_buffer_infos = &[vk::CommandBufferSubmitInfo {
            command_buffer: command_buffer.command_buffer,
            device_mask: 1,
            ..default()
        }];

        vk_check!(self.device_fn.queue_submit2(
            self.universal_queue,
            &[vk::SubmitInfo2 {
                wait_semaphore_infos: wait_semaphores.as_slice().into(),
                command_buffer_infos: command_buffer_infos.into(),
                signal_semaphore_infos: signal_semaphores.as_slice().into(),
                ..default()
            }],
            vk::Fence::null()
        ));
    }

    fn begin_frame(&self) -> FrameToken {
        let device_fn = &self.device_fn;
        let device = self.device;

        let mut frame_token = self.frame_counter.acquire(self);
        let frame = self.frame_mut(&mut frame_token);

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

        for pool in frame.descriptor_pool_pools.slots_mut() {
            *pool = vk::DescriptorPool::null()
        }

        for pool in frame.command_buffer_pools.slots_mut() {
            if pool.next_free_index == 0 {
                continue;
            }

            vk_check!(device_fn.reset_command_pool(
                device,
                pool.command_pool,
                vk::CommandPoolResetFlags::default()
            ));

            pool.next_free_index = 0;
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
            .expire(|(window, swapchain, surface, image_views)| {
                self.destroy_swapchain(window, surface, swapchain, &image_views);
            });

        frame_token
    }

    fn end_frame(&self, mut frame_token: FrameToken) {
        let frame = self.frame_mut(&mut frame_token);

        let present_swapchains = frame.present_swapchains.get_mut();
        if !present_swapchains.is_empty() {
            let mut windows = Vec::new();
            let mut wait_semaphores = Vec::new();
            let mut swapchains = Vec::new();
            let mut swapchain_image_indices = Vec::new();
            let mut results = Vec::new();

            for (window, present_info) in present_swapchains.drain() {
                windows.push(window);
                wait_semaphores.push(present_info.release);
                swapchains.push(present_info.swapchain);
                swapchain_image_indices.push(present_info.image_index);
            }

            results.resize_with(swapchains.len(), || vk::Result::Success);

            let present_info = vk::PresentInfoKHR {
                wait_semaphores: wait_semaphores.as_slice().into(),
                swapchains: (swapchains.as_slice(), swapchain_image_indices.as_slice()).into(),
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

        self.frame_counter.release(frame_token);
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
}

impl<'app> Drop for VulkanDevice<'app> {
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

            for pool in frame.command_buffer_pools.slots_mut() {
                if !pool.command_buffers.is_empty() {
                    let command_buffers = pool
                        .command_buffers
                        .iter()
                        .map(|x| x.command_buffer)
                        .collect::<Vec<_>>();
                    unsafe {
                        device_fn.free_command_buffers(
                            device,
                            pool.command_pool,
                            command_buffers.as_slice(),
                        )
                    };
                }

                unsafe { device_fn.destroy_command_pool(device, pool.command_pool, None) }
            }
        }

        for buffer in self.buffer_pool.get_mut().values() {
            unsafe { device_fn.destroy_buffer(device, buffer.buffer, None) }
            unsafe { device_fn.free_memory(device, buffer.memory.memory, None) }
        }

        {
            let mut image_views = Vec::new();
            let mut images = Vec::new();
            for texture in self.texture_pool.get_mut().values() {
                match texture {
                    VulkanTextureHolder::Unique(texture) => {
                        image_views.push(texture.view);
                        images.push(texture.texture.image)
                    }
                    VulkanTextureHolder::Shared(texture) => {
                        image_views.push(texture.view);
                    }
                    VulkanTextureHolder::Swapchain(texture) => {
                        image_views.push(texture.view);
                    }
                }
            }

            for image_view in image_views {
                unsafe { device_fn.destroy_image_view(device, image_view, None) }
            }

            for image in images {
                unsafe { device_fn.destroy_image(device, image, None) }
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
            for (_, (window, swapchain, surface, image_views)) in destroyed_swapchains {
                self.destroy_swapchain(window, surface, swapchain, &image_views);
            }
        }

        for (_, swapchain) in self.swapchains.get_mut().iter() {
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
            unsafe {
                self.surface_fn
                    .destroy_surface(instance, swapchain.surface, None)
            }
        }

        unsafe { device_fn.destroy_device(device, None) }
        unsafe { self.instance_fn.destroy_instance(self.instance, None) };
    }
}
