use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    ffi::CStr,
};

use narcissus_core::{
    cstr, cstr_from_bytes_until_nul, default,
    raw_window::{AsRawWindow, RawWindow},
    HybridArena, Mutex, Pool, Widen,
};
use vulkan_sys as vk;

use crate::{
    backend::vulkan::{vk_vec, vulkan_format, VulkanImageHolder, VulkanImageSwapchain},
    delay_queue::DelayQueue,
    vk_check, Frame, Image, ImageFormat, SwapchainOutOfDateError,
};

use super::{SwapchainDestroyQueue, VulkanDevice, VulkanFrame, VULKAN_CONSTANTS};

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
    surface_format: vk::SurfaceFormatKHR,

    state: VulkanSwapchainState,

    _formats: Box<[vk::SurfaceFormatKHR]>,
    _present_modes: Box<[vk::PresentModeKHR]>,
    capabilities: vk::SurfaceCapabilitiesKHR,
}

#[derive(Default)]
pub struct VulkanWsiSupport {
    wayland: bool,
    xlib: bool,
    xcb: bool,
}

pub struct VulkanWsi {
    surfaces: Mutex<HashMap<RawWindow, vk::SurfaceKHR>>,
    swapchains: Mutex<HashMap<vk::SurfaceKHR, VulkanSwapchain>>,
    suboptimal_swapchains: Mutex<HashSet<vk::SwapchainKHR>>,

