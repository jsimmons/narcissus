use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    ffi::CStr,
};

use narcissus_core::{
    default,
    raw_window::{AsRawWindow, RawWindow},
    HybridArena, Mutex, Pool, Widen,
};
use vulkan_sys as vk;

use crate::{
    backend::vulkan::{
        from_vulkan_image_usage_flags, vk_vec, vulkan_color_space, vulkan_format,
        vulkan_image_usage_flags, vulkan_present_mode, VulkanImageHolder, VulkanImageSwapchain,
    },
    vk_check, ColorSpace, Frame, Image, ImageFormat, PresentMode, SwapchainConfigurator,
    SwapchainImage, SwapchainOutOfDateError,
};

use super::{VulkanDevice, VulkanFrame, VULKAN_CONSTANTS};

#[derive(Default)]
struct VulkanPresentInfo {
    acquire: vk::Semaphore,
    release: vk::Semaphore,
    swapchain: vk::SwapchainKHR,
    image_index: u32,
}

enum VulkanSwapchainState {
    Vacant,
    Occupied {
        width: u32,
        height: u32,
        swapchain: vk::SwapchainKHR,
        image_views: Box<[Image]>,
    },
}

pub struct VulkanSwapchain {
    present_mode: vk::PresentModeKHR,
    surface_format: vk::SurfaceFormatKHR,
    usage_flags: vk::ImageUsageFlags,
    capabilities: vk::SurfaceCapabilitiesKHR,
    state: VulkanSwapchainState,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct VulkanWsiSupport {
    wayland: bool,
    xlib: bool,
    xcb: bool,
    surface_maintenance1: bool,
    swapchain_maintenance1: bool,
    swapchain_mutable_format: bool,
}

struct RecycleSwapchainSemaphore {
    fence: vk::Fence,
    semaphore: vk::Semaphore,
    swapchain: vk::SwapchainKHR,
}

struct DestroySwapchain {
    swapchain: vk::SwapchainKHR,
    surface: vk::SurfaceKHR,
    image_views: Box<[vk::ImageView]>,
}

pub struct VulkanWsi {
    support: VulkanWsiSupport,

    surfaces: Mutex<HashMap<RawWindow, vk::SurfaceKHR>>,
    swapchains: Mutex<HashMap<vk::SurfaceKHR, VulkanSwapchain>>,
    suboptimal_swapchains: Mutex<HashSet<vk::SwapchainKHR>>,

    recycle_swapchain_semaphores: Mutex<Vec<RecycleSwapchainSemaphore>>,
    destroy_swapchains: Mutex<Vec<DestroySwapchain>>,

    xcb_surface_fn: Option<vk::XcbSurfaceKHRFunctions>,
    xlib_surface_fn: Option<vk::XlibSurfaceKHRFunctions>,
    wayland_surface_fn: Option<vk::WaylandSurfaceKHRFunctions>,
    surface_fn: vk::SurfaceKHRFunctions,
    swapchain_fn: vk::SwapchainKHRFunctions,
}

impl VulkanWsi {
    /// Check available WSI instance extensions, and append required extensions to
    /// `enabled_extensions`.
    pub fn check_instance_extensions<'a>(
        extension_properties: &'a [vk::ExtensionProperties],
        enabled_extensions: &mut Vec<&'a CStr>,
        wsi_support: &mut VulkanWsiSupport,
    ) {
        for extension in extension_properties {
            let extension_name = CStr::from_bytes_until_nul(&extension.extension_name).unwrap();

            match extension_name.to_str().unwrap() {
                "VK_EXT_surface_maintenance1" => {
                    wsi_support.surface_maintenance1 = true;
                    enabled_extensions.push(extension_name);
                }
                "VK_KHR_wayland_surface" => {
                    wsi_support.wayland = true;
                    enabled_extensions.push(extension_name);
                }
                "VK_KHR_xlib_surface" => {
                    wsi_support.xlib = true;
                    enabled_extensions.push(extension_name);
                }
                "VK_KHR_xcb_surface" => {
                    wsi_support.xcb = true;
                    enabled_extensions.push(extension_name);
                }
                _ => {}
            }
        }

        // If we found any surface extensions, we need to additionally enable
        // `VK_KHR_surface`.
        if wsi_support.wayland || wsi_support.xlib || wsi_support.xcb {
            enabled_extensions.push(c"VK_KHR_surface");
        }
    }

