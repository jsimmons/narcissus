use std::os::raw::c_void;

use vulkan_sys as vk;
use vulkan_sys::cstr;

mod libc {
    use std::os::raw::{c_char, c_int, c_void};

    pub const RTLD_NOW: c_int = 0x2;
    pub const RTLD_LOCAL: c_int = 0;

    extern "C" {
        pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }
}

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

fn vk_vec<T, F: FnMut(&mut u32, *mut T) -> vk::Result>(mut f: F) -> Vec<T> {
    let mut count = 0;
    vk_check!(f(&mut count, std::ptr::null_mut()));
    let mut v = Vec::with_capacity(count as usize);
    vk_check!(f(&mut count, v.as_mut_ptr()));
    unsafe { v.set_len(count as usize) };
    v
}

/// Avoid the awful..default()` spam.
#[inline(always)]
fn default<T: Default>() -> T {
    T::default()
}

pub fn main() {
    let get_proc_addr = unsafe {
        let module = libc::dlopen(
            cstr!("libvulkan.so.1").as_ptr(),
            libc::RTLD_NOW | libc::RTLD_LOCAL,
        );
        libc::dlsym(module, cstr!("vkGetInstanceProcAddr").as_ptr())
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

    #[cfg(debug_assertions)]
    let enabled_layers = &[cstr!("VK_LAYER_KHRONOS_validation").as_ptr()];
    #[cfg(not(debug_assertions))]
    let enabled_layers = &[];

    let instance = {
        let application_info = vk::ApplicationInfo {
            application_name: cstr!("TRIANGLE").as_ptr(),
            application_version: 0,
            engine_name: cstr!("TRIANGLE").as_ptr(),
            engine_version: 0,
            api_version: vk::VERSION_1_3,
            ..default()
        };
        let create_info = vk::InstanceCreateInfo {
            enabled_layers: enabled_layers.into(),
            application_info: Some(&application_info),
            ..default()
        };
        let mut instance = vk::Instance::null();
        vk_check!(global_fn.create_instance(&create_info, None, &mut instance));
        instance
    };

    let instance_fn = vk::InstanceFunctions::new(&global_fn, instance, vk::VERSION_1_2);

    let physical_devices = vk_vec(|count, ptr| unsafe {
        instance_fn.enumerate_physical_devices(instance, count, ptr)
    });

    let physical_device = physical_devices
        .iter()
        .copied()
        .filter(|&physical_device| {
            let (
                physical_device_properties,
                _physical_device_properties_11,
                _physical_device_properties_12,
                _physical_device_properties_13,
            ) = {
                let mut properties_13 = vk::PhysicalDeviceVulkan13Properties::default();
                let mut properties_12 = vk::PhysicalDeviceVulkan12Properties::default();
                let mut properties_11 = vk::PhysicalDeviceVulkan11Properties::default();
                let mut properties = vk::PhysicalDeviceProperties2::default();
                unsafe {
                    properties._next = std::mem::transmute::<_, *mut c_void>(&mut properties_11);
                    properties_11._next = std::mem::transmute::<_, *mut c_void>(&mut properties_12);
                    properties_12._next = std::mem::transmute::<_, *mut c_void>(&mut properties_13);
                    instance_fn.get_physical_device_properties2(physical_device, &mut properties);
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
                let mut features_12 = vk::PhysicalDeviceVulkan12Features::default();
                let mut features_11 = vk::PhysicalDeviceVulkan11Features::default();
                let mut features = vk::PhysicalDeviceFeatures2::default();
                unsafe {
                    features._next = std::mem::transmute::<_, *mut c_void>(&mut features_11);
                    features_11._next = std::mem::transmute::<_, *mut c_void>(&mut features_12);
                    features_12._next = std::mem::transmute::<_, *mut c_void>(&mut features_13);
                    instance_fn.get_physical_device_features2(physical_device, &mut features);
                }
                (features.features, features_11, features_12, features_13)
            };

            physical_device_properties.properties.api_version >= vk::VERSION_1_3
                && physical_device_features_13.dynamic_rendering == vk::Bool32::True
                && physical_device_features_12.timeline_semaphore == vk::Bool32::True
        })
        .next()
        .expect("no supported physical devices reported");

    let physical_device_memory_properties = unsafe {
        let mut memory_properties = vk::PhysicalDeviceMemoryProperties::default();
        instance_fn.get_physical_device_memory_properties(physical_device, &mut memory_properties);
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

    let device = unsafe {
        let queue_priorities = &[1.0];
        let device_queue_create_infos = &[vk::DeviceQueueCreateInfo {
            queue_family_index,
            queue_priorities: queue_priorities.into(),
            ..default()
        }];
        let enabled_features_13 = vk::PhysicalDeviceVulkan13Features {
            dynamic_rendering: vk::Bool32::True,
            synchronization2: vk::Bool32::True,
            ..default()
        };
        let enabled_features_12 = vk::PhysicalDeviceVulkan12Features {
            _next: std::mem::transmute::<_, *mut c_void>(&enabled_features_13),
            timeline_semaphore: vk::Bool32::True,
            ..default()
        };
        let enabled_features_11 = vk::PhysicalDeviceVulkan11Features {
            _next: std::mem::transmute::<_, *mut c_void>(&enabled_features_12),
            ..default()
        };
        let enabled_features = vk::PhysicalDeviceFeatures2 {
            _next: std::mem::transmute::<_, *mut c_void>(&enabled_features_11),
            ..default()
        };
        let create_info = vk::DeviceCreateInfo {
            _next: std::mem::transmute::<_, *mut c_void>(&enabled_features),
            queue_create_infos: device_queue_create_infos.into(),
            ..default()
        };
        let mut device = vk::Device::null();
        vk_check!(instance_fn.create_device(physical_device, &create_info, None, &mut device));
        device
    };

    let device_fn = vk::DeviceFunctions::new(&instance_fn, device, vk::VERSION_1_3);

    let queue = unsafe {
        let mut queue = vk::Queue::default();
        device_fn.get_device_queue(device, queue_family_index, 0, &mut queue);
        queue
    };

    let mut semaphore_value = 0;
    let semaphore = unsafe {
        let type_create_info = vk::SemaphoreTypeCreateInfo {
            semaphore_type: vk::SemaphoreType::Timeline,
            initial_value: semaphore_value,
            ..default()
        };
        let create_info = vk::SemaphoreCreateInfo {
            _next: std::mem::transmute::<_, _>(&type_create_info),
            ..default()
        };
        let mut semaphore = vk::Semaphore::null();
        vk_check!(device_fn.create_semaphore(device, &create_info, None, &mut semaphore));
        semaphore
    };

    let create_shader_module = |code: &[u8]| {
        debug_assert!(code.as_ptr().align_offset(4) == 0);
        let create_info = vk::ShaderModuleCreateInfo {
            code: code.into(),
            ..default()
        };
        let mut shader_module = vk::ShaderModule::null();
        vk_check!(device_fn.create_shader_module(device, &create_info, None, &mut shader_module));
        shader_module
    };

    let create_graphics_pipeline = |vert, frag| {
        let vert_shader_module = create_shader_module(vert);
        let frag_shader_module = create_shader_module(frag);

        let layout = {
            let create_info = vk::PipelineLayoutCreateInfo::default();
            let mut pipeline_layout = vk::PipelineLayout::null();
            vk_check!(device_fn.create_pipeline_layout(
                device,
                &create_info,
                None,
                &mut pipeline_layout
            ));
            pipeline_layout
        };

        let mut pipeline = vk::Pipeline::null();

        unsafe {
            let stages = &[
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::ShaderStageFlags::VERTEX,
                    module: vert_shader_module,
                    name: cstr!("main").as_ptr(),
                    ..default()
                },
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::ShaderStageFlags::FRAGMENT,
                    module: frag_shader_module,
                    name: cstr!("main").as_ptr(),
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
            let color_attachment_formats = &[vk::Format::R8G8B8A8_SRGB];
            let pipeline_rendering_create_info = vk::PipelineRenderingCreateInfo {
                color_attachment_formats: color_attachment_formats.into(),
                ..default()
            };
            let create_info = vk::GraphicsPipelineCreateInfo {
                _next: std::mem::transmute::<_, _>(&pipeline_rendering_create_info),
                stages: stages.into(),
                vertex_input_state: Some(&vertex_input_state),
                input_assembly_state: Some(&input_assembly_state),
                viewport_state: Some(&viewport_state),
                rasterization_state: Some(&rasterization_state),
                multisample_state: Some(&multisample_state),
                color_blend_state: Some(&color_blend_state),
                dynamic_state: Some(&dynamic_state),
                layout,
                ..default()
            };
            vk_check!(device_fn.create_graphics_pipelines(
                device,
                vk::PipelineCache::null(),
                &[create_info],
                None,
                std::slice::from_mut(&mut pipeline),
            ));
        }

        unsafe { device_fn.destroy_shader_module(device, vert_shader_module, None) };
        unsafe { device_fn.destroy_shader_module(device, frag_shader_module, None) };

        (layout, pipeline)
    };

    #[repr(align(4))]
    struct Spirv<const LEN: usize>([u8; LEN]);

    let vert_shader_spv = Spirv(*include_bytes!("triangle.vert.spv"));
    let frag_shader_spv = Spirv(*include_bytes!("triangle.frag.spv"));

    let (pipeline_layout, pipeline) =
        create_graphics_pipeline(&vert_shader_spv.0, &frag_shader_spv.0);

    let command_pool = {
        let create_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index,
            ..default()
        };
        let mut command_pool = vk::CommandPool::default();
        vk_check!(device_fn.create_command_pool(device, &create_info, None, &mut command_pool));
        command_pool
    };

    let find_memory_type = |filter, flags| {
        (0..physical_device_memory_properties.memory_type_count)
            .map(|memory_type_index| {
                (
                    memory_type_index,
                    physical_device_memory_properties.memory_types[memory_type_index as usize],
                )
            })
            .filter(|(i, memory_type)| {
                (filter & (1 << i)) != 0 && memory_type.property_flags.contains(flags)
            })
            .next()
            .expect("could not find memory type matching flags")
            .0
    };

    let create_image = |width, height, format, tiling, usage, memory_properties| {
        let queue_family_indices = &[queue_family_index];
        let create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::Type2d,
            extent: vk::Extent3d {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            format,
            tiling,
            usage,
            sharing_mode: vk::SharingMode::Exclusive,
            samples: vk::SampleCountFlags::SAMPLE_COUNT_1,
            queue_family_indices: queue_family_indices.into(),
            initial_layout: vk::ImageLayout::Undefined,
            ..default()
        };
        let mut image = vk::Image::null();
        vk_check!(device_fn.create_image(device, &create_info, None, &mut image));

        let memory_requirements = {
            let mut memory_requirements = vk::MemoryRequirements2::default();
            unsafe {
                device_fn.get_image_memory_requirements2(
                    device,
                    &vk::ImageMemoryRequirementsInfo2 { image, ..default() },
                    &mut memory_requirements,
                )
            };
            memory_requirements
        };

        let memory_type_index = find_memory_type(
            memory_requirements.memory_requirements.memory_type_bits,
            memory_properties,
        );

        let mut memory = vk::DeviceMemory::null();
        vk_check!(device_fn.allocate_memory(
            device,
            &vk::MemoryAllocateInfo {
                allocation_size: memory_requirements.memory_requirements.size,
                memory_type_index,
                ..default()
            },
            None,
            &mut memory,
        ));
        unsafe {
            device_fn.bind_image_memory2(
                device,
                &[vk::BindImageMemoryInfo {
                    image,
                    memory,
                    offset: 0,
                    ..default()
                }],
            )
        };

        (image, memory)
    };

    let create_image_and_view = |width, height, format, tiling, usage, memory_properties| {
        let (image, memory) = create_image(width, height, format, tiling, usage, memory_properties);
        let mut view = vk::ImageView::null();
        let create_info = vk::ImageViewCreateInfo {
            image,
            view_type: vk::ImageViewType::Type2d,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..default()
        };
        vk_check!(device_fn.create_image_view(device, &create_info, None, &mut view));
        (image, view, memory)
    };

    let width = 120;
    let height = 40;
    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    let scissor = vk::Rect2d {
        offset: vk::Offset2d { x: 0, y: 0 },
        extent: vk::Extent2d { width, height },
    };

    let (image, image_view, image_memory) = create_image_and_view(
        width,
        height,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    let (host_image, host_image_memory) = create_image(
        width,
        height,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::LINEAR,
        vk::ImageUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    let host_subresource_layout = unsafe {
        let mut layout = vk::SubresourceLayout::default();
        device_fn.get_image_subresource_layout(
            device,
            host_image,
            &vk::ImageSubresource {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                array_layer: 0,
            },
            &mut layout,
        );
        layout
    };

    let mut data = std::ptr::null_mut();
    vk_check!(device_fn.map_memory(
        device,
        host_image_memory,
        0,
        vk::WHOLE_SIZE,
        vk::MemoryMapFlags::default(),
        &mut data,
    ));

    // Do the rendering!
    let command_buffer = {
        let mut command_buffers = [vk::CommandBuffer::default()];
        let allocate_info = vk::CommandBufferAllocateInfo {
            command_pool,
            command_buffer_count: command_buffers.len() as u32,
            ..default()
        };
        vk_check!(device_fn.allocate_command_buffers(
            device,
            &allocate_info,
            command_buffers.as_mut_ptr()
        ));
        command_buffers[0]
    };

    {
        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..default()
        };
        vk_check!(device_fn.begin_command_buffer(command_buffer, &begin_info));
    }

    unsafe {
        device_fn.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::Graphics, pipeline)
    };

    unsafe { device_fn.cmd_set_viewport_with_count(command_buffer, &[viewport]) };
    unsafe { device_fn.cmd_set_scissor_with_count(command_buffer, &[scissor]) };

    let image_memory_barriers = &[vk::ImageMemoryBarrier2 {
        src_stage_mask: vk::PipelineStageFlags2::TOP_OF_PIPE,
        src_access_mask: vk::AccessFlags2::NONE,
        dst_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        old_layout: vk::ImageLayout::Undefined,
        new_layout: vk::ImageLayout::ColorAttachmentOptimal,
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
    unsafe {
        device_fn.cmd_pipeline_barrier2(
            command_buffer,
            &vk::DependencyInfo {
                image_memory_barriers: image_memory_barriers.into(),
                ..default()
            },
        )
    };

    unsafe {
        device_fn.cmd_begin_rendering(
            command_buffer,
            &vk::RenderingInfo {
                render_area: scissor,
                layer_count: 1,
                color_attachments: [vk::RenderingAttachmentInfo {
                    image_view,
                    image_layout: vk::ImageLayout::AttachmentOptimal,
                    resolve_image_layout: vk::ImageLayout::AttachmentOptimal,
                    load_op: vk::AttachmentLoadOp::Clear,
                    store_op: vk::AttachmentStoreOp::Store,
                    clear_value: vk::ClearValue {
                        color: vk::ClearColorValue {
                            f32: [0.392157, 0.584314, 0.929412, 1.0],
                        },
                    },
                    ..default()
                }]
                .as_ref()
                .into(),
                ..default()
            },
        );
        device_fn.cmd_draw(command_buffer, 3, 1, 0, 0);
        device_fn.cmd_end_rendering(command_buffer);
    };

    let image_memory_barriers = &[
        // transition color attachment to transfer src
        vk::ImageMemoryBarrier2 {
            src_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags2::TRANSFER,
            dst_access_mask: vk::AccessFlags2::TRANSFER_READ,
            old_layout: vk::ImageLayout::ColorAttachmentOptimal,
            new_layout: vk::ImageLayout::TransferSrcOptimal,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..default()
        },
        // transition host image to transfer dst
        vk::ImageMemoryBarrier2 {
            src_stage_mask: vk::PipelineStageFlags2::TRANSFER,
            src_access_mask: vk::AccessFlags2::NONE,
            dst_stage_mask: vk::PipelineStageFlags2::TRANSFER,
            dst_access_mask: vk::AccessFlags2::TRANSFER_WRITE,
            old_layout: vk::ImageLayout::Undefined,
            new_layout: vk::ImageLayout::TransferDstOptimal,
            image: host_image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..default()
        },
    ];
    unsafe {
        device_fn.cmd_pipeline_barrier2(
            command_buffer,
            &vk::DependencyInfo {
                image_memory_barriers: image_memory_barriers.into(),
                ..default()
            },
        )
    };

    let regions = &[vk::ImageCopy {
        src_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_array_layer: 0,
            layer_count: 1,
            mip_level: 0,
        },
        src_offset: vk::Offset3d { x: 0, y: 0, z: 0 },
        dst_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_array_layer: 0,
            layer_count: 1,
            mip_level: 0,
        },
        dst_offset: vk::Offset3d { x: 0, y: 0, z: 0 },
        extent: vk::Extent3d {
            width,
            height,
            depth: 1,
        },
    }];
    unsafe {
        device_fn.cmd_copy_image(
            command_buffer,
            image,
            vk::ImageLayout::TransferSrcOptimal,
            host_image,
            vk::ImageLayout::TransferDstOptimal,
            regions,
        )
    };

    unsafe {
        device_fn.cmd_pipeline_barrier2(
            command_buffer,
            &vk::DependencyInfo {
                image_memory_barriers: [
                    // transition host image to general so we can read it
                    vk::ImageMemoryBarrier2 {
                        src_stage_mask: vk::PipelineStageFlags2::TRANSFER,
                        src_access_mask: vk::AccessFlags2::TRANSFER_WRITE,
                        dst_stage_mask: vk::PipelineStageFlags2::TRANSFER,
                        dst_access_mask: vk::AccessFlags2::MEMORY_READ,
                        old_layout: vk::ImageLayout::TransferDstOptimal,
                        new_layout: vk::ImageLayout::General,
                        image: host_image,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        ..default()
                    },
                ]
                .as_ref()
                .into(),
                ..default()
            },
        )
    };

    vk_check!(device_fn.end_command_buffer(command_buffer));

    // SUBMIT!
    semaphore_value += 1;

    let command_buffer_infos = &[vk::CommandBufferSubmitInfo {
        command_buffer,
        ..default()
    }];
    let signal_semaphore_infos = &[vk::SemaphoreSubmitInfo {
        semaphore,
        semaphore_value,
        stage_mask: vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
        ..default()
    }];
    let submit = vk::SubmitInfo2 {
        command_buffer_infos: command_buffer_infos.into(),
        signal_semaphore_infos: signal_semaphore_infos.into(),
        ..default()
    };
    vk_check!(device_fn.queue_submit2(queue, &[submit], vk::Fence::null()));

    vk_check!(device_fn.wait_semaphores(
        device,
        &vk::SemaphoreWaitInfo {
            semaphores: (&[semaphore], &[semaphore_value]).into(),
            ..default()
        },
        !0,
    ));

    let data = data as *const u8;
    let image_bytes = unsafe {
        std::slice::from_raw_parts(
            data.offset(host_subresource_layout.offset as isize),
            host_subresource_layout.size as usize,
        )
    };

    #[inline]
    unsafe fn as_chunks_unchecked<T, const N: usize>(slice: &[T]) -> &[[T; N]] {
        debug_assert_ne!(N, 0);
        debug_assert_eq!(slice.len() % N, 0);
        let new_len = slice.len() / N;
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        std::slice::from_raw_parts(slice.as_ptr().cast(), new_len)
    }

    print!("\x1b[2J");
    for row in image_bytes.chunks_exact(host_subresource_layout.row_pitch as usize) {
        let pixels = unsafe { as_chunks_unchecked(row) };
        for [r, g, b, _a] in &pixels[0..width as usize] {
            print!("\x1b[38;2;{r};{g};{b}m█");
        }
        println!();
    }
    print!("\x1b[0m");

    unsafe { device_fn.free_command_buffers(device, command_pool, &[command_buffer]) };

    unsafe { device_fn.destroy_image_view(device, image_view, None) };
    unsafe { device_fn.destroy_image(device, image, None) };
    unsafe { device_fn.free_memory(device, image_memory, None) };

    unsafe { device_fn.destroy_image(device, host_image, None) };
    unsafe { device_fn.free_memory(device, host_image_memory, None) };

    unsafe { device_fn.destroy_pipeline(device, pipeline, None) };
    unsafe { device_fn.destroy_pipeline_layout(device, pipeline_layout, None) };

    unsafe { device_fn.destroy_command_pool(device, command_pool, None) };

    unsafe { device_fn.destroy_semaphore(device, semaphore, None) };

    unsafe { device_fn.destroy_device(device, None) };
    unsafe { instance_fn.destroy_instance(instance, None) };
}
