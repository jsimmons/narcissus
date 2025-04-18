use std::{
    cell::{Cell, RefCell, UnsafeCell},
    collections::{HashMap, VecDeque},
    ffi::CStr,
    marker::PhantomData,
    os::raw::c_char,
    ptr::NonNull,
    sync::atomic::{AtomicU64, Ordering},
};

use narcissus_core::{
    Arc, Arena, HybridArena, Mutex, PhantomUnsend, Pool, Widen, default, is_aligned_to,
    manual_arc::{self, ManualArc},
    raw_window::AsRawWindow,
};

use physical_device_features::VulkanPhysicalDeviceFeatures;
use physical_device_properties::VulkanPhysicalDeviceProperties;
use vulkan_sys::{self as vk};

use crate::{
    Bind, BindDesc, BindGroupLayout, BindingType, Buffer, BufferAddress, BufferArg, BufferDesc,
    BufferImageCopy, BufferUsageFlags, CmdEncoder, ComputePipelineDesc, Device, Extent2d, Extent3d,
    Frame, GlobalBarrier, GpuConcurrent, GraphicsPipelineDesc, Image, ImageBarrier, ImageBlit,
    ImageDesc, ImageDimension, ImageLayout, ImageTiling, ImageViewDesc, IndexType, MemoryLocation,
    Offset2d, Offset3d, PersistentBuffer, Pipeline, PipelineLayout, Sampler, SamplerAddressMode,
    SamplerCompareOp, SamplerDesc, SamplerFilter, ShaderStageFlags, SpecConstant,
    SwapchainConfigurator, SwapchainImage, SwapchainOutOfDateError, ThreadToken, TransientBuffer,
    TypedBind, frame_counter::FrameCounter, mapped_buffer::TransientBindGroup,
};

mod allocator;
mod barrier;
mod convert;
mod libc;
mod physical_device_features;
mod physical_device_properties;
mod wsi;

use self::{
    allocator::{VulkanAllocator, VulkanMemory},
    barrier::{vulkan_image_memory_barrier, vulkan_memory_barrier},
    convert::*,
    wsi::{VulkanWsi, VulkanWsiFrame},
};

/// Important constant data configuration for the vulkan backend.
pub struct VulkanConstants {
    /// Per-frame data is duplicated this many times. Additional frames will
    /// increase the latency between submission and when the frame fence is waited
    /// on. This subsequently, increases the latency between submission and the
    /// recycling of resources.
    num_frames: usize,

    /// How many frames to delay swapchain semaphore release and swapchain
    /// destruction. There's no correct answer here (spec bug) we're just picking a
    /// big number and hoping for the best.
    ///
    /// This will not be used if VK_EXT_swapchain_maintenance1 is available.
    swapchain_semaphore_destroy_delay: u64,

    /// How large should transient buffers be, this will limit the maximum size of
    /// transient allocations.
    transient_buffer_size: u64,

    /// Default size for backing allocations used by the Tlsf allocator.
    tlsf_default_super_block_size: u64,

    /// For memory heaps that are smaller than `tlsf_default_super_block_size` *
    /// `tlsf_small_super_block_divisor`, use heap size divided by
    /// `tlsf_small_super_block_divisor` as the super block size.
    tlsf_small_super_block_divisor: u64,

    /// Force use of separate allocators for optimal tiling images and buffers.
    tlsf_force_segregated_non_linear_allocator: bool,

    /// The max number of descriptor sets allocatable from each descriptor pool.
    descriptor_pool_max_sets: u32,
    /// The number of sampler descriptors available in each descriptor pool.
    descriptor_pool_sampler_count: u32,
    /// The number of uniform buffer descriptors available in each descriptor pool.
    descriptor_pool_uniform_buffer_count: u32,
    /// The number of storage buffer descriptors available in each descriptor pool.
    descriptor_pool_storage_buffer_count: u32,
    /// The number of sampled image descriptors available in each descriptor pool.
    descriptor_pool_sampled_image_count: u32,
}

const VULKAN_CONSTANTS: VulkanConstants = VulkanConstants {
    num_frames: 2,
    swapchain_semaphore_destroy_delay: 8,
    transient_buffer_size: 4 * 1024 * 1024,
    tlsf_default_super_block_size: 128 * 1024 * 1024,
    tlsf_small_super_block_divisor: 16,
    tlsf_force_segregated_non_linear_allocator: false,
    descriptor_pool_max_sets: 500,
    descriptor_pool_sampler_count: 100,
    descriptor_pool_uniform_buffer_count: 500,
    descriptor_pool_storage_buffer_count: 500,
    descriptor_pool_sampled_image_count: 500,
};

#[macro_export]
macro_rules! vk_check {
    ($e:expr) => ({
        let e = $e;
        if e != vulkan_sys::Result::Success {
            panic!("assertion failed: `result == vk::Result::Success`: \n value: `{:?}`", e);
        }
    });
    ($e:expr, $($msg_args:tt)+) => ({
        let e = $e;
        if e != vulkan_sys::::Result::Success {
            panic!("assertion failed: `result == vk::Result::Success`: \n value: `{:?}: {}`", e, format_args!($($msg_args)+));
        }
    })
}

#[must_use]
fn vk_vec<T, F: FnMut(&mut u32, *mut T) -> vulkan_sys::Result>(mut f: F) -> Vec<T> {
    let mut count = 0;
    vk_check!(f(&mut count, std::ptr::null_mut()));
    let mut v = Vec::with_capacity(count.widen());
    vk_check!(f(&mut count, v.as_mut_ptr()));
    unsafe { v.set_len(count as usize) };
    v
}

impl From<Extent2d> for vk::Extent2d {
    fn from(extent: Extent2d) -> Self {
        vk::Extent2d {
            width: extent.width,
            height: extent.height,
        }
    }
}

impl From<Extent3d> for vk::Extent3d {
    fn from(extent: Extent3d) -> Self {
        vk::Extent3d {
            width: extent.width,
            height: extent.height,
            depth: extent.depth,
        }
    }
}

impl From<Offset2d> for vk::Offset2d {
    fn from(extent: Offset2d) -> Self {
        vk::Offset2d {
            x: extent.x,
            y: extent.y,
        }
    }
}

impl From<Offset3d> for vk::Offset3d {
    fn from(extent: Offset3d) -> Self {
        vk::Offset3d {
            x: extent.x,
            y: extent.y,
            z: extent.z,
        }
    }
}

fn vulkan_shader_module(
    device_fn: &vk::DeviceFunctions,
    device: vk::Device,
    spirv: &[u8],
) -> vk::ShaderModule {
    assert!(
        is_aligned_to(spirv.as_ptr(), 4),
        "spir-v must be aligned on a 4 byte boundary"
    );
    let create_info = vk::ShaderModuleCreateInfo {
        code: spirv.into(),
        ..default()
    };
    let mut shader_module = vk::ShaderModule::null();
    vk_check!(unsafe {
        device_fn.create_shader_module(device, &create_info, None, &mut shader_module)
    });
    shader_module
}

struct VulkanBuffer {
    memory: VulkanMemory,
    buffer: vk::Buffer,
    address: vk::DeviceAddress,
    map_count: u64,
}

#[derive(Clone)]
struct VulkanImage {
    memory: VulkanMemory,
    image: vk::Image,
}

struct VulkanImageUnique {
    image: VulkanImage,
    view: vk::ImageView,
}

struct VulkanImageShared {
    image: ManualArc<VulkanImage>,
    view: vk::ImageView,
}

struct VulkanImageSwapchain {
    surface: vk::SurfaceKHR,
    image: vk::Image,
    view: vk::ImageView,
}

enum VulkanImageHolder {
    Unique(VulkanImageUnique),
    Shared(VulkanImageShared),
    Swapchain(VulkanImageSwapchain),
}

impl VulkanImageHolder {
    fn image(&self) -> vk::Image {
        match self {
            VulkanImageHolder::Unique(x) => x.image.image,
            VulkanImageHolder::Shared(_) => panic!(),
            VulkanImageHolder::Swapchain(x) => x.image,
        }
    }

    fn image_view(&self) -> vk::ImageView {
        match self {
            VulkanImageHolder::Unique(x) => x.view,
            VulkanImageHolder::Shared(x) => x.view,
            VulkanImageHolder::Swapchain(x) => x.view,
        }
    }
}

struct VulkanSampler(vk::Sampler);

struct VulkanBindGroupLayout {
    hash: blake3_smol::Hash,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

struct VulkanPipelineLayout {
    pipeline_layout: vk::PipelineLayout,
}

struct VulkanPipeline {
    pipeline: vk::Pipeline,
    pipeline_layout: Arc<VulkanPipelineLayout>,
    pipeline_bind_point: vk::PipelineBindPoint,
}

#[derive(Clone)]
struct VulkanBoundPipeline {
    pipeline_layout: vk::PipelineLayout,
    pipeline_bind_point: vk::PipelineBindPoint,
}

#[derive(Clone)]
struct VulkanTransientBuffer {
    buffer: vk::Buffer,
    address: u64,
    memory: VulkanMemory,
}

#[derive(Default)]
struct VulkanTransientBufferAllocator {
    offset: u64,
    current: Option<VulkanTransientBuffer>,
    used_buffers: Vec<VulkanTransientBuffer>,
}

impl VulkanTransientBufferAllocator {
    fn reset(&mut self) {
        self.current = None;
        self.offset = 0;
    }
}

struct VulkanTouchedSwapchain {
    image: vk::Image,
    layout: vk::ImageLayout,
    access_mask: vk::AccessFlags2,
    stage_mask: vk::PipelineStageFlags2,
}

struct VulkanCmdEncoder {
    #[cfg(debug_assertions)]
    in_render_pass: bool,
    command_buffer: vk::CommandBuffer,
    bound_pipeline: Option<VulkanBoundPipeline>,
    swapchains_touched: HashMap<vk::SurfaceKHR, VulkanTouchedSwapchain>,
}

impl Default for VulkanCmdEncoder {
    fn default() -> Self {
        Self {
            #[cfg(debug_assertions)]
            in_render_pass: false,
            command_buffer: default(),
            bound_pipeline: default(),
            swapchains_touched: default(),
        }
    }
}

struct VulkanCmdBufferPool {
    command_pool: vk::CommandPool,
    next_free_index: usize,
    command_buffers: Vec<vk::CommandBuffer>,
}

#[repr(align(64))]
struct VulkanPerThread {
    cmd_buffer_pool: RefCell<VulkanCmdBufferPool>,
    descriptor_pool: Cell<vk::DescriptorPool>,
    transient_buffer_allocator: RefCell<VulkanTransientBufferAllocator>,
    arena: Arena,
}

pub(crate) struct VulkanFrame {
    universal_queue_fence: AtomicU64,

    per_thread: GpuConcurrent<VulkanPerThread>,

    wsi: VulkanWsiFrame,

    destroyed_allocations: Mutex<VecDeque<VulkanMemory>>,
    destroyed_buffers: Mutex<VecDeque<vk::Buffer>>,
    destroyed_buffer_views: Mutex<VecDeque<vk::BufferView>>,
    destroyed_images: Mutex<VecDeque<vk::Image>>,
    destroyed_image_views: Mutex<VecDeque<vk::ImageView>>,
    destroyed_samplers: Mutex<VecDeque<vk::Sampler>>,
    destroyed_descriptor_set_layouts: Mutex<VecDeque<vk::DescriptorSetLayout>>,
    destroyed_pipelines: Mutex<VecDeque<vk::Pipeline>>,

    recycled_semaphores: Mutex<VecDeque<vk::Semaphore>>,
    recycled_descriptor_pools: Mutex<VecDeque<vk::DescriptorPool>>,
}

impl VulkanFrame {
    fn recycle_semaphore(&self, semaphore: vk::Semaphore) {
        self.recycled_semaphores.lock().push_back(semaphore);
    }

    fn recycle_descriptor_pool(&self, descriptor_pool: vk::DescriptorPool) {
        self.recycled_descriptor_pools
            .lock()
            .push_back(descriptor_pool)
    }
}

pub(crate) struct VulkanDevice {
    instance: vk::Instance,
    physical_device: vk::PhysicalDevice,
    device: vk::Device,

    universal_queue: vk::Queue,
    universal_queue_fence: AtomicU64,
    universal_queue_semaphore: vk::Semaphore,
    universal_queue_family_index: u32,