    /// Check available WSI device extensions, and append required extensions to
    /// `enabled_extensions`.
    ///
    /// Panics if device does not support required extensions.
    pub fn check_device_extensions<'a>(
        extension_properties: &'a [vk::ExtensionProperties],
        enabled_extensions: &mut Vec<&'a CStr>,
        wsi_support: &mut VulkanWsiSupport,
    ) {
        let mut khr_swapchain_support = false;
        for extension in extension_properties {
            let extension_name = CStr::from_bytes_until_nul(&extension.extension_name).unwrap();
            match extension_name.to_str().unwrap() {
                "VK_KHR_swapchain" => {
                    khr_swapchain_support = true;
                    enabled_extensions.push(extension_name);
                }
                "VK_EXT_swapchain_maintenance1" => {
                    wsi_support.swapchain_maintenance1 = true;
                    enabled_extensions.push(extension_name);
                }
                "VK_KHR_swapchain_mutable_format" => {
                    wsi_support.swapchain_mutable_format = true;
                    enabled_extensions.push(extension_name);
                }
                _ => {}
            }
        }
        assert!(khr_swapchain_support);
        assert!(wsi_support.swapchain_mutable_format);
    }

    pub fn new(
        global_fn: &vk::GlobalFunctions,
        instance: vk::Instance,
        support: VulkanWsiSupport,
    ) -> Self {
        let xcb_surface_fn = if support.xcb {
            Some(vk::XcbSurfaceKHRFunctions::new(global_fn, instance))
        } else {
            None
        };

        let xlib_surface_fn = if support.xlib {
            Some(vk::XlibSurfaceKHRFunctions::new(global_fn, instance))
        } else {
            None
        };

        let wayland_surface_fn = if support.wayland {
            Some(vk::WaylandSurfaceKHRFunctions::new(global_fn, instance))
        } else {
            None
        };

        let surface_fn = vk::SurfaceKHRFunctions::new(global_fn, instance);
        let swapchain_fn = vk::SwapchainKHRFunctions::new(global_fn, instance, vk::VERSION_1_1);

        VulkanWsi {
            support,

            surfaces: default(),
            swapchains: default(),
            suboptimal_swapchains: default(),

            recycle_swapchain_semaphores: default(),
            destroy_swapchains: default(),

            xcb_surface_fn,
            xlib_surface_fn,
            wayland_surface_fn,
            surface_fn,
            swapchain_fn,
        }
    }
}

#[derive(Default)]
pub struct VulkanWsiFrame {
    presents: Mutex<HashMap<vk::SurfaceKHR, VulkanPresentInfo>>,
}