    swapchain_destroy_queue: Mutex<SwapchainDestroyQueue>,

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
    ) -> VulkanWsiSupport {
        let mut wsi_support: VulkanWsiSupport = default();

        for extension in extension_properties {
            let extension_name = cstr_from_bytes_until_nul(&extension.extension_name).unwrap();

            match extension_name.to_str().unwrap() {
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
            enabled_extensions.push(cstr!("VK_KHR_surface"));
        }

        wsi_support
    }

    /// Check available WSI device extensions, and append required extensions to
    /// `enabled_extensions`.
    ///
    /// Panics if device does not support required extensions.
    pub fn check_device_extensions<'a>(
        extension_properties: &'a [vk::ExtensionProperties],
        enabled_extensions: &mut Vec<&'a CStr>,
    ) {
        for extension in extension_properties {
            let extension_name = cstr_from_bytes_until_nul(&extension.extension_name).unwrap();
            if extension_name.to_str().unwrap() == "VK_KHR_swapchain" {
                enabled_extensions.push(extension_name);
                return;
            }
        }

        panic!("VK_KHR_swapchain not supported")
    }

    pub fn new(
        global_fn: &vk::GlobalFunctions,
        instance: vk::Instance,
        wsi_support: &VulkanWsiSupport,
    ) -> Self {
        let xcb_surface_fn = if wsi_support.xcb {
            Some(vk::XcbSurfaceKHRFunctions::new(global_fn, instance))
        } else {
            None
        };

        let xlib_surface_fn = if wsi_support.xlib {
            Some(vk::XlibSurfaceKHRFunctions::new(global_fn, instance))
        } else {
            None
        };

        let wayland_surface_fn = if wsi_support.wayland {
            Some(vk::WaylandSurfaceKHRFunctions::new(global_fn, instance))
        } else {
            None
        };

        let surface_fn = vk::SurfaceKHRFunctions::new(global_fn, instance);
        let swapchain_fn = vk::SwapchainKHRFunctions::new(global_fn, instance, vk::VERSION_1_1);

        VulkanWsi {
            surfaces: default(),
            swapchains: default(),
            suboptimal_swapchains: default(),
            swapchain_destroy_queue: Mutex::new(DelayQueue::new(
                VULKAN_CONSTANTS.swapchain_destroy_delay,
            )),
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
        format: ImageFormat,
    ) -> Result<(u32, u32, Image), SwapchainOutOfDateError> {
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
                        .create_xcb_surface(self.instance, &create_info, None, &mut surface,));
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
                        .create_xlib_surface(self.instance, &create_info, None, &mut surface,));
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
                        .create_wayland_surface(self.instance, &create_info, None, &mut surface,));
                    surface
                }
            });

        let format = vulkan_format(format);

        let mut swapchains = self.wsi.swapchains.lock();
        let vulkan_swapchain = swapchains.entry(surface).or_insert_with(|| {
            let mut supported = vk::Bool32::False;
            vk_check!(self.wsi.surface_fn.get_physical_device_surface_support(
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
                self.wsi.surface_fn.get_physical_device_surface_formats(
                    self.physical_device,
                    surface,
                    count,
                    ptr,
                )
            })
            .into_boxed_slice();

            let present_modes = vk_vec(|count, ptr| unsafe {
                self.wsi
                    .surface_fn
                    .get_physical_device_surface_present_modes(
                        self.physical_device,
                        surface,
                        count,
                        ptr,
                    )
            })
            .into_boxed_slice();

            let mut capabilities = vk::SurfaceCapabilitiesKHR::default();
            vk_check!(self
                .wsi
                .surface_fn
                .get_physical_device_surface_capabilities(
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
                surface_format,
                state: VulkanSwapchainState::Vacant,
                _formats: formats,
                _present_modes: present_modes,
                capabilities,
            }
        });

        assert_eq!(format, vulkan_swapchain.surface_format.format);

        let frame = self.frame(frame);
        let mut image_pool = self.image_pool.lock();

        let mut present_swapchains = frame.wsi.presents.lock();
        let present_info = match present_swapchains.entry(surface) {
            Entry::Occupied(_) => panic!("acquiring swapchain multiple times in a frame"),
            Entry::Vacant(entry) => entry.insert(default()),
        };

        vk_check!(self
            .wsi
            .surface_fn
            .get_physical_device_surface_capabilities(
                self.physical_device,
                surface,
                &mut vulkan_swapchain.capabilities
            ));

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
                        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                            | vk::ImageUsageFlags::TRANSFER_SRC
                            | vk::ImageUsageFlags::TRANSFER_DST,
                        image_array_layers: 1,
                        image_sharing_mode: vk::SharingMode::Exclusive,
                        pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
                        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                        present_mode: vk::PresentModeKHR::Fifo,
                        clipped: vk::Bool32::True,
                        old_swapchain,
                        ..default()
                    };
                    vk_check!(self.wsi.swapchain_fn.create_swapchain(
                        self.device,
                        &create_info,
                        None,
                        &mut new_swapchain
                    ));
                    assert!(!new_swapchain.is_null());

                    let images = vk_vec(|count, ptr| unsafe {
                        self.wsi.swapchain_fn.get_swapchain_images(
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

                            let handle = image_pool.insert(VulkanImageHolder::Swapchain(
                                VulkanImageSwapchain {
                                    surface,
                                    image,
                                    view,
                                },
                            ));
                            Image(handle)
                        })
                        .collect::<Box<_>>();

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
                    let destroy_image_views =
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
                        let image_views = destroy_image_views(&mut image_pool);
                        old_swapchain = swapchain;
                        self.wsi.swapchain_destroy_queue.lock().push((
                            old_swapchain,
                            vk::SurfaceKHR::null(),
                            image_views,
                        ));

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
                            let image_views = destroy_image_views(&mut image_pool);

                            old_swapchain = swapchain;
                            self.wsi.swapchain_destroy_queue.lock().push((
                                old_swapchain,
                                vk::SurfaceKHR::null(),
                                image_views,
                            ));

                            vulkan_swapchain.state = VulkanSwapchainState::Vacant;
                            return Err(SwapchainOutOfDateError(()));
                        }
                        result => vk_check!(result),
                    }

                    present_info.acquire = acquire;
                    present_info.image_index = image_index;
                    present_info.swapchain = swapchain;
                    let view = image_views[image_index.widen()];

                    return Ok((width, height, view));
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
            state,
            _formats: _,
            _present_modes: _,
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

                self.wsi.swapchain_destroy_queue.lock().push((
                    swapchain,
                    surface,
                    vulkan_image_views.into_boxed_slice(),
                ));
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
        self.wsi
            .swapchain_destroy_queue
            .lock()
            .expire(|(swapchain, surface, image_views)| {
                self.destroy_swapchain_deferred(surface, swapchain, &image_views);
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

        let wait_semaphores: &[_] = arena.alloc_slice_fill_iter(presents.iter().map(|x| x.release));
        let swapchains: &[_] = arena.alloc_slice_fill_iter(presents.iter().map(|x| x.swapchain));
        let image_indices: &[_] =
            arena.alloc_slice_fill_iter(presents.iter().map(|x| x.image_index));

        let results = arena.alloc_slice_fill_copy(swapchains.len(), vk::Result::Success);

        let present_info = vk::PresentInfoKHR {
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
        let destroyed_swapchains = self
            .wsi
            .swapchain_destroy_queue
            .get_mut()
            .drain(..)
            .collect::<Vec<_>>();
        for (_, (swapchain, surface, image_views)) in destroyed_swapchains {
            self.destroy_swapchain_deferred(surface, swapchain, &image_views);
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