    frame_counter: FrameCounter,
    frames: Box<[UnsafeCell<VulkanFrame>; VULKAN_CONSTANTS.num_frames]>,

    wsi: Box<VulkanWsi>,

    image_pool: Mutex<Pool<VulkanImageHolder>>,
    buffer_pool: Mutex<Pool<VulkanBuffer>>,
    sampler_pool: Mutex<Pool<VulkanSampler>>,
    bind_group_layout_pool: Mutex<Pool<VulkanBindGroupLayout>>,
    pipeline_pool: Mutex<Pool<VulkanPipeline>>,

    pipeline_layout_cache: Mutex<HashMap<blake3_smol::Hash, Arc<VulkanPipelineLayout>>>,

    recycled_fences: Mutex<VecDeque<vk::Fence>>,
    recycled_semaphores: Mutex<VecDeque<vk::Semaphore>>,
    recycled_descriptor_pools: Mutex<VecDeque<vk::DescriptorPool>>,
    recycled_transient_buffers: Mutex<VecDeque<VulkanTransientBuffer>>,

    allocator: VulkanAllocator,

    physical_device_properties: Box<VulkanPhysicalDeviceProperties>,
    physical_device_memory_properties: Box<vk::PhysicalDeviceMemoryProperties>,
    _physical_device_features: Box<VulkanPhysicalDeviceFeatures>,

    _global_fn: vk::GlobalFunctions,
    instance_fn: vk::InstanceFunctions,
    device_fn: vk::DeviceFunctions,

    #[cfg(feature = "debug_markers")]
    debug_utils_fn: Option<vk::DebugUtilsFunctions>,
}

impl VulkanDevice {
    pub(crate) fn new() -> Self {
        let get_proc_addr = unsafe {
            let module = libc::dlopen(
                c"libvulkan.so.1".as_ptr(),
                libc::RTLD_NOW | libc::RTLD_LOCAL,
            );
            libc::dlsym(module, (c"vkGetInstanceProcAddr").as_ptr())
        };

        let global_fn = unsafe { vk::GlobalFunctions::new(get_proc_addr) };

        let api_version = {
            let mut api_version = 0;
            vk_check!(global_fn.enumerate_instance_version(&mut api_version));
            api_version
        };

        if api_version < vk::VERSION_1_2 {
            panic!("instance does not support vulkan 1.2")
        }

        let layer_properties = vk_vec(|count, ptr| unsafe {
            global_fn.enumerate_instance_layer_properties(count, ptr)
        });

        let mut enabled_layers = vec![];

        if cfg!(debug_assertions) {
            for layer in &layer_properties {
                let layer_name = CStr::from_bytes_until_nul(&layer.layer_name).unwrap();
                if layer_name == c"VK_LAYER_KHRONOS_validation" {
                    enabled_layers.push(layer_name);
                    break;
                }
            }
        }

        let enabled_layers = enabled_layers
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<*const c_char>>();

        let extension_properties = vk_vec(|count, ptr| unsafe {
            global_fn.enumerate_instance_extension_properties(std::ptr::null(), count, ptr)
        });

        let mut enabled_extensions = vec![];

        let mut has_get_surface_capabilities2 = false;
        #[cfg(feature = "debug_markers")]
        let mut has_debug_utils = false;
        for extension in &extension_properties {
            let extension_name = CStr::from_bytes_until_nul(&extension.extension_name).unwrap();
            match extension_name.to_str().unwrap() {
                "VK_KHR_get_surface_capabilities2" => {
                    has_get_surface_capabilities2 = true;
                    enabled_extensions.push(extension_name);
                }

                #[cfg(feature = "debug_markers")]
                "VK_EXT_debug_utils" => {
                    has_debug_utils = true;
                    enabled_extensions.push(extension_name);
                }
                _ => {}
            }
        }

        assert!(has_get_surface_capabilities2);

        let mut wsi_support = default();
        VulkanWsi::check_instance_extensions(
            &extension_properties,
            &mut enabled_extensions,
            &mut wsi_support,
        );

        let enabled_extensions = enabled_extensions
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<*const c_char>>();

        let instance = {
            let application_info = vk::ApplicationInfo {
                application_name: c"TRIANGLE".as_ptr(),
                application_version: 0,
                engine_name: c"NARCISSUS".as_ptr(),
                engine_version: 0,
                api_version: vk::VERSION_1_3,
                ..default()
            };
            let create_info = vk::InstanceCreateInfo {
                enabled_layers: enabled_layers.as_slice().into(),
                enabled_extension_names: enabled_extensions.as_slice().into(),
                application_info: Some(&application_info),
                ..default()
            };
            let mut instance = vk::Instance::null();
            vk_check!(unsafe { global_fn.create_instance(&create_info, None, &mut instance) });
            instance
        };

        let instance_fn = vk::InstanceFunctions::new(&global_fn, instance, vk::VERSION_1_2);

        let physical_devices = vk_vec(|count, ptr| unsafe {
            instance_fn.enumerate_physical_devices(instance, count, ptr)
        });

        let mut physical_device_properties: Box<VulkanPhysicalDeviceProperties> = default();
        let mut physical_device_features: Box<VulkanPhysicalDeviceFeatures> = default();

        let physical_device = physical_devices
            .iter()
            .copied()
            .find(|&physical_device| {
                unsafe {
                    instance_fn.get_physical_device_properties2(
                        physical_device,
                        physical_device_properties.link(),
                    );
                    instance_fn.get_physical_device_features2(
                        physical_device,
                        physical_device_features.link(),
                    );
                }

                physical_device_properties.api_version() >= vk::VERSION_1_3
                    && physical_device_features.dynamic_rendering()
                    && physical_device_features.subgroup_size_control()
                    && physical_device_features.maintenance4()
                    && physical_device_features.compute_full_subgroups()
                    && physical_device_features.timeline_semaphore()
                    && physical_device_features.descriptor_indexing()
                    && physical_device_features.descriptor_binding_partially_bound()
                    && physical_device_features.draw_indirect_count()
                    && physical_device_features.uniform_buffer_standard_layout()
            })
            .expect("no supported physical devices reported");

        let physical_device_memory_properties = unsafe {
            let mut memory_properties = Box::<vk::PhysicalDeviceMemoryProperties>::default();
            instance_fn
                .get_physical_device_memory_properties(physical_device, memory_properties.as_mut());
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

            let extension_properties = vk_vec(|count, ptr| unsafe {
                instance_fn.enumerate_device_extension_properties(
                    physical_device,
                    std::ptr::null(),
                    count,
                    ptr,
                )
            });

            let mut enabled_extensions = vec![];

            VulkanWsi::check_device_extensions(
                &extension_properties,
                &mut enabled_extensions,
                &mut wsi_support,
            );

            let enabled_extensions = enabled_extensions
                .iter()
                .map(|x| x.as_ptr())
                .collect::<Vec<*const c_char>>();

            let mut enabled_features: Box<VulkanPhysicalDeviceFeatures> = default();

            enabled_features.set_buffer_device_address(true);
            enabled_features.set_compute_full_subgroups(true);
            enabled_features.set_descriptor_binding_partially_bound(true);
            enabled_features.set_descriptor_indexing(true);
            enabled_features.set_draw_indirect_count(true);
            enabled_features.set_dynamic_rendering(true);
            enabled_features.set_maintenance4(true);
            enabled_features.set_shader_storage_image_read_without_format(true);
            enabled_features.set_shader_storage_image_write_without_format(true);
            enabled_features.set_subgroup_size_control(true);
            enabled_features.set_synchronization2(true);
            enabled_features.set_timeline_semaphore(true);
            enabled_features.set_uniform_buffer_standard_layout(true);

            if wsi_support.swapchain_maintenance1() {
                enabled_features.set_swapchain_maintenance1(true);
            }

            let create_info = vk::DeviceCreateInfo {
                _next: enabled_features.link() as *mut _ as *const _,
                enabled_extension_names: enabled_extensions.as_slice().into(),
                queue_create_infos: device_queue_create_infos.into(),
                ..default()
            };

            let mut device = vk::Device::null();
            vk_check!(unsafe {
                instance_fn.create_device(physical_device, &create_info, None, &mut device)
            });
            device
        };

        let device_fn = vk::DeviceFunctions::new(&instance_fn, device, vk::VERSION_1_3);

        #[cfg(feature = "debug_markers")]
        let debug_utils_fn = if has_debug_utils {
            Some(vk::DebugUtilsFunctions::new(
                &global_fn,
                &instance_fn,
                instance,
                device,
            ))
        } else {
            None
        };

        let wsi = Box::new(VulkanWsi::new(&global_fn, instance, wsi_support));

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
            vk_check!(unsafe {
                device_fn.create_semaphore(device, &create_info, None, &mut semaphore)
            });
            semaphore
        };

        let frames = Box::new(std::array::from_fn(|_| {
            let per_thread = GpuConcurrent::new(|| {
                let command_pool = {
                    let create_info = vk::CommandPoolCreateInfo {
                        flags: vk::CommandPoolCreateFlags::TRANSIENT,
                        queue_family_index,
                        ..default()
                    };
                    let mut pool = vk::CommandPool::null();
                    vk_check!(unsafe {
                        device_fn.create_command_pool(device, &create_info, None, &mut pool)
                    });
                    pool
                };
                let cmd_buffer_pool = VulkanCmdBufferPool {
                    command_pool,
                    command_buffers: Vec::new(),
                    next_free_index: 0,
                };

                VulkanPerThread {
                    cmd_buffer_pool: RefCell::new(cmd_buffer_pool),
                    descriptor_pool: Cell::new(vk::DescriptorPool::null()),
                    transient_buffer_allocator: default(),
                    arena: Arena::new(),
                }
            });

            UnsafeCell::new(VulkanFrame {
                per_thread,
                universal_queue_fence: AtomicU64::new(universal_queue_fence),
                wsi: default(),
                destroyed_allocations: default(),
                destroyed_buffers: default(),
                destroyed_buffer_views: default(),
                destroyed_images: default(),
                destroyed_image_views: default(),
                destroyed_samplers: default(),
                destroyed_descriptor_set_layouts: default(),
                destroyed_pipelines: default(),
                recycled_semaphores: default(),
                recycled_descriptor_pools: default(),
            })
        }));

        let allocator = VulkanAllocator::new(
            physical_device_properties.limits().buffer_image_granularity,
            physical_device_memory_properties.as_ref(),
        );

        Self {
            instance,
            physical_device,
            device,

            universal_queue,
            universal_queue_fence: AtomicU64::new(universal_queue_fence),
            universal_queue_semaphore,
            universal_queue_family_index: queue_family_index,

            frame_counter: FrameCounter::new(),
            frames,

            wsi,

            image_pool: default(),
            buffer_pool: default(),
            sampler_pool: default(),
            bind_group_layout_pool: default(),
            pipeline_pool: default(),

            pipeline_layout_cache: default(),

            recycled_fences: default(),
            recycled_semaphores: default(),
            recycled_descriptor_pools: default(),
            recycled_transient_buffers: default(),

            allocator,

            physical_device_properties,
            physical_device_memory_properties,
            _physical_device_features: physical_device_features,

            _global_fn: global_fn,
            instance_fn,
            device_fn,

            #[cfg(feature = "debug_markers")]
            debug_utils_fn,
        }
    }