impl VulkanDevice {
    pub fn acquire_swapchain(
        &self,
        frame: &Frame,
        window: &dyn AsRawWindow,
        width: u32,
        height: u32,
        configurator: &mut dyn SwapchainConfigurator,
    ) -> Result<SwapchainImage, SwapchainOutOfDateError> {
        let raw_window = window.as_raw_window();
        let mut surfaces = self.wsi.surfaces.lock();
        let surface = *surfaces
            .entry(raw_window)
            .or_insert_with(|| match raw_window {
                RawWindow::Xcb(xcb) => {
                    let create_info = vk::XcbSurfaceCreateInfoKHR {
                        connection: xcb.connection,
                        window: xcb.window,
                        ..default()
                    };
                    let mut surface = vk::SurfaceKHR::null();
                    vk_check!(self
                        .wsi
                        .xcb_surface_fn
                        .as_ref()
                        .unwrap()
                        .create_xcb_surface(self.instance, &create_info, None, &mut surface));
                    surface
                }
                RawWindow::Xlib(xlib) => {
                    let create_info = vk::XlibSurfaceCreateInfoKHR {
                        display: xlib.display,
                        window: xlib.window,
                        ..default()
                    };
                    let mut surface = vk::SurfaceKHR::null();
                    vk_check!(self
                        .wsi
                        .xlib_surface_fn
                        .as_ref()
                        .unwrap()
                        .create_xlib_surface(self.instance, &create_info, None, &mut surface));
                    surface
                }
                RawWindow::Wayland(wayland) => {
                    let create_info = vk::WaylandSurfaceCreateInfoKHR {
                        display: wayland.display,
                        surface: wayland.surface,
                        ..default()
                    };
                    let mut surface = vk::SurfaceKHR::null();
                    vk_check!(self
                        .wsi
                        .wayland_surface_fn
                        .as_ref()
                        .unwrap()
                        .create_wayland_surface(self.instance, &create_info, None, &mut surface));
                    surface
                }
            });

        let mut swapchains = self.wsi.swapchains.lock();
        let vulkan_swapchain = swapchains.entry(surface).or_insert_with(|| {
            let mut supported = vk::Bool32::False;
            vk_check!(unsafe {
                self.wsi.surface_fn.get_physical_device_surface_support(
                    self.physical_device,
                    self.universal_queue_family_index,
                    surface,
                    &mut supported,
                )
            });

            assert_eq!(
                supported,
                vk::Bool32::True,
                "universal queue does not support presenting this surface"
            );

            let available_present_modes = vk_vec(|count, ptr| unsafe {
                self.wsi
                    .surface_fn
                    .get_physical_device_surface_present_modes(
                        self.physical_device,
                        surface,
                        count,
                        ptr,
                    )
            })
            .into_iter()
            .filter_map(|present_mode| match present_mode {
                vk::PresentModeKHR::Immediate => Some(PresentMode::Immediate),
                vk::PresentModeKHR::Mailbox => Some(PresentMode::Mailbox),
                vk::PresentModeKHR::Fifo => Some(PresentMode::Fifo),
                vk::PresentModeKHR::FifoRelaxed => Some(PresentMode::FifoRelaxed),
                vk::PresentModeKHR::SharedDemandRefresh => None,
                vk::PresentModeKHR::SharedContinuousRefresh => None,
            })
            .collect::<Vec<_>>();

            let supported_surface_formats = vk_vec(|count, ptr| unsafe {
                self.wsi.surface_fn.get_physical_device_surface_formats(
                    self.physical_device,
                    surface,
                    count,
                    ptr,
                )
            })
            .into_iter()
            .filter_map(
                |vk::SurfaceFormatKHR {
                     format,
                     color_space,
                 }| {
                    let color_space = match color_space {
                        vk::ColorSpaceKHR::SrgbNonlinearKhr => Some(ColorSpace::Srgb),
                        _ => None,
                    }?;
                    let format = match format {
                        vk::Format::R8_SRGB => Some(ImageFormat::R8_SRGB),
                        vk::Format::R8_UNORM => Some(ImageFormat::R8_UNORM),
                        vk::Format::R8G8B8A8_SRGB => Some(ImageFormat::RGBA8_SRGB),
                        vk::Format::R8G8B8A8_UNORM => Some(ImageFormat::RGBA8_UNORM),
                        vk::Format::R16G16B16A16_SFLOAT => Some(ImageFormat::RGBA16_FLOAT),
                        vk::Format::B8G8R8A8_SRGB => Some(ImageFormat::BGRA8_SRGB),
                        vk::Format::B8G8R8A8_UNORM => Some(ImageFormat::BGRA8_UNORM),
                        vk::Format::A2R10G10B10_UNORM_PACK32 => {
                            Some(ImageFormat::A2R10G10B10_UNORM)
                        }
                        vk::Format::A2B10G10R10_UNORM_PACK32 => {
                            Some(ImageFormat::A2B10G10R10_UNORM)
                        }
                        vk::Format::E5B9G9R9_UFLOAT_PACK32 => Some(ImageFormat::E5B9G9R9_UFLOAT),
                        vk::Format::D32_SFLOAT => Some(ImageFormat::DEPTH_F32),
                        _ => None,
                    }?;
                    Some((format, color_space))
                },
            )
            .collect::<Vec<_>>();

            let mut capabilities = vk::SurfaceCapabilitiesKHR::default();
            vk_check!(unsafe {
                self.wsi
                    .surface_fn
                    .get_physical_device_surface_capabilities(
                        self.physical_device,
                        surface,
                        &mut capabilities,
                    )
            });

            let supported_usage_flags =
                from_vulkan_image_usage_flags(capabilities.supported_usage_flags);

            let present_mode = configurator.choose_present_mode(&available_present_modes);
            let (usage_flags, surface_format) = configurator
                .choose_surface_format(supported_usage_flags, &supported_surface_formats);

            assert!(available_present_modes.contains(&present_mode));
            assert!((!supported_usage_flags.as_raw() & usage_flags.as_raw()) == 0);
            assert!(supported_surface_formats
                .iter()
                .any(|&supported_format| { supported_format == surface_format }));

            let present_mode = vulkan_present_mode(present_mode);
            let usage_flags = vulkan_image_usage_flags(usage_flags);
            let surface_format = vk::SurfaceFormatKHR {
                format: vulkan_format(surface_format.0),
                color_space: vulkan_color_space(surface_format.1),
            };
            VulkanSwapchain {
                present_mode,
                surface_format,
                usage_flags,
                state: VulkanSwapchainState::Vacant,
                capabilities,
            }
        });

        let frame = self.frame(frame);
        let mut image_pool = self.image_pool.lock();

        let mut present_swapchains = frame.wsi.presents.lock();
        let present_info = match present_swapchains.entry(surface) {
            Entry::Occupied(_) => panic!("acquiring swapchain multiple times in a frame"),
            Entry::Vacant(entry) => entry.insert(default()),
        };

        vk_check!(unsafe {
            self.wsi
                .surface_fn
                .get_physical_device_surface_capabilities(
                    self.physical_device,
                    surface,
                    &mut vulkan_swapchain.capabilities,
                )
        });

        let width = width.clamp(
            vulkan_swapchain.capabilities.min_image_extent.width,
            vulkan_swapchain.capabilities.max_image_extent.width,
        );
        let height = height.clamp(
            vulkan_swapchain.capabilities.min_image_extent.height,
            vulkan_swapchain.capabilities.max_image_extent.height,
        );

        let mut suboptimal_swapchains = self.wsi.suboptimal_swapchains.lock();

        let mut old_swapchain = vk::SwapchainKHR::null();
        loop {
            match &mut vulkan_swapchain.state {
                VulkanSwapchainState::Vacant => {
                    let mut new_swapchain = vk::SwapchainKHR::null();
                    let create_info = vk::SwapchainCreateInfoKHR {
                        surface,
                        min_image_count: vulkan_swapchain.capabilities.min_image_count,
                        image_format: vulkan_swapchain.surface_format.format,
                        image_color_space: vulkan_swapchain.surface_format.color_space,
                        image_extent: vk::Extent2d { width, height },
                        image_usage: vulkan_swapchain.usage_flags,
                        image_array_layers: 1,
                        image_sharing_mode: vk::SharingMode::Exclusive,
                        pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
                        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                        present_mode: vulkan_swapchain.present_mode,
                        clipped: vk::Bool32::True,
                        old_swapchain,
                        ..default()
                    };

                    vk_check!(unsafe {
                        self.wsi.swapchain_fn.create_swapchain(
                            self.device,
                            &create_info,
                            None,
                            &mut new_swapchain,
                        )
                    });
                    assert!(!new_swapchain.is_null());

                    let swapchain_images = vk_vec(|count, ptr| unsafe {
                        self.wsi.swapchain_fn.get_swapchain_images(
                            self.device,
                            new_swapchain,
                            count,
                            ptr,
                        )
                    });

                    let image_views = swapchain_images
                        .into_iter()
                        .map(|swapchain_image| {
                            let create_info = vk::ImageViewCreateInfo {
                                image: swapchain_image,
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
                            vk_check!(unsafe {
                                self.device_fn.create_image_view(
                                    self.device,
                                    &create_info,
                                    None,
                                    &mut view,
                                )
                            });

                            let handle = image_pool.insert(VulkanImageHolder::Swapchain(
                                VulkanImageSwapchain {
                                    surface,
                                    image: swapchain_image,
                                    view,
                                },
                            ));

                            Image(handle)
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice();

                    vulkan_swapchain.state = VulkanSwapchainState::Occupied {
                        width,
                        height,
                        swapchain: new_swapchain,
                        image_views,
                    };
                }
                VulkanSwapchainState::Occupied {
                    width: current_width,
                    height: current_height,
                    swapchain,
                    image_views,
                } => {
                    let detach_image_views =
                        |images: &mut Pool<VulkanImageHolder>| -> Box<[vk::ImageView]> {
                            let mut vulkan_image_views = Vec::new();
                            for &image_view in image_views.iter() {
                                match images.remove(image_view.0) {
                                    Some(VulkanImageHolder::Swapchain(VulkanImageSwapchain {
                                        surface: _,
                                        image: _,
                                        view,
                                    })) => vulkan_image_views.push(view),
                                    _ => panic!("swapchain image in wrong state"),
                                }
                            }
                            vulkan_image_views.into_boxed_slice()
                        };

                    let swapchain = *swapchain;

                    if width != *current_width
                        || height != *current_height
                        || suboptimal_swapchains.remove(&swapchain)
                    {
                        let image_views = detach_image_views(&mut image_pool);
                        old_swapchain = swapchain;
                        self.wsi.destroy_swapchains.lock().push(DestroySwapchain {
                            swapchain: old_swapchain,
                            surface: vk::SurfaceKHR::null(),
                            image_views,
                        });

                        vulkan_swapchain.state = VulkanSwapchainState::Vacant;
                        continue;
                    }

                    let acquire = self.request_transient_semaphore(frame);
                    let mut image_index = 0;
                    match unsafe {
                        self.wsi.swapchain_fn.acquire_next_image2(
                            self.device,
                            &vk::AcquireNextImageInfoKHR {
                                swapchain,
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
                            suboptimal_swapchains.insert(swapchain);
                        }
                        vk::Result::ErrorOutOfDateKHR => {
                            let image_views = detach_image_views(&mut image_pool);

                            old_swapchain = swapchain;

                            self.wsi.destroy_swapchains.lock().push(DestroySwapchain {
                                swapchain: old_swapchain,
                                surface: vk::SurfaceKHR::null(),
                                image_views,
                            });

                            vulkan_swapchain.state = VulkanSwapchainState::Vacant;
                            return Err(SwapchainOutOfDateError(()));
                        }
                        result => vk_check!(result),
                    }

                    present_info.acquire = acquire;
                    present_info.image_index = image_index;
                    present_info.swapchain = swapchain;

                    return Ok(SwapchainImage {
                        width,
                        height,
                        image: image_views[image_index.widen()],
                    });
                }
            }
        }
    }

    pub fn destroy_swapchain(&self, window: &dyn AsRawWindow) {
        let raw_window = window.as_raw_window();

        let Some(surface) = self.wsi.surfaces.lock().remove(&raw_window) else {
            return;
        };

        if let Some(VulkanSwapchain {
            surface_format: _,
            present_mode: _,
            usage_flags: _,
            state,
            capabilities: _,
        }) = self.wsi.swapchains.lock().remove(&surface)
        {
            let mut image_pool = self.image_pool.lock();

            if let VulkanSwapchainState::Occupied {
                width: _,
                height: _,
                swapchain,
                image_views,
            } = state
            {
                let mut vulkan_image_views = Vec::new();
                for &image_view in image_views.iter() {
                    match image_pool.remove(image_view.0) {
                        Some(VulkanImageHolder::Swapchain(VulkanImageSwapchain {
                            surface: _,
                            image: _,
                            view,
                        })) => vulkan_image_views.push(view),
                        _ => panic!("swapchain image in wrong state"),
                    }
                }

                self.wsi.destroy_swapchains.lock().push(DestroySwapchain {
                    swapchain,
                    surface,
                    image_views: vulkan_image_views.into_boxed_slice(),
                });
            }
        }
    }

    pub fn touch_swapchain(
        &self,
        frame: &VulkanFrame,
        surface: vk::SurfaceKHR,
        stage_mask: vk::PipelineStageFlags2,
        wait_semaphores: &mut Vec<vk::SemaphoreSubmitInfo>,
        signal_semaphores: &mut Vec<vk::SemaphoreSubmitInfo>,
    ) {
        let mut presents = frame.wsi.presents.lock();
        let present_swapchain = presents
            .get_mut(&surface)
            .expect("presenting a swapchain that hasn't been acquired this frame");

        assert!(
            !present_swapchain.acquire.is_null(),
            "acquiring a swapchain image multiple times"
        );
        present_swapchain.release = self.request_semaphore();

        wait_semaphores.push(vk::SemaphoreSubmitInfo {
            semaphore: present_swapchain.acquire,
            stage_mask,
            ..default()
        });
        signal_semaphores.push(vk::SemaphoreSubmitInfo {
            semaphore: present_swapchain.release,
            stage_mask,
            ..default()
        });
    }

    fn destroy_swapchain_deferred(
        &self,
        surface: vk::SurfaceKHR,
        swapchain: vk::SwapchainKHR,
        image_views: &[vk::ImageView],
    ) {
        let instance = self.instance;
        let device = self.device;

        self.wsi.suboptimal_swapchains.lock().remove(&swapchain);

        if !image_views.is_empty() {
            for &image_view in image_views {
                unsafe { self.device_fn.destroy_image_view(device, image_view, None) }
            }
        }
        if !swapchain.is_null() {
            unsafe {
                self.wsi
                    .swapchain_fn
                    .destroy_swapchain(device, swapchain, None)
            }
        }
        if !surface.is_null() {
            unsafe { self.wsi.surface_fn.destroy_surface(instance, surface, None) }
        }
    }

    pub fn wsi_begin_frame(&self) {
        let mut recycle_swapchain_semaphores = self.wsi.recycle_swapchain_semaphores.lock();

        if self.wsi.support.swapchain_maintenance1 {
            recycle_swapchain_semaphores.retain(|recycle| {
                // With VK_EXT_swapchain_maintenance1 we can just check the fence.
                if unsafe {
                    self.device_fn.get_fence_status(self.device, recycle.fence)
                        == vk::Result::NotReady
                } {
                    return true;
                }

                self.recycled_fences.lock().push_back(recycle.fence);
                self.recycled_semaphores.lock().push_back(recycle.semaphore);

                false
            });
        } else {
            recycle_swapchain_semaphores.retain_mut(|recycle| {
                // Without VK_EXT_swapchain_maintenance1 we use the fence to store a counter.
                // When the counter hits zero we cleanup.
                if !recycle.fence.is_null() {
                    recycle.fence = vk::Fence::from_raw(recycle.fence.as_raw() - 1);
                    return true;
                }

                self.recycled_semaphores.lock().push_back(recycle.semaphore);

                false
            });
        }

        self.wsi.destroy_swapchains.lock().retain(|destroy| {
            let found_associated_semaphore = recycle_swapchain_semaphores
                .iter()
                .any(|recycle| destroy.swapchain == recycle.swapchain);

            if !found_associated_semaphore {
                self.destroy_swapchain_deferred(
                    destroy.surface,
                    destroy.swapchain,
                    &destroy.image_views,
                );
            }

            found_associated_semaphore
        });
    }

    pub fn wsi_end_frame(&self, frame: &mut VulkanFrame) {
        let presents = frame.wsi.presents.get_mut();

        if presents.is_empty() {
            return;
        }

        let arena = HybridArena::<512>::new();

        let presents =
            arena.alloc_slice_fill_iter(presents.drain().map(|(_, present_info)| present_info));

        for present_info in presents.iter() {
            assert!(
                !present_info.release.is_null(),
                "swapchain image was acquired, but not consumed"
            );
        }

        let fences: &[_] = if self.wsi.support.swapchain_maintenance1 {
            arena.alloc_slice_fill_with(presents.len(), |_| self.request_fence())
        } else {
            arena.alloc_slice_fill_with(presents.len(), |_| {
                vk::Fence::from_raw(VULKAN_CONSTANTS.swapchain_semaphore_destroy_delay)
            })
        };
        let wait_semaphores: &[_] = arena.alloc_slice_fill_iter(presents.iter().map(|x| x.release));
        let swapchains: &[_] = arena.alloc_slice_fill_iter(presents.iter().map(|x| x.swapchain));
        let image_indices: &[_] =
            arena.alloc_slice_fill_iter(presents.iter().map(|x| x.image_index));
        let results = arena.alloc_slice_fill_copy(swapchains.len(), vk::Result::Success);

        for i in 0..presents.len() {
            self.wsi
                .recycle_swapchain_semaphores
                .lock()
                .push(RecycleSwapchainSemaphore {
                    fence: fences[i],
                    semaphore: wait_semaphores[i],
                    swapchain: swapchains[i],
                });
        }

        let present_fence_info = vk::SwapchainPresentFenceInfoEXT {
            fences: fences.into(),
            ..default()
        };

        let present_info = vk::PresentInfoKHR {
            _next: if self.wsi.support.swapchain_maintenance1 {
                &present_fence_info as *const _ as *const core::ffi::c_void
            } else {
                core::ptr::null()
            },
            wait_semaphores: wait_semaphores.into(),
            swapchains: (swapchains, image_indices).into(),
            results: results.as_mut_ptr(),
            ..default()
        };

        unsafe {
            // check results below, so ignore this return value.
            let _ = self
                .wsi
                .swapchain_fn
                .queue_present(self.universal_queue, &present_info);
        };

        for (i, &result) in results.iter().enumerate() {
            match result {
                vk::Result::Success => {}
                vk::Result::SuboptimalKHR => {
                    self.wsi.suboptimal_swapchains.lock().insert(swapchains[i]);
                }
                _ => vk_check!(result),
            }
        }
    }

    pub fn wsi_drop(&mut self) {
        for recyle in self.wsi.recycle_swapchain_semaphores.get_mut().drain(..) {
            if self.wsi.support.swapchain_maintenance1 {
                let fences = &[recyle.fence];
                vk_check!(unsafe {
                    self.device_fn
                        .wait_for_fences(self.device, fences, vk::Bool32::True, !0)
                });
                unsafe {
                    self.device_fn
                        .destroy_fence(self.device, recyle.fence, None)
                };
            }

            unsafe {
                self.device_fn
                    .destroy_semaphore(self.device, recyle.semaphore, None);
            }
        }

        let destroyed_swapchains = self
            .wsi
            .destroy_swapchains
            .get_mut()
            .drain(..)
            .collect::<Vec<_>>();

        for destroy in destroyed_swapchains {
            self.destroy_swapchain_deferred(
                destroy.surface,
                destroy.swapchain,
                &destroy.image_views,
            );
        }

        for (&surface, swapchain) in self.wsi.swapchains.get_mut().iter() {
            if let VulkanSwapchainState::Occupied {
                width: _,
                height: _,
                swapchain,
                image_views: _,
            } = swapchain.state
            {
                unsafe {
                    self.wsi
                        .swapchain_fn
                        .destroy_swapchain(self.device, swapchain, None)
                }
            }
            unsafe {
                self.wsi
                    .surface_fn
                    .destroy_surface(self.instance, surface, None)
            }
        }
    }
}