    fn frame<'token>(&self, frame: &'token Frame) -> &'token VulkanFrame {
        frame.check_device(self as *const _ as usize);
        frame.check_frame_counter(self.frame_counter.load());
        // SAFETY: Reference is bound to the frame exposed by the API. only one frame
        // can be valid at a time. The returned VulkanFrame is only valid so long as we
        // have a ref on the frame.
        unsafe { &*self.frames[frame.frame_index % VULKAN_CONSTANTS.num_frames].get() }
    }

    fn frame_mut<'token>(&self, frame: &'token mut Frame) -> &'token mut VulkanFrame {
        frame.check_device(self as *const _ as usize);
        frame.check_frame_counter(self.frame_counter.load());
        // SAFETY: Reference is bound to the frame exposed by the API. only one frame
        // can be valid at a time. The returned VulkanFrame is only valid so long as we
        // have a ref on the frame.
        unsafe { &mut *self.frames[frame.frame_index % VULKAN_CONSTANTS.num_frames].get() }
    }

    fn cmd_encoder_mut<'a>(&self, cmd_encoder: &'a mut CmdEncoder) -> &'a mut VulkanCmdEncoder {
        // SAFETY: `CmdEncoder`s can't outlive a frame, and the memory for a cmd_encoder
        // is reset when the frame ends. So the pointer contained in the cmd_encoder is
        // always valid while the `CmdEncoder` is valid. They can't cloned, copied or be
        // sent between threads, and we have a mutable reference.
        unsafe {
            NonNull::new_unchecked(cmd_encoder.cmd_encoder_addr as *mut VulkanCmdEncoder).as_mut()
        }
    }

    fn request_descriptor_pool(&self) -> vk::DescriptorPool {
        if let Some(descriptor_pool) = self.recycled_descriptor_pools.lock().pop_front() {
            descriptor_pool
        } else {
            let pool_sizes = &[
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::Sampler,
                    descriptor_count: VULKAN_CONSTANTS.descriptor_pool_sampler_count,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::UniformBuffer,
                    descriptor_count: VULKAN_CONSTANTS.descriptor_pool_uniform_buffer_count,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::StorageBuffer,
                    descriptor_count: VULKAN_CONSTANTS.descriptor_pool_storage_buffer_count,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::SampledImage,
                    descriptor_count: VULKAN_CONSTANTS.descriptor_pool_sampled_image_count,
                },
            ];
            let mut descriptor_pool = vk::DescriptorPool::null();
            let create_info = vk::DescriptorPoolCreateInfo {
                max_sets: VULKAN_CONSTANTS.descriptor_pool_max_sets,
                pool_sizes: pool_sizes.into(),
                ..default()
            };
            vk_check!(unsafe {
                self.device_fn.create_descriptor_pool(
                    self.device,
                    &create_info,
                    None,
                    &mut descriptor_pool,
                )
            });
            descriptor_pool
        }
    }

    fn request_fence(&self) -> vk::Fence {
        if let Some(fence) = self.recycled_fences.lock().pop_front() {
            let fences = &[fence];
            vk_check!(unsafe { self.device_fn.reset_fences(self.device, fences) });
            fence
        } else {
            let mut fence = vk::Fence::null();
            let create_info = vk::FenceCreateInfo::default();
            vk_check!(unsafe {
                self.device_fn
                    .create_fence(self.device, &create_info, None, &mut fence)
            });
            fence
        }
    }

    fn request_semaphore(&self) -> vk::Semaphore {
        if let Some(semaphore) = self.recycled_semaphores.lock().pop_front() {
            semaphore
        } else {
            let mut semaphore = vk::Semaphore::null();
            let create_info = vk::SemaphoreCreateInfo::default();
            vk_check!(unsafe {
                self.device_fn
                    .create_semaphore(self.device, &create_info, None, &mut semaphore)
            });
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
    }
}

impl Device for VulkanDevice {
    fn create_buffer(&self, desc: &BufferDesc) -> Buffer {
        let queue_family_indices = &[self.universal_queue_family_index];

        let create_info = vk::BufferCreateInfo {
            size: desc.size as u64,
            usage: vulkan_buffer_usage_flags(desc.usage),
            queue_family_indices: queue_family_indices.into(),
            sharing_mode: vk::SharingMode::Exclusive,
            ..default()
        };
        let mut buffer = vk::Buffer::null();
        vk_check!(unsafe {
            self.device_fn
                .create_buffer(self.device, &create_info, None, &mut buffer)
        });

        let memory = self.allocate_memory(
            desc.memory_location,
            false,
            desc.host_mapped,
            allocator::VulkanAllocationResource::Buffer(buffer),
        );

        unsafe {
            self.device_fn.bind_buffer_memory2(
                self.device,
                &[vk::BindBufferMemoryInfo {
                    buffer,
                    memory: memory.device_memory(),
                    offset: memory.offset(),
                    ..default()
                }],
            )
        };

        let address = unsafe {
            self.device_fn.get_buffer_device_address(
                self.device,
                &vk::BufferDeviceAddressInfo {
                    buffer,
                    ..default()
                },
            )
        };

        let handle = self.buffer_pool.lock().insert(VulkanBuffer {
            memory,
            buffer,
            address,
            map_count: 0,
        });

        Buffer(handle)
    }

    fn create_persistent_buffer<'device>(
        &'device self,
        desc: &BufferDesc,
    ) -> PersistentBuffer<'device> {
        assert!(desc.host_mapped);

        let buffer = self.create_buffer(desc);
        unsafe {
            let ptr = std::ptr::NonNull::new(self.map_buffer(buffer))
                .expect("failed to map buffer memory");

            PersistentBuffer {
                ptr,
                len: desc.size,
                buffer,
                phantom: PhantomData,
            }
        }
    }

    fn create_image(&self, image_desc: &ImageDesc) -> Image {
        debug_assert_ne!(image_desc.layer_count, 0, "layers must be at least one");
        debug_assert_ne!(image_desc.width, 0, "width must be at least one");
        debug_assert_ne!(image_desc.height, 0, "height must be at least one");
        debug_assert_ne!(image_desc.depth, 0, "depth must be at least one");

        if image_desc.dimension == ImageDimension::Type3d {
            debug_assert_eq!(image_desc.layer_count, 1, "3d image arrays are illegal");
        }

        if image_desc.dimension == ImageDimension::TypeCube {
            debug_assert!(
                image_desc.layer_count % 6 == 0,
                "cubemaps must have 6 layers each"
            );
            debug_assert_eq!(image_desc.depth, 1, "cubemap faces must be 2d");
        }

        let mut flags = vk::ImageCreateFlags::default();
        if image_desc.dimension == ImageDimension::TypeCube {
            flags |= vk::ImageCreateFlags::CUBE_COMPATIBLE
        }

        let image_type = match image_desc.dimension {
            ImageDimension::Type1d => vk::ImageType::Type1d,
            ImageDimension::Type2d => vk::ImageType::Type2d,
            ImageDimension::Type3d => vk::ImageType::Type3d,
            ImageDimension::TypeCube => vk::ImageType::Type2d,
        };
        let format = vulkan_format(image_desc.format);
        let extent = vk::Extent3d {
            width: image_desc.width,
            height: image_desc.height,
            depth: image_desc.depth,
        };

        let tiling = vulkan_image_tiling(image_desc.tiling);
        let usage = vulkan_image_usage_flags(image_desc.usage);

        let queue_family_indices = &[self.universal_queue_family_index];
        let create_info = vk::ImageCreateInfo {
            flags,
            image_type,
            format,
            extent,
            mip_levels: image_desc.mip_levels,
            array_layers: image_desc.layer_count,
            samples: vk::SampleCountFlags::SAMPLE_COUNT_1,
            tiling,
            usage,
            sharing_mode: vk::SharingMode::Exclusive,
            queue_family_indices: queue_family_indices.into(),
            initial_layout: vk::ImageLayout::Undefined,
            ..default()
        };

        let mut image = vk::Image::null();
        vk_check!(unsafe {
            self.device_fn
                .create_image(self.device, &create_info, None, &mut image)
        });

        let memory = self.allocate_memory(
            image_desc.memory_location,
            image_desc.tiling == ImageTiling::Optimal,
            image_desc.host_mapped,
            allocator::VulkanAllocationResource::Image(image),
        );

        unsafe {
            self.device_fn.bind_image_memory2(
                self.device,
                &[vk::BindImageMemoryInfo {
                    image,
                    memory: memory.device_memory(),
                    offset: memory.offset(),
                    ..default()
                }],
            )
        };

        let view_type = vulkan_image_view_type(image_desc.layer_count, image_desc.dimension);
        let aspect_mask = vulkan_aspect_for_format(image_desc.format);
        let create_info = vk::ImageViewCreateInfo {
            image,
            view_type,
            format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: image_desc.mip_levels,
                base_array_layer: 0,
                layer_count: image_desc.layer_count,
            },
            ..default()
        };

        let mut view = vk::ImageView::null();
        vk_check!(unsafe {
            self.device_fn
                .create_image_view(self.device, &create_info, None, &mut view)
        });

        let image = VulkanImageUnique {
            image: VulkanImage { image, memory },
            view,
        };

        let handle = self
            .image_pool
            .lock()
            .insert(VulkanImageHolder::Unique(image));

        Image(handle)
    }

    fn create_image_view(&self, image_view_desc: &ImageViewDesc) -> Image {
        let mut image_pool = self.image_pool.lock();
        let image = image_pool.get_mut(image_view_desc.image.0).unwrap();

        let arc_image;
        match image {
            VulkanImageHolder::Shared(shared) => arc_image = shared.image.clone(),
            VulkanImageHolder::Unique(unique) => {
                let unique_image = ManualArc::new(unique.image.clone());
                arc_image = unique_image.clone();
                let unique_view = unique.view;
                *image = VulkanImageHolder::Shared(VulkanImageShared {
                    image: unique_image,
                    view: unique_view,
                })
            }
            VulkanImageHolder::Swapchain(_) => {
                panic!("unable to create additional views of swapchain images")
            }
        }

        let subresource_range = vulkan_subresource_range(&image_view_desc.subresource_range);
        let view_type = vulkan_image_view_type(
            image_view_desc.subresource_range.array_layer_count,
            image_view_desc.dimension,
        );
        let format = vulkan_format(image_view_desc.format);

        let create_info = vk::ImageViewCreateInfo {
            image: arc_image.image,
            view_type,
            format,
            subresource_range,
            ..default()
        };

        let mut view = vk::ImageView::null();
        vk_check!(unsafe {
            self.device_fn
                .create_image_view(self.device, &create_info, None, &mut view)
        });

        let handle = image_pool.insert(VulkanImageHolder::Shared(VulkanImageShared {
            image: arc_image,
            view,
        }));

        Image(handle)
    }

    fn create_sampler(&self, sampler_desc: &SamplerDesc) -> Sampler {
        let (filter, mipmap_mode, anisotropy_enable) = match sampler_desc.filter {
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

        let address_mode = match sampler_desc.address_mode {
            SamplerAddressMode::Wrap => vk::SamplerAddressMode::Repeat,
            SamplerAddressMode::Clamp => vk::SamplerAddressMode::ClampToEdge,
        };

        let (compare_enable, compare_op) = match sampler_desc.compare_op {
            None => (vk::Bool32::False, vk::CompareOp::Always),
            Some(SamplerCompareOp::Less) => (vk::Bool32::True, vk::CompareOp::Less),
            Some(SamplerCompareOp::LessEq) => (vk::Bool32::True, vk::CompareOp::LessOrEqual),
            Some(SamplerCompareOp::Greater) => (vk::Bool32::True, vk::CompareOp::Greater),
            Some(SamplerCompareOp::GreaterEq) => (vk::Bool32::True, vk::CompareOp::GreaterOrEqual),
        };

        let unnormalized_coordinates = sampler_desc.unnormalized_coordinates.into();

        let mut sampler = vk::Sampler::null();
        vk_check!(unsafe {
            self.device_fn.create_sampler(
                self.device,
                &vk::SamplerCreateInfo {
                    max_lod: sampler_desc.max_lod,
                    min_lod: sampler_desc.min_lod,
                    mip_lod_bias: sampler_desc.mip_lod_bias,
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
                    unnormalized_coordinates,
                    ..default()
                },
                None,
                &mut sampler,
            )
        });

        let handle = self.sampler_pool.lock().insert(VulkanSampler(sampler));
        Sampler(handle)
    }

    fn create_bind_group_layout(&self, binds_desc: &[BindDesc]) -> BindGroupLayout {
        let arena = HybridArena::<256>::new();
        let mut hasher = blake3_smol::Hasher::new();

        for bind_desc in binds_desc {
            hasher.update(&bind_desc.slot.to_le_bytes());
            hasher.update(&bind_desc.stages.as_raw().to_le_bytes());
            hasher.update(&(bind_desc.binding_type as u32).to_le_bytes());
            hasher.update(&bind_desc.count.to_le_bytes());
        }

        let layout_bindings: &mut [vulkan_sys::DescriptorSetLayoutBinding] = arena
            .alloc_slice_fill_iter(binds_desc.iter().enumerate().map(|(i, bind_desc)| {
                let immutable_samplers = if !bind_desc.immutable_samplers.is_empty() {
                    assert_eq!(
                        bind_desc.binding_type,
                        BindingType::Sampler,
                        "can only use immutable samplers with sampler binds"
                    );
                    assert_eq!(
                        bind_desc.count as usize,
                        bind_desc.immutable_samplers.len(),
                        "number of immutable samplers must match bind count"
                    );

                    let sampler_pool = self.sampler_pool.lock();

                    let immutable_samplers = arena.alloc_slice_fill_iter(
                        bind_desc.immutable_samplers.iter().map(|sampler| {
                            let sampler = sampler_pool
                                .get(sampler.0)
                                .expect("trying to set an invalid immutable sampler");

                            // We need to make sure we include immutable samplers in the hash calculation.
                            hasher.update(&sampler.0.as_raw().to_le_bytes());

                            sampler.0
                        }),
                    );

                    // This pointer can safely escape the block as its lifetime is bound to the arena.
                    immutable_samplers.as_ptr()
                } else {
                    std::ptr::null()
                };

                vk::DescriptorSetLayoutBinding {
                    binding: if bind_desc.slot != !0 {
                        bind_desc.slot
                    } else {
                        i as u32
                    },
                    descriptor_type: vulkan_descriptor_type(bind_desc.binding_type),
                    descriptor_count: bind_desc.count,
                    stage_flags: vulkan_shader_stage_flags(bind_desc.stages),
                    immutable_samplers,
                }
            }));
        let create_info = &vk::DescriptorSetLayoutCreateInfo {
            bindings: layout_bindings.into(),
            ..default()
        };
        let mut descriptor_set_layout = vk::DescriptorSetLayout::null();
        vk_check!(unsafe {
            self.device_fn.create_descriptor_set_layout(
                self.device,
                create_info,
                None,
                &mut descriptor_set_layout,
            )
        });

        let hash = hasher.finalize();
        let bind_group_layout = self
            .bind_group_layout_pool
            .lock()
            .insert(VulkanBindGroupLayout {
                hash,
                descriptor_set_layout,
            });

        BindGroupLayout(bind_group_layout)
    }

    fn create_graphics_pipeline(&self, pipeline_desc: &GraphicsPipelineDesc) -> Pipeline {
        let pipeline_layout = self.cache_pipeline_layout(&pipeline_desc.layout);

        let arena = HybridArena::<1024>::new();

        let vertex_module = vulkan_shader_module(
            &self.device_fn,
            self.device,
            pipeline_desc.vertex_shader.code,
        );
        let fragment_module = vulkan_shader_module(
            &self.device_fn,
            self.device,
            pipeline_desc.fragment_shader.code,
        );

        assert!(
            !(pipeline_desc.vertex_shader.required_subgroup_size.is_some()
                || pipeline_desc
                    .fragment_shader
                    .required_subgroup_size
                    .is_some()
                || pipeline_desc.vertex_shader.allow_varying_subgroup_size
                || pipeline_desc.fragment_shader.allow_varying_subgroup_size),
            "subgroup size control features not implemented for graphics shader stages"
        );

        let stages = &[
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::VERTEX,
                name: pipeline_desc.vertex_shader.entry.as_ptr(),
                module: vertex_module,
                ..default()
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::FRAGMENT,
                name: pipeline_desc.fragment_shader.entry.as_ptr(),
                module: fragment_module,
                ..default()
            },
        ];

        let topology = vulkan_primitive_topology(pipeline_desc.topology);
        let primitive_restart_enable = vulkan_bool32(pipeline_desc.primitive_restart);
        let polygon_mode = vulkan_polygon_mode(pipeline_desc.polygon_mode);
        let cull_mode = vulkan_cull_mode(pipeline_desc.culling_mode);
        let front_face = vulkan_front_face(pipeline_desc.front_face);
        let (
            depth_bias_enable,
            depth_bias_constant_factor,
            depth_bias_clamp,
            depth_bias_slope_factor,
        ) = if let Some(depth_bias) = &pipeline_desc.depth_bias {
            (
                vk::Bool32::True,
                depth_bias.constant_factor,
                depth_bias.clamp,
                depth_bias.slope_factor,
            )
        } else {
            (vk::Bool32::False, 0.0, 0.0, 0.0)
        };
        let depth_compare_op = vulkan_compare_op(pipeline_desc.depth_compare_op);
        let depth_test_enable = vulkan_bool32(pipeline_desc.depth_test_enable);
        let depth_write_enable = vulkan_bool32(pipeline_desc.depth_write_enable);
        let stencil_test_enable = vulkan_bool32(pipeline_desc.stencil_test_enable);
        let back = vulkan_stencil_op_state(pipeline_desc.stencil_back);
        let front = vulkan_stencil_op_state(pipeline_desc.stencil_front);

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            topology,
            primitive_restart_enable,
            ..default()
        };
        let viewport_state = vk::PipelineViewportStateCreateInfo::default();
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo {
            polygon_mode,
            cull_mode,
            front_face,
            line_width: 1.0,
            depth_bias_enable,
            depth_bias_constant_factor,
            depth_bias_clamp,
            depth_bias_slope_factor,
            ..default()
        };
        let multisample_state = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::SAMPLE_COUNT_1,
            ..default()
        };
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo {
            depth_compare_op,
            depth_test_enable,
            depth_write_enable,
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
            stencil_test_enable,
            back,
            front,
            ..default()
        };
        let color_blend_attachments = &[vulkan_blend_mode(pipeline_desc.blend_mode)];
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
        let color_attachment_formats = arena.alloc_slice_fill_iter(
            pipeline_desc
                .attachments
                .color_attachment_formats
                .iter()
                .copied()
                .map(vulkan_format),
        );
        let pipeline_rendering_create_info = vk::PipelineRenderingCreateInfo {
            view_mask: 0,
            color_attachment_formats: color_attachment_formats.into(),
            depth_attachment_format: pipeline_desc
                .attachments
                .depth_attachment_format
                .map_or(vk::Format::Undefined, vulkan_format),
            stencil_attachment_format: pipeline_desc
                .attachments
                .stencil_attachment_format
                .map_or(vk::Format::Undefined, vulkan_format),
            ..default()
        };

        let create_infos = &mut [vk::GraphicsPipelineCreateInfo {
            _next: &pipeline_rendering_create_info as *const vk::PipelineRenderingCreateInfo
                as *const _,
            stages: stages.into(),
            vertex_input_state: Some(&vertex_input_state),
            input_assembly_state: Some(&input_assembly_state),
            tessellation_state: None,
            viewport_state: Some(&viewport_state),
            rasterization_state: Some(&rasterization_state),
            multisample_state: Some(&multisample_state),
            depth_stencil_state: Some(&depth_stencil_state),
            color_blend_state: Some(&color_blend_state),
            dynamic_state: Some(&dynamic_state),
            layout: pipeline_layout.pipeline_layout,
            ..default()
        }];
        let mut pipelines = [vk::Pipeline::null()];
        vk_check!(unsafe {
            self.device_fn.create_graphics_pipelines(
                self.device,
                vk::PipelineCache::null(),
                create_infos,
                None,
                &mut pipelines,
            )
        });

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
            pipeline_layout,
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
        });

        Pipeline(handle)
    }

    fn create_compute_pipeline(&self, pipeline_desc: &ComputePipelineDesc) -> Pipeline {
        let arena = HybridArena::<1024>::new();

        let pipeline_layout = self.cache_pipeline_layout(&pipeline_desc.layout);

        let module = vulkan_shader_module(&self.device_fn, self.device, pipeline_desc.shader.code);

        let mut shader_stage_create_flags = default();

        if pipeline_desc.shader.require_full_subgroups {
            shader_stage_create_flags |= vk::PipelineShaderStageCreateFlags::REQUIRE_FULL_SUBGROUPS
        }

        if pipeline_desc.shader.allow_varying_subgroup_size {
            shader_stage_create_flags |=
                vk::PipelineShaderStageCreateFlags::ALLOW_VARYING_SUBGROUP_SIZE;
        }

        let specialization_info: Option<&vk::SpecializationInfo> =
            if !pipeline_desc.shader.spec_constants.is_empty() {
                let block_len = pipeline_desc
                    .shader
                    .spec_constants
                    .iter()
                    .map(|spec_constant| match spec_constant {
                        SpecConstant::Bool { id: _, value: _ }
                        | SpecConstant::U32 { id: _, value: _ }
                        | SpecConstant::I32 { id: _, value: _ }
                        | SpecConstant::F32 { id: _, value: _ } => 4,
                    })
                    .sum::<usize>();

                let block = arena.alloc_slice_fill_copy(block_len, 0u8);

                let mut offset = 0;
                let map_entries =
                    arena.alloc_slice_fill_iter(pipeline_desc.shader.spec_constants.iter().map(
                        |spec_constant| {
                            let constant_id;
                            let value_size;
                            match *spec_constant {
                                SpecConstant::Bool { id, value } => {
                                    constant_id = id;
                                    let value = if value {
                                        vk::Bool32::True
                                    } else {
                                        vk::Bool32::False
                                    } as u32;
                                    value_size = std::mem::size_of_val(&value);
                                    block[offset..offset + value_size]
                                        .copy_from_slice(&value.to_ne_bytes())
                                }
                                SpecConstant::U32 { id, value } => {
                                    constant_id = id;
                                    value_size = std::mem::size_of_val(&value);
                                    block[offset..offset + value_size]
                                        .copy_from_slice(&value.to_ne_bytes());
                                }
                                SpecConstant::I32 { id, value } => {
                                    constant_id = id;
                                    value_size = std::mem::size_of_val(&value);
                                    block[offset..offset + value_size]
                                        .copy_from_slice(&value.to_ne_bytes());
                                }
                                SpecConstant::F32 { id, value } => {
                                    constant_id = id;
                                    value_size = std::mem::size_of_val(&value);
                                    block[offset..offset + value_size]
                                        .copy_from_slice(&value.to_ne_bytes());
                                }
                            }

                            let map_entry = vk::SpecializationMapEntry {
                                constant_id,
                                offset: offset as u32,
                                size: value_size,
                            };

                            offset += value_size;

                            map_entry
                        },
                    ));

                Some(arena.alloc(vk::SpecializationInfo {
                    data: block.into(),
                    map_entries: map_entries.into(),
                }))
            } else {
                None
            };

        let compute_pipeline_create_info = arena.alloc(vk::ComputePipelineCreateInfo {
            layout: pipeline_layout.pipeline_layout,
            stage: vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::COMPUTE,
                name: pipeline_desc.shader.entry.as_ptr(),
                module,
                flags: shader_stage_create_flags,
                specialization_info,
                ..default()
            },
            ..default()
        });

        if let Some(required_subgroup_size) = pipeline_desc.shader.required_subgroup_size {
            assert!(
                self.physical_device_properties
                    .required_subgroup_size_stages()
                    .contains(vk::ShaderStageFlags::COMPUTE)
            );
            assert!(
                required_subgroup_size >= self.physical_device_properties.min_subgroup_size()
                    && required_subgroup_size
                        <= self.physical_device_properties.max_subgroup_size()
            );

            let shader_stage_required_subgroup_size_create_info =
                arena.alloc(vk::PipelineShaderStageRequiredSubgroupSizeCreateInfo {
                    required_subgroup_size,
                    ..default()
                });

            // SAFETY: Both are arena allocations and therefore have identical lifetimes.
            compute_pipeline_create_info.stage._next =
                shader_stage_required_subgroup_size_create_info as *const _ as *const _;
        }

        let mut pipelines = [vk::Pipeline::null()];
        vk_check!(unsafe {
            self.device_fn.create_compute_pipelines(
                self.device,
                vk::PipelineCache::null(),
                std::slice::from_ref(compute_pipeline_create_info),
                None,
                &mut pipelines,
            )
        });

        unsafe {
            self.device_fn
                .destroy_shader_module(self.device, module, None)
        };

        let handle = self.pipeline_pool.lock().insert(VulkanPipeline {
            pipeline: pipelines[0],
            pipeline_layout,
            pipeline_bind_point: vk::PipelineBindPoint::Compute,
        });

        Pipeline(handle)
    }

    fn debug_name_buffer(&self, buffer: BufferArg, name: &str) {
        #[cfg(feature = "debug_markers")]
        if let Some(debug_utils_fn) = &self.debug_utils_fn {
            let buffer = match buffer {
                BufferArg::Unmanaged(buffer) => buffer,
                BufferArg::Persistent(buffer) => buffer.buffer,
                BufferArg::Transient(_) => return,
            };

            let buffer_handle;
            {
                let buffer_pool = self.buffer_pool.lock();
                let Some(buffer) = buffer_pool.get(buffer.0) else {
                    return;
                };

                buffer_handle = buffer.buffer.as_raw();
            }

            let arena = HybridArena::<512>::new();
            let object_name = arena.alloc_cstr_from_str(name);

            let name_info = vk::DebugUtilsObjectNameInfoExt {
                object_type: vk::ObjectType::Buffer,
                object_handle: buffer_handle,
                object_name: object_name.as_ptr(),
                ..default()
            };

            unsafe { debug_utils_fn.set_debug_utils_object_name_ext(self.device, &name_info) }
        }
    }

    fn debug_name_image(&self, image: Image, name: &str) {
        #[cfg(feature = "debug_markers")]
        if let Some(debug_utils_fn) = &self.debug_utils_fn {
            let image_handle;
            let image_view_handle;
            {
                let image_pool = self.image_pool.lock();
                let Some(image_holder) = image_pool.get(image.0) else {
                    return;
                };

                match image_holder {
                    VulkanImageHolder::Unique(unique) => {
                        image_handle = unique.image.image.as_raw();
                        image_view_handle = unique.view.as_raw();
                    }
                    VulkanImageHolder::Shared(shared) => {
                        image_handle = 0;
                        image_view_handle = shared.view.as_raw();
                    }
                    VulkanImageHolder::Swapchain(_) => return,
                }
            }
            let arena = HybridArena::<512>::new();
            let object_name = arena.alloc_cstr_from_str(name);

            if image_handle != 0 {
                let image_name_info = vk::DebugUtilsObjectNameInfoExt {
                    object_type: vk::ObjectType::Image,
                    object_handle: image_handle,
                    object_name: object_name.as_ptr(),
                    ..default()
                };
                unsafe {
                    debug_utils_fn.set_debug_utils_object_name_ext(self.device, &image_name_info)
                }
            }

            let image_view_name_info = vk::DebugUtilsObjectNameInfoExt {
                object_type: vk::ObjectType::ImageView,
                object_handle: image_view_handle,
                object_name: object_name.as_ptr(),
                ..default()
            };
            unsafe {
                debug_utils_fn.set_debug_utils_object_name_ext(self.device, &image_view_name_info)
            }
        }
    }

    fn debug_name_bind_group_layout(&self, bind_group_layout: BindGroupLayout, name: &str) {
        #[cfg(feature = "debug_markers")]
        if let Some(debug_utils_fn) = &self.debug_utils_fn {
            let descriptor_set_layout;
            {
                let bind_group_layout_pool = self.bind_group_layout_pool.lock();
                let Some(bind_group_layout) = bind_group_layout_pool.get(bind_group_layout.0)
                else {
                    return;
                };

                descriptor_set_layout = bind_group_layout.descriptor_set_layout;
            }

            let arena = HybridArena::<512>::new();
            let object_name = arena.alloc_cstr_from_str(name);

            let image_name_info = vk::DebugUtilsObjectNameInfoExt {
                object_type: vk::ObjectType::DescriptorSetLayout,
                object_handle: descriptor_set_layout.as_raw(),
                object_name: object_name.as_ptr(),
                ..default()
            };
            unsafe { debug_utils_fn.set_debug_utils_object_name_ext(self.device, &image_name_info) }
        }
    }

    fn debug_name_pipeline(&self, pipeline: Pipeline, name: &str) {
        #[cfg(feature = "debug_markers")]
        if let Some(debug_utils_fn) = &self.debug_utils_fn {
            let pipeline_handle;
            let pipeline_layout_handle;
            {
                let pipeline_pool = self.pipeline_pool.lock();
                let Some(pipeline) = pipeline_pool.get(pipeline.0) else {
                    return;
                };

                pipeline_handle = pipeline.pipeline;
                pipeline_layout_handle = pipeline.pipeline_layout.pipeline_layout;
            }

            let arena = HybridArena::<512>::new();
            let object_name = arena.alloc_cstr_from_str(name);

            let image_name_info = vk::DebugUtilsObjectNameInfoExt {
                object_type: vk::ObjectType::Pipeline,
                object_handle: pipeline_handle.as_raw(),
                object_name: object_name.as_ptr(),
                ..default()
            };
            unsafe { debug_utils_fn.set_debug_utils_object_name_ext(self.device, &image_name_info) }

            let image_view_name_info = vk::DebugUtilsObjectNameInfoExt {
                object_type: vk::ObjectType::PipelineLayout,
                object_handle: pipeline_layout_handle.as_raw(),
                object_name: object_name.as_ptr(),
                ..default()
            };
            unsafe {
                debug_utils_fn.set_debug_utils_object_name_ext(self.device, &image_view_name_info)
            }
        }
    }

    fn destroy_buffer(&self, frame: &Frame, buffer: Buffer) {
        if let Some(buffer) = self.buffer_pool.lock().remove(buffer.0) {
            assert_eq!(
                buffer.map_count, 0,
                "destroying a buffer that is still mapped"
            );
            let frame = self.frame(frame);
            frame.destroyed_buffers.lock().push_back(buffer.buffer);
            frame.destroyed_allocations.lock().push_back(buffer.memory);
        }
    }

    fn destroy_persistent_buffer(&self, frame: &Frame, buffer: PersistentBuffer) {
        unsafe { self.unmap_buffer(buffer.buffer) }
        self.destroy_buffer(frame, buffer.buffer)
    }

    fn destroy_image(&self, frame: &Frame, image: Image) {
        if let Some(image_holder) = self.image_pool.lock().remove(image.0) {
            let frame = self.frame(frame);

            match image_holder {
                // The image is unique, we've never allocated a reference counted object for it.
                VulkanImageHolder::Unique(image) => {
                    frame.destroyed_image_views.lock().push_back(image.view);
                    frame.destroyed_images.lock().push_back(image.image.image);
                    frame
                        .destroyed_allocations
                        .lock()
                        .push_back(image.image.memory);
                }
                // The image was at one point shared, we may or may not have the last reference.
                VulkanImageHolder::Shared(image) => {
                    frame.destroyed_image_views.lock().push_back(image.view);
                    // If we had the last reference we need to destroy the image and memory too
                    if let manual_arc::Release::Unique(image) = image.image.release() {
                        frame.destroyed_images.lock().push_back(image.image);
                        frame.destroyed_allocations.lock().push_back(image.memory);
                    }
                }
                VulkanImageHolder::Swapchain(_) => {
                    panic!("cannot directly destroy swapchain images")
                }
            }
        }
    }

    fn destroy_sampler(&self, frame: &Frame, sampler: Sampler) {
        if let Some(sampler) = self.sampler_pool.lock().remove(sampler.0) {
            self.frame(frame)
                .destroyed_samplers
                .lock()
                .push_back(sampler.0)
        }
    }

    fn destroy_bind_group_layout(&self, frame: &Frame, bind_group_layout: BindGroupLayout) {
        if let Some(bind_group_layout) = self
            .bind_group_layout_pool
            .lock()
            .remove(bind_group_layout.0)
        {
            self.frame(frame)
                .destroyed_descriptor_set_layouts
                .lock()
                .push_back(bind_group_layout.descriptor_set_layout)
        }
    }

    fn destroy_pipeline(&self, frame: &Frame, pipeline: Pipeline) {
        if let Some(pipeline) = self.pipeline_pool.lock().remove(pipeline.0) {
            let frame = self.frame(frame);
            frame
                .destroyed_pipelines
                .lock()
                .push_back(pipeline.pipeline);
        }
    }

    fn acquire_swapchain(
        &self,
        frame: &Frame,
        window: &dyn AsRawWindow,
        width: u32,
        height: u32,
        configurator: &mut dyn SwapchainConfigurator,
    ) -> Result<SwapchainImage, SwapchainOutOfDateError> {
        self.acquire_swapchain(frame, window, width, height, configurator)
    }

    fn destroy_swapchain(&self, window: &dyn AsRawWindow) {
        self.destroy_swapchain(window)
    }

    unsafe fn map_buffer(&self, buffer: Buffer) -> *mut u8 {
        let mut buffer_pool = self.buffer_pool.lock();
        let buffer = buffer_pool.get_mut(buffer.0).unwrap();
        buffer.map_count += 1;
        buffer.memory.mapped_ptr()
    }

    unsafe fn unmap_buffer(&self, buffer: Buffer) {
        let mut buffer_pool = self.buffer_pool.lock();
        let buffer = buffer_pool.get_mut(buffer.0).unwrap();
        assert!(buffer.map_count > 0);
        buffer.map_count -= 1;
    }

    fn request_transient_buffer<'a>(
        &self,
        frame: &'a Frame,
        thread_token: &'a ThreadToken,
        usage: BufferUsageFlags,
        size: usize,
    ) -> TransientBuffer<'a> {
        self.request_transient_buffer(frame, thread_token, usage, size as u64)
    }

    fn request_transient_bind_group<'a>(
        &self,
        frame: &'a Frame<'a>,
        thread_token: &'a ThreadToken,
        layout: BindGroupLayout,
        bindings: &[Bind],
    ) -> TransientBindGroup<'a> {
        let arena = HybridArena::<4096>::new();

        let descriptor_set_layout = self
            .bind_group_layout_pool
            .lock()
            .get(layout.0)
            .unwrap()
            .descriptor_set_layout;

        let frame = self.frame(frame);
        let per_thread = frame.per_thread.get(thread_token);

        let mut descriptor_pool = per_thread.descriptor_pool.get();
        let mut allocated_pool = false;
        let descriptor_set = loop {
            if descriptor_pool.is_null() {
                // Need to fetch a new descriptor pool
                descriptor_pool = self.request_descriptor_pool();
                per_thread.descriptor_pool.set(descriptor_pool);
                frame.recycle_descriptor_pool(descriptor_pool);
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
            TypedBind::SampledImage(images) => {
                let image_infos_iter = images.iter().map(|(image_layout, image)| {
                    let image_view = self.image_pool.lock().get(image.0).unwrap().image_view();
                    vk::DescriptorImageInfo {
                        image_layout: match image_layout {
                            ImageLayout::Optimal => vk::ImageLayout::ReadOnlyOptimal,
                            ImageLayout::General => vk::ImageLayout::General,
                        },
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
            TypedBind::StorageImage(images) => {
                let image_infos_iter = images.iter().map(|(image_layout, image)| {
                    let image_view = self.image_pool.lock().get(image.0).unwrap().image_view();
                    vk::DescriptorImageInfo {
                        image_layout: match image_layout {
                            ImageLayout::Optimal => vk::ImageLayout::ReadOnlyOptimal,
                            ImageLayout::General => vk::ImageLayout::General,
                        },
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
                    descriptor_type: vk::DescriptorType::StorageImage,
                    image_info: image_infos.as_ptr(),
                    ..default()
                }
            }
            TypedBind::UniformBuffer(buffers) => {
                let buffer_infos_iter = buffers.iter().map(|buffer_arg| {
                    let (buffer, offset, range) = self.unwrap_buffer_arg(buffer_arg);
                    vk::DescriptorBufferInfo {
                        buffer,
                        offset,
                        range,
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
            TypedBind::StorageBuffer(buffers) => {
                let buffer_infos_iter = buffers.iter().map(|buffer_arg| {
                    let (buffer, offset, range) = self.unwrap_buffer_arg(buffer_arg);
                    vk::DescriptorBufferInfo {
                        buffer,
                        offset,
                        range,
                    }
                });
                let buffer_infos = arena.alloc_slice_fill_iter(buffer_infos_iter);
                vk::WriteDescriptorSet {
                    dst_set: descriptor_set,
                    dst_binding: bind.binding,
                    dst_array_element: bind.array_element,
                    descriptor_count: buffer_infos.len() as u32,
                    descriptor_type: vk::DescriptorType::StorageBuffer,
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

        TransientBindGroup {
            bind_group: descriptor_set.as_raw(),
            phantom: PhantomData,
        }
    }

    fn request_cmd_encoder<'a, 'thread>(
        &self,
        frame: &'a Frame,
        thread_token: &'a ThreadToken,
    ) -> CmdEncoder<'a> {
        let frame = self.frame(frame);
        let per_thread = frame.per_thread.get(thread_token);
        let mut cmd_buffer_pool = per_thread.cmd_buffer_pool.borrow_mut();

        // We have consumed all available command buffers, need to allocate a new one.
        if cmd_buffer_pool.next_free_index >= cmd_buffer_pool.command_buffers.len() {
            let mut cmd_buffers = [vk::CommandBuffer::null(); 4];
            let allocate_info = vk::CommandBufferAllocateInfo {
                command_pool: cmd_buffer_pool.command_pool,
                level: vk::CommandBufferLevel::Primary,
                command_buffer_count: cmd_buffers.len() as u32,
                ..default()
            };
            vk_check!(unsafe {
                self.device_fn.allocate_command_buffers(
                    self.device,
                    &allocate_info,
                    cmd_buffers.as_mut_ptr(),
                )
            });
            cmd_buffer_pool.command_buffers.extend(cmd_buffers.iter());
        }

        let index = cmd_buffer_pool.next_free_index;
        cmd_buffer_pool.next_free_index += 1;
        let command_buffer = cmd_buffer_pool.command_buffers[index];

        vk_check!(unsafe {
            self.device_fn.begin_command_buffer(
                command_buffer,
                &vk::CommandBufferBeginInfo {
                    flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                    ..default()
                },
            )
        });

        let vulkan_cmd_encoder = per_thread.arena.alloc(VulkanCmdEncoder {
            command_buffer,
            ..default()
        });

        CmdEncoder {
            cmd_encoder_addr: vulkan_cmd_encoder as *mut _ as usize,
            phantom: PhantomData,
            phantom_unsend: PhantomUnsend {},
        }
    }

    fn cmd_insert_debug_marker(
        &self,
        cmd_encoder: &mut CmdEncoder,
        label_name: &str,
        color: [f32; 4],
    ) {
        #[cfg(feature = "debug_markers")]
        if let Some(debug_utils_fn) = &self.debug_utils_fn {
            let arena = HybridArena::<256>::new();
            let label_name = arena.alloc_cstr_from_str(label_name);

            let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
            let label_info = vk::DebugUtilsLabelExt {
                label_name: label_name.as_ptr(),
                color,
                ..default()
            };
            unsafe {
                debug_utils_fn.cmd_insert_debug_utils_label_ext(command_buffer, &label_info);
            }
        }
    }

    fn cmd_begin_debug_marker(
        &self,
        cmd_encoder: &mut CmdEncoder,
        label_name: &str,
        color: [f32; 4],
    ) {
        #[cfg(feature = "debug_markers")]
        if let Some(debug_utils_fn) = &self.debug_utils_fn {
            let arena = HybridArena::<256>::new();
            let label_name = arena.alloc_cstr_from_str(label_name);

            let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
            let label_info = vk::DebugUtilsLabelExt {
                label_name: label_name.as_ptr(),
                color,
                ..default()
            };
            unsafe {
                debug_utils_fn.cmd_begin_debug_utils_label_ext(command_buffer, &label_info);
            }
        }
    }

    fn cmd_end_debug_marker(&self, cmd_encoder: &mut CmdEncoder) {
        #[cfg(feature = "debug_markers")]
        if let Some(debug_utils_fn) = &self.debug_utils_fn {
            let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
            unsafe { debug_utils_fn.cmd_end_debug_utils_label_ext(command_buffer) }
        }
    }

    fn cmd_set_bind_group(
        &self,
        cmd_encoder: &mut CmdEncoder,
        bind_group_index: u32,
        bind_group: &TransientBindGroup,
    ) {
        let cmd_encoder = self.cmd_encoder_mut(cmd_encoder);

        let bound_pipeline = cmd_encoder
            .bound_pipeline
            .as_ref()
            .expect("cannot set bind group without a pipeline bound");

        let command_buffer = cmd_encoder.command_buffer;
        let descriptor_set = vk::DescriptorSet::from_raw(bind_group.bind_group);

        unsafe {
            self.device_fn.cmd_bind_descriptor_sets(
                command_buffer,
                bound_pipeline.pipeline_bind_point,
                bound_pipeline.pipeline_layout,
                bind_group_index,
                &[descriptor_set],
                &[],
            )
        }
    }

    fn cmd_set_index_buffer(
        &self,
        cmd_encoder: &mut CmdEncoder,
        buffer: BufferArg,
        offset: u64,
        index_type: IndexType,
    ) {
        let (buffer, base_offset, _range) = self.unwrap_buffer_arg(&buffer);

        let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
        let index_type = vulkan_index_type(index_type);
        unsafe {
            self.device_fn.cmd_bind_index_buffer(
                command_buffer,
                buffer,
                offset + base_offset,
                index_type,
            )
        }
    }

    fn cmd_compute_touch_swapchain(&self, cmd_encoder: &mut CmdEncoder, image: Image) {
        let cmd_encoder = self.cmd_encoder_mut(cmd_encoder);

        match self.image_pool.lock().get(image.0) {
            Some(VulkanImageHolder::Swapchain(image)) => {
                assert!(
                    !cmd_encoder.swapchains_touched.contains_key(&image.surface),
                    "swapchain attached multiple times in a command buffer"
                );
                cmd_encoder.swapchains_touched.insert(
                    image.surface,
                    VulkanTouchedSwapchain {
                        image: image.image,
                        layout: vk::ImageLayout::General,
                        access_mask: vk::AccessFlags2::SHADER_STORAGE_WRITE,
                        stage_mask: vk::PipelineStageFlags2::COMPUTE_SHADER,
                    },
                );

                // Transition swapchain image to shader storage write
                let image_memory_barriers = &[vk::ImageMemoryBarrier2 {
                    src_stage_mask: vk::PipelineStageFlags2::COMPUTE_SHADER,
                    src_access_mask: vk::AccessFlags2::NONE,
                    dst_stage_mask: vk::PipelineStageFlags2::COMPUTE_SHADER,
                    dst_access_mask: vk::AccessFlags2::SHADER_STORAGE_WRITE,
                    old_layout: vk::ImageLayout::Undefined,
                    new_layout: vk::ImageLayout::General,
                    image: image.image,
                    ..default()
                }];

                let dependency_info = vk::DependencyInfo {
                    image_memory_barriers: image_memory_barriers.into(),
                    ..default()
                };

                unsafe {
                    self.device_fn
                        .cmd_pipeline_barrier2(cmd_encoder.command_buffer, &dependency_info)
                };
            }
            _ => panic!(),
        }
    }

    fn cmd_set_pipeline(&self, cmd_encoder: &mut CmdEncoder, pipeline: Pipeline) {
        let cmd_encoder = self.cmd_encoder_mut(cmd_encoder);

        let vk_pipeline;
        let pipeline_layout;
        let pipeline_bind_point;
        {
            let pipeline_pool = self.pipeline_pool.lock();
            let pipeline = pipeline_pool.get(pipeline.0).unwrap();
            vk_pipeline = pipeline.pipeline;
            pipeline_layout = pipeline.pipeline_layout.pipeline_layout;
            pipeline_bind_point = pipeline.pipeline_bind_point;
        }

        cmd_encoder.bound_pipeline = Some(VulkanBoundPipeline {
            pipeline_layout,
            pipeline_bind_point,
        });

        let command_buffer = cmd_encoder.command_buffer;

        unsafe {
            self.device_fn
                .cmd_bind_pipeline(command_buffer, pipeline_bind_point, vk_pipeline)
        };
    }

    fn cmd_set_viewports(&self, cmd_encoder: &mut CmdEncoder, viewports: &[crate::Viewport]) {
        let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
        unsafe {
            self.device_fn.cmd_set_viewport_with_count(
                command_buffer,
                std::mem::transmute::<&[crate::Viewport], &[vk::Viewport]>(viewports), // yolo
            );
        }
    }

    fn cmd_set_scissors(&self, cmd_encoder: &mut CmdEncoder, scissors: &[crate::Scissor]) {
        let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
        unsafe {
            self.device_fn.cmd_set_scissor_with_count(
                command_buffer,
                std::mem::transmute::<&[crate::Scissor], &[vk::Rect2d]>(scissors), // yolo
            );
        }
    }

    fn cmd_barrier(
        &self,
        cmd_encoder: &mut CmdEncoder,
        global_barrier: Option<&GlobalBarrier>,
        image_barriers: &[ImageBarrier],
    ) {
        let arena = HybridArena::<4096>::new();

        let memory_barriers = arena.alloc_slice_fill_iter(
            global_barrier
                .iter()
                .map(|global_barrier| vulkan_memory_barrier(global_barrier)),
        );

        let image_memory_barriers =
            arena.alloc_slice_fill_iter(image_barriers.iter().map(|image_barrier| {
                let image = self
                    .image_pool
                    .lock()
                    .get(image_barrier.image.0)
                    .expect("invalid image handle")
                    .image();
                let subresource_range = vulkan_subresource_range(&image_barrier.subresource_range);
                vulkan_image_memory_barrier(image_barrier, image, subresource_range)
            }));

        let cmd_encoder = self.cmd_encoder_mut(cmd_encoder);

        #[cfg(debug_assertions)]
        debug_assert!(!cmd_encoder.in_render_pass);

        unsafe {
            self.device_fn.cmd_pipeline_barrier2(
                cmd_encoder.command_buffer,
                &vk::DependencyInfo {
                    memory_barriers: memory_barriers.into(),

                    image_memory_barriers: image_memory_barriers.into(),
                    ..default()
                },
            )
        }
    }

    unsafe fn cmd_push_constants(
        &self,
        cmd_encoder: &mut CmdEncoder,
        stage_flags: ShaderStageFlags,
        offset: u32,
        ptr: *const u8,
        len: usize,
    ) {
        unsafe {
            let len = u32::try_from(len).unwrap();

            let cmd_encoder = self.cmd_encoder_mut(cmd_encoder);
            let command_buffer = cmd_encoder.command_buffer;

            let VulkanBoundPipeline {
                pipeline_layout,
                pipeline_bind_point: _,
            } = cmd_encoder
                .bound_pipeline
                .as_ref()
                .expect("cannot push constants without a pipeline bound")
                .clone();

            let stage_flags = vulkan_shader_stage_flags(stage_flags);
            self.device_fn.cmd_push_constants(
                command_buffer,
                pipeline_layout,
                stage_flags,
                offset,
                len,
                ptr as *const std::ffi::c_void,
            )
        }
    }

    fn cmd_copy_buffer_to_image(
        &self,
        cmd_encoder: &mut CmdEncoder,
        src_buffer: BufferArg,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        copies: &[BufferImageCopy],
    ) {
        let arena = HybridArena::<4096>::new();

        let (src_buffer, base_offset, _range) = self.unwrap_buffer_arg(&src_buffer);

        let regions = arena.alloc_slice_fill_iter(copies.iter().map(|copy| vk::BufferImageCopy {
            buffer_offset: copy.buffer_offset + base_offset,
            buffer_row_length: copy.buffer_row_length,
            buffer_image_height: copy.buffer_image_height,
            image_subresource: vulkan_subresource_layers(&copy.image_subresource),
            image_offset: copy.image_offset.into(),
            image_extent: copy.image_extent.into(),
        }));

        let dst_image = self
            .image_pool
            .lock()
            .get(dst_image.0)
            .expect("invalid image handle")
            .image();

        let dst_image_layout = match dst_image_layout {
            ImageLayout::Optimal => vk::ImageLayout::TransferDstOptimal,
            ImageLayout::General => vk::ImageLayout::General,
        };

        let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
        unsafe {
            self.device_fn.cmd_copy_buffer_to_image(
                command_buffer,
                src_buffer,
                dst_image,
                dst_image_layout,
                regions,
            )
        }
    }

    fn cmd_blit_image(
        &self,
        cmd_encoder: &mut CmdEncoder,
        src_image: Image,
        src_image_layout: ImageLayout,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        regions: &[ImageBlit],
    ) {
        let arena = HybridArena::<4096>::new();

        let regions = arena.alloc_slice_fill_iter(regions.iter().map(|blit| vk::ImageBlit {
            src_subresource: vulkan_subresource_layers(&blit.src_subresource),
            src_offsets: [blit.src_offset_min.into(), blit.src_offset_max.into()],
            dst_subresource: vulkan_subresource_layers(&blit.dst_subresource),
            dst_offsets: [blit.dst_offset_min.into(), blit.dst_offset_max.into()],
        }));

        let src_image = self
            .image_pool
            .lock()
            .get(src_image.0)
            .expect("invalid src image handle")
            .image();

        let src_image_layout = match src_image_layout {
            ImageLayout::Optimal => vk::ImageLayout::TransferSrcOptimal,
            ImageLayout::General => vk::ImageLayout::General,
        };

        let dst_image = self
            .image_pool
            .lock()
            .get(dst_image.0)
            .expect("invalid dst image handle")
            .image();

        let dst_image_layout = match dst_image_layout {
            ImageLayout::Optimal => vk::ImageLayout::TransferDstOptimal,
            ImageLayout::General => vk::ImageLayout::General,
        };

        let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
        unsafe {
            self.device_fn.cmd_blit_image(
                command_buffer,
                src_image,
                src_image_layout,
                dst_image,
                dst_image_layout,
                regions,
                vk::Filter::Linear,
            );
        }
    }

    fn cmd_begin_rendering(&self, cmd_encoder: &mut CmdEncoder, desc: &crate::RenderingDesc) {
        let arena = HybridArena::<1024>::new();
        let cmd_encoder = self.cmd_encoder_mut(cmd_encoder);

        #[cfg(debug_assertions)]
        {
            assert!(!cmd_encoder.in_render_pass);
            cmd_encoder.in_render_pass = true;
        }

        let color_attachments =
            arena.alloc_slice_fill_iter(desc.color_attachments.iter().map(|attachment| {
                let image_view = match self.image_pool.lock().get(attachment.image.0).unwrap() {
                    VulkanImageHolder::Unique(image) => image.view,
                    VulkanImageHolder::Shared(image) => image.view,
                    VulkanImageHolder::Swapchain(image) => {
                        assert!(
                            !cmd_encoder.swapchains_touched.contains_key(&image.surface),
                            "swapchain attached multiple times in a command buffer"
                        );
                        cmd_encoder.swapchains_touched.insert(
                            image.surface,
                            VulkanTouchedSwapchain {
                                image: image.image,
                                layout: vk::ImageLayout::AttachmentOptimal,
                                access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                                stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                            },
                        );

                        // transition swapchain image to attachment optimal
                        let image_memory_barriers = &[vk::ImageMemoryBarrier2 {
                            src_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                            src_access_mask: vk::AccessFlags2::NONE,
                            dst_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                            dst_access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                            old_layout: vk::ImageLayout::Undefined,
                            new_layout: vk::ImageLayout::AttachmentOptimal,
                            image: image.image,
                            ..default()
                        }];

                        let dependency_info = vk::DependencyInfo {
                            image_memory_barriers: image_memory_barriers.into(),
                            ..default()
                        };

                        unsafe {
                            self.device_fn
                                .cmd_pipeline_barrier2(cmd_encoder.command_buffer, &dependency_info)
                        };

                        image.view
                    }
                };

                let (load_op, clear_value) = vulkan_load_op(attachment.load_op);
                let store_op = vulkan_store_op(attachment.store_op);

                vk::RenderingAttachmentInfo {
                    image_view,
                    image_layout: vk::ImageLayout::ColorAttachmentOptimal,
                    load_op,
                    store_op,
                    clear_value,
                    ..default()
                }
            }));

        let depth_attachment = desc.depth_attachment.as_ref().map(|attachment| {
            let image_view = match self.image_pool.lock().get(attachment.image.0).unwrap() {
                VulkanImageHolder::Unique(image) => image.view,
                VulkanImageHolder::Shared(image) => image.view,
                VulkanImageHolder::Swapchain(_) => panic!(),
            };

            let (load_op, clear_value) = vulkan_load_op(attachment.load_op);
            let store_op = vulkan_store_op(attachment.store_op);

            vk::RenderingAttachmentInfo {
                image_view,
                image_layout: vk::ImageLayout::DepthAttachmentOptimal,
                load_op,
                store_op,
                clear_value,
                ..default()
            }
        });

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
            color_attachments: color_attachments.into(),
            depth_attachment: depth_attachment.as_ref(),
            stencil_attachment: None,
            ..default()
        };
        unsafe {
            self.device_fn
                .cmd_begin_rendering(cmd_encoder.command_buffer, &rendering_info)
        }
    }

    fn cmd_end_rendering(&self, cmd_encoder: &mut CmdEncoder) {
        let cmd_encoder = self.cmd_encoder_mut(cmd_encoder);

        #[cfg(debug_assertions)]
        {
            assert!(cmd_encoder.in_render_pass);
            cmd_encoder.in_render_pass = false;
        }

        unsafe { self.device_fn.cmd_end_rendering(cmd_encoder.command_buffer) }
    }

    fn cmd_draw(
        &self,
        cmd_encoder: &mut CmdEncoder,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
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

    fn cmd_draw_indexed(
        &self,
        cmd_encoder: &mut CmdEncoder,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
        unsafe {
            self.device_fn.cmd_draw_indexed(
                command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            )
        }
    }

    fn cmd_dispatch(
        &self,
        cmd_encoder: &mut CmdEncoder,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
        unsafe {
            self.device_fn
                .cmd_dispatch(command_buffer, group_count_x, group_count_y, group_count_z)
        }
    }

    fn cmd_dispatch_indirect(&self, cmd_encoder: &mut CmdEncoder, buffer: BufferArg, offset: u64) {
        let (buffer, base_offset, _range) = self.unwrap_buffer_arg(&buffer);

        let command_buffer = self.cmd_encoder_mut(cmd_encoder).command_buffer;
        unsafe {
            self.device_fn
                .cmd_dispatch_indirect(command_buffer, buffer, base_offset + offset);
        }
    }

    fn submit(&self, frame: &Frame, mut cmd_encoder: CmdEncoder) {
        let fence = self.universal_queue_fence.fetch_add(1, Ordering::SeqCst) + 1;

        let frame = self.frame(frame);
        frame.universal_queue_fence.store(fence, Ordering::Relaxed);

        let cmd_encoder = self.cmd_encoder_mut(&mut cmd_encoder);

        #[cfg(debug_assertions)]
        debug_assert!(!cmd_encoder.in_render_pass);

        for &VulkanTouchedSwapchain {
            image,
            layout,
            access_mask,
            stage_mask,
        } in cmd_encoder.swapchains_touched.values()
        {
            // transition swapchain image to present src
            let image_memory_barriers = &[vk::ImageMemoryBarrier2 {
                src_stage_mask: stage_mask,
                src_access_mask: access_mask,
                // According to the vulkan documentation, this should be `vk::PipelineStageFlags2::NONE`, however it
                // seems that is not true any longer.
                // see: <https://github.com/KhronosGroup/Vulkan-ValidationLayers/issues/6177#issuecomment-1693009636>
                dst_stage_mask: stage_mask,
                dst_access_mask: vk::AccessFlags2::NONE,
                old_layout: layout,
                new_layout: vk::ImageLayout::PresentSrcKhr,
                image,
                ..default()
            }];
            let dependency_info = vk::DependencyInfo {
                image_memory_barriers: image_memory_barriers.into(),
                ..default()
            };
            unsafe {
                self.device_fn
                    .cmd_pipeline_barrier2(cmd_encoder.command_buffer, &dependency_info)
            };
        }

        vk_check!(unsafe {
            self.device_fn
                .end_command_buffer(cmd_encoder.command_buffer)
        });

        let mut wait_semaphores = Vec::new();
        let mut signal_semaphores = Vec::new();

        if !cmd_encoder.swapchains_touched.is_empty() {
            for (
                surface,
                VulkanTouchedSwapchain {
                    image: _,
                    layout: _,
                    access_mask: _,
                    stage_mask,
                },
            ) in cmd_encoder.swapchains_touched.drain()
            {
                self.touch_swapchain(
                    frame,
                    surface,
                    stage_mask,
                    &mut wait_semaphores,
                    &mut signal_semaphores,
                );
            }
        }

        signal_semaphores.push(vk::SemaphoreSubmitInfo {
            semaphore: self.universal_queue_semaphore,
            semaphore_value: fence,
            stage_mask: vk::PipelineStageFlags2::COMPUTE_SHADER
                | vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            ..default()
        });

        let cmd_buffer_infos = &[vk::CommandBufferSubmitInfo {
            command_buffer: cmd_encoder.command_buffer,
            device_mask: 1,
            ..default()
        }];

        vk_check!(unsafe {
            self.device_fn.queue_submit2(
                self.universal_queue,
                &[vk::SubmitInfo2 {
                    wait_semaphore_infos: wait_semaphores.as_slice().into(),
                    command_buffer_infos: cmd_buffer_infos.into(),
                    signal_semaphore_infos: signal_semaphores.as_slice().into(),
                    ..default()
                }],
                vk::Fence::null(),
            )
        });
    }

    fn wait_idle(&self) {
        vk_check!(unsafe { self.device_fn.device_wait_idle(self.device) });
    }

    fn begin_frame(&self) -> Frame {
        let device_fn = &self.device_fn;
        let device = self.device;

        let mut frame = self.frame_counter.acquire(self as *const _ as usize);
        {
            let frame = self.frame_mut(&mut frame);

            {
                let semaphore_fences = &[frame
                    .universal_queue_fence
                    .load(std::sync::atomic::Ordering::Relaxed)];
                let semaphores = &[self.universal_queue_semaphore];
                let wait_info = vk::SemaphoreWaitInfo {
                    semaphores: (semaphores, semaphore_fences).into(),
                    ..default()
                };
                vk_check!(unsafe { device_fn.wait_semaphores(device, &wait_info, !0) });
            }

            for per_thread in frame.per_thread.slots_mut() {
                per_thread.descriptor_pool.set(vk::DescriptorPool::null());
                let cmd_buffer_pool = per_thread.cmd_buffer_pool.get_mut();
                if cmd_buffer_pool.next_free_index != 0 {
                    vk_check!(unsafe {
                        device_fn.reset_command_pool(
                            device,
                            cmd_buffer_pool.command_pool,
                            vk::CommandPoolResetFlags::default(),
                        )
                    });
                    cmd_buffer_pool.next_free_index = 0;
                }

                let transient_buffer_allocator = per_thread.transient_buffer_allocator.get_mut();
                if !transient_buffer_allocator.used_buffers.is_empty() {
                    self.recycled_transient_buffers
                        .lock()
                        .extend(transient_buffer_allocator.used_buffers.drain(..))
                }
                transient_buffer_allocator.reset();

                per_thread.arena.reset()
            }

            self.recycled_semaphores
                .lock()
                .extend(frame.recycled_semaphores.get_mut().drain(..));

            for descriptor_pool in frame.recycled_descriptor_pools.get_mut() {
                vk_check!(unsafe {
                    device_fn.reset_descriptor_pool(
                        device,
                        *descriptor_pool,
                        vk::DescriptorPoolResetFlags::default(),
                    )
                })
            }

            self.recycled_descriptor_pools
                .lock()
                .extend(frame.recycled_descriptor_pools.get_mut().drain(..));

            Self::destroy_deferred(device_fn, device, frame);

            self.wsi_begin_frame();

            self.allocator_begin_frame(frame);
        }

        frame
    }

    fn end_frame(&self, mut frame: Frame) {
        self.wsi_end_frame(self.frame_mut(&mut frame));
        self.frame_counter.release(frame);
    }

    fn get_buffer_address<'a>(&self, buffer: BufferArg<'a>) -> BufferAddress<'a> {
        let buffer = match buffer {
            BufferArg::Unmanaged(buffer) => buffer.0,
            BufferArg::Persistent(buffer) => buffer.buffer.0,
            BufferArg::Transient(buffer) => return buffer.address,
        };
        let buffer_pool = self.buffer_pool.lock();
        let buffer = buffer_pool.get(buffer).unwrap();
        BufferAddress {
            value: buffer.address,
            phantom: PhantomData,
        }
    }
}

impl VulkanDevice {
    fn request_transient_buffer<'a>(
        &self,
        frame: &'a Frame,
        thread_token: &'a ThreadToken,
        usage: BufferUsageFlags,
        size: u64,
    ) -> TransientBuffer<'a> {
        let frame = self.frame(frame);

        // If the requested size is too large, fall back to a regular allocation that we
        // queue for destruction right away.
        if size > VULKAN_CONSTANTS.transient_buffer_size {
            let queue_family_indices = &[self.universal_queue_family_index];
            let create_info = vk::BufferCreateInfo {
                size,
                usage: vulkan_buffer_usage_flags(usage),
                queue_family_indices: queue_family_indices.into(),
                sharing_mode: vk::SharingMode::Exclusive,
                ..default()
            };
            let mut buffer = vk::Buffer::null();
            vk_check!(unsafe {
                self.device_fn
                    .create_buffer(self.device, &create_info, None, &mut buffer)
            });

            let memory = self.allocate_memory(
                MemoryLocation::Host,
                false,
                true,
                allocator::VulkanAllocationResource::Buffer(buffer),
            );

            unsafe {
                self.device_fn.bind_buffer_memory2(
                    self.device,
                    &[vk::BindBufferMemoryInfo {
                        buffer,
                        memory: memory.device_memory(),
                        offset: memory.offset(),
                        ..default()
                    }],
                )
            };

            let address = unsafe {
                self.device_fn.get_buffer_device_address(
                    self.device,
                    &vk::BufferDeviceAddressInfo {
                        buffer,
                        ..default()
                    },
                )
            };

            let address = BufferAddress {
                value: address,
                phantom: PhantomData,
            };

            let ptr = NonNull::new(memory.mapped_ptr()).unwrap();

            frame.destroyed_buffers.lock().push_back(buffer);
            frame.destroyed_allocations.lock().push_back(memory);

            return TransientBuffer {
                ptr,
                len: size as usize,
                buffer: buffer.as_raw(),
                address,
                offset: 0,
            };
        }

        let per_thread = frame.per_thread.get(thread_token);
        let mut allocator = per_thread.transient_buffer_allocator.borrow_mut();

        let align = 1;

        let align = if usage.contains(BufferUsageFlags::UNIFORM) {
            align.max(
                self.physical_device_properties
                    .limits()
                    .min_uniform_buffer_offset_alignment,
            )
        } else {
            align
        };

        let align = if usage.contains(BufferUsageFlags::STORAGE) {
            align.max(
                self.physical_device_properties
                    .limits()
                    .min_storage_buffer_offset_alignment,
            )
        } else {
            align
        };

        // TODO: This is only necessary for buffer <-> image transfers, however
        // we're applying it to all transfer enabled requests.
        let align = if usage.contains(BufferUsageFlags::TRANSFER) {
            align.max(
                self.physical_device_properties
                    .limits()
                    .optimal_buffer_copy_offset_alignment,
            )
        } else {
            align
        };

        if allocator.offset < size || allocator.current.is_none() {
            let transient_buffer = self.allocate_transient_buffer();
            allocator.used_buffers.push(transient_buffer.clone());
            allocator.current = Some(transient_buffer);
            allocator.offset = VULKAN_CONSTANTS.transient_buffer_size;
        }

        allocator.offset = allocator.offset.wrapping_sub(size);
        allocator.offset &= !(align - 1);

        let current = allocator.current.as_ref().unwrap();

        let address = BufferAddress {
            value: current.address,
            phantom: PhantomData,
        };

        let address = address.byte_add(allocator.offset);

        TransientBuffer {
            ptr: NonNull::new(
                current
                    .memory
                    .mapped_ptr()
                    .wrapping_offset(allocator.offset as isize),
            )
            .unwrap(),
            len: size as usize,
            buffer: current.buffer.as_raw(),
            address,
            offset: allocator.offset,
        }
    }

    fn allocate_transient_buffer(&self) -> VulkanTransientBuffer {
        if let Some(transient_buffer) = self.recycled_transient_buffers.lock().pop_back() {
            return transient_buffer;
        }

        let queue_family_indices = &[self.universal_queue_family_index];

        // Allocate transient buffers with all possible usage flags so that we only
        // need a single collection of temporary buffers.
        let create_info = vk::BufferCreateInfo {
            size: VULKAN_CONSTANTS.transient_buffer_size,
            usage: vk::BufferUsageFlags::INDEX_BUFFER
                | vk::BufferUsageFlags::INDIRECT_BUFFER
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::STORAGE_BUFFER
                | vk::BufferUsageFlags::TRANSFER_DST
                | vk::BufferUsageFlags::TRANSFER_SRC
                | vk::BufferUsageFlags::UNIFORM_BUFFER,
            queue_family_indices: queue_family_indices.into(),
            sharing_mode: vk::SharingMode::Exclusive,
            ..default()
        };
        let mut buffer = vk::Buffer::null();
        vk_check!(unsafe {
            self.device_fn
                .create_buffer(self.device, &create_info, None, &mut buffer)
        });

        let memory = self.allocate_memory(
            MemoryLocation::Host,
            false,
            true,
            allocator::VulkanAllocationResource::Buffer(buffer),
        );

        assert!(!memory.mapped_ptr().is_null());
        // SAFETY: The memory has just been allocated, so as long as the pointer is
        // non-null, then we can create a slice for it.
        unsafe {
            let dst = std::slice::from_raw_parts_mut(memory.mapped_ptr(), memory.size().widen());
            dst.fill(0);
        }

        unsafe {
            self.device_fn.bind_buffer_memory2(
                self.device,
                &[vk::BindBufferMemoryInfo {
                    buffer,
                    memory: memory.device_memory(),
                    offset: memory.offset(),
                    ..default()
                }],
            )
        };

        let address = unsafe {
            self.device_fn.get_buffer_device_address(
                self.device,
                &vk::BufferDeviceAddressInfo {
                    buffer,
                    ..default()
                },
            )
        };

        VulkanTransientBuffer {
            buffer,
            address,
            memory,
        }
    }

    fn unwrap_buffer_arg(&self, buffer_arg: &BufferArg) -> (vk::Buffer, u64, u64) {
        match buffer_arg {
            BufferArg::Unmanaged(buffer) => (
                self.buffer_pool.lock().get(buffer.0).unwrap().buffer,
                0,
                vk::WHOLE_SIZE,
            ),
            BufferArg::Transient(transient) => (
                vk::Buffer::from_raw(transient.buffer),
                transient.offset,
                transient.len as u64,
            ),
            BufferArg::Persistent(buffer) => (
                self.buffer_pool.lock().get(buffer.buffer.0).unwrap().buffer,
                0,
                vk::WHOLE_SIZE,
            ),
        }
    }

    fn cache_pipeline_layout(&self, pipeline_layout: &PipelineLayout) -> Arc<VulkanPipelineLayout> {
        let hash = {
            let mut hasher = blake3_smol::Hasher::new();
            let bind_group_layout_pool = self.bind_group_layout_pool.lock();
            for bind_group_layout in pipeline_layout.bind_group_layouts {
                hasher.update(
                    bind_group_layout_pool
                        .get(bind_group_layout.0)
                        .unwrap()
                        .hash
                        .as_bytes(),
                );
            }
            for push_constant_range in pipeline_layout.push_constant_ranges {
                assert!(
                    push_constant_range.offset & 3 == 0,
                    "push constant offsets must be 4 byte aligned"
                );
                assert!(
                    push_constant_range.size & 3 == 0,
                    "push constant sizes must be 4 byte aligned"
                );
                hasher.update(&push_constant_range.stage_flags.as_raw().to_le_bytes());
                hasher.update(&push_constant_range.offset.to_le_bytes());
                hasher.update(&push_constant_range.size.to_le_bytes());
            }
            hasher.finalize()
        };

        let mut cache = self.pipeline_layout_cache.lock();
        let entry = cache.entry(hash);

        entry
            .or_insert_with(
                #[cold]
                || {
                    let arena = HybridArena::<1024>::new();

                    let bind_group_layout_pool = self.bind_group_layout_pool.lock();

                    let set_layouts_iter =
                        pipeline_layout
                            .bind_group_layouts
                            .iter()
                            .map(|bind_group_layout| {
                                bind_group_layout_pool
                                    .get(bind_group_layout.0)
                                    .unwrap()
                                    .descriptor_set_layout
                            });
                    let set_layouts = arena.alloc_slice_fill_iter(set_layouts_iter);

                    let push_constant_ranges_iter = pipeline_layout
                        .push_constant_ranges
                        .iter()
                        .copied()
                        .map(vulkan_push_constant_range);
                    let push_constant_ranges =
                        arena.alloc_slice_fill_iter(push_constant_ranges_iter);

                    let pipeline_layout = {
                        let create_info = vk::PipelineLayoutCreateInfo {
                            set_layouts: set_layouts.into(),
                            push_constant_ranges: push_constant_ranges.into(),
                            ..default()
                        };
                        let mut pipeline_layout = vk::PipelineLayout::null();
                        vk_check!(unsafe {
                            self.device_fn.create_pipeline_layout(
                                self.device,
                                &create_info,
                                None,
                                &mut pipeline_layout,
                            )
                        });
                        pipeline_layout
                    };

                    Arc::new(VulkanPipelineLayout { pipeline_layout })
                },
            )
            .clone()
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        vk_check!(unsafe { self.device_fn.device_wait_idle(self.device) });

        let device = self.device;

        for frame in self.frames.as_mut() {
            let frame = frame.get_mut();

            for semaphore in frame.recycled_semaphores.get_mut() {
                unsafe { self.device_fn.destroy_semaphore(device, *semaphore, None) }
            }

            for descriptor_pool in frame.recycled_descriptor_pools.get_mut() {
                unsafe {
                    self.device_fn
                        .destroy_descriptor_pool(device, *descriptor_pool, None)
                }
            }

            Self::destroy_deferred(&self.device_fn, device, frame);

            let mut arena = HybridArena::<512>::new();

            for per_thread in frame.per_thread.slots_mut() {
                let cmd_buffer_pool = per_thread.cmd_buffer_pool.get_mut();
                if !cmd_buffer_pool.command_buffers.is_empty() {
                    arena.reset();
                    let command_buffers = arena
                        .alloc_slice_fill_iter(cmd_buffer_pool.command_buffers.iter().copied());
                    unsafe {
                        self.device_fn.free_command_buffers(
                            device,
                            cmd_buffer_pool.command_pool,
                            command_buffers,
                        )
                    };
                }
                unsafe {
                    self.device_fn
                        .destroy_command_pool(device, cmd_buffer_pool.command_pool, None)
                }

                for buffer in per_thread
                    .transient_buffer_allocator
                    .get_mut()
                    .used_buffers
                    .iter()
                {
                    unsafe { self.device_fn.destroy_buffer(device, buffer.buffer, None) }
                }
            }
        }

        for buffer in self.recycled_transient_buffers.get_mut() {
            unsafe { self.device_fn.destroy_buffer(device, buffer.buffer, None) }
        }

        for buffer in self.buffer_pool.get_mut().values() {
            unsafe { self.device_fn.destroy_buffer(device, buffer.buffer, None) }
        }

        {
            let mut image_views = Vec::new();
            let mut images = Vec::new();
            for image in self.image_pool.get_mut().values() {
                match image {
                    VulkanImageHolder::Unique(image) => {
                        image_views.push(image.view);
                        images.push(image.image.image);
                    }
                    VulkanImageHolder::Shared(image) => {
                        image_views.push(image.view);
                    }
                    VulkanImageHolder::Swapchain(image) => {
                        image_views.push(image.view);
                    }
                }
            }

            for image_view in image_views {
                unsafe { self.device_fn.destroy_image_view(device, image_view, None) }
            }

            for image in images {
                unsafe { self.device_fn.destroy_image(device, image, None) }
            }
        }

        for sampler in self.sampler_pool.get_mut().values() {
            unsafe { self.device_fn.destroy_sampler(device, sampler.0, None) }
        }

        for pipeline in self.pipeline_pool.get_mut().values() {
            unsafe {
                self.device_fn
                    .destroy_pipeline(device, pipeline.pipeline, None)
            }
        }

        for pipeline_layout in self.pipeline_layout_cache.get_mut().values() {
            unsafe {
                self.device_fn.destroy_pipeline_layout(
                    device,
                    pipeline_layout.pipeline_layout,
                    None,
                );
            }
        }

        for bind_group_layout in self.bind_group_layout_pool.get_mut().values() {
            unsafe {
                self.device_fn.destroy_descriptor_set_layout(
                    device,
                    bind_group_layout.descriptor_set_layout,
                    None,
                )
            }
        }

        for fence in self.recycled_fences.get_mut() {
            unsafe { self.device_fn.destroy_fence(device, *fence, None) }
        }

        for semaphore in self
            .recycled_semaphores
            .get_mut()
            .iter()
            .chain(std::iter::once(&self.universal_queue_semaphore))
        {
            unsafe { self.device_fn.destroy_semaphore(device, *semaphore, None) }
        }

        for descriptor_pool in self.recycled_descriptor_pools.get_mut() {
            unsafe {
                self.device_fn
                    .destroy_descriptor_pool(device, *descriptor_pool, None)
            }
        }

        self.wsi_drop();

        self.allocator_drop();

        unsafe { self.device_fn.destroy_device(device, None) }
        unsafe { self.instance_fn.destroy_instance(self.instance, None) };
    }
}
