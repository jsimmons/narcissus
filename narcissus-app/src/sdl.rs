use std::{
    collections::HashMap,
    ffi::{c_void, CStr, CString},
    mem::MaybeUninit,
    os::raw::c_char,
};

use crate::{App, Event, Window};

use narcissus_core::{Handle, Mutex, Pool};
use sdl2_sys as sdl;

struct SdlWindow(*mut sdl::Window);

pub struct SdlApp {
    windows: Mutex<Pool<SdlWindow>>,
    window_id_to_handle: Mutex<HashMap<u32, Window>>,
}

impl SdlApp {
    pub fn new() -> Result<Self, ()> {
        unsafe { sdl::SDL_Init(sdl::INIT_VIDEO) };
        Ok(Self {
            windows: Mutex::new(Pool::new()),
            window_id_to_handle: Mutex::new(HashMap::new()),
        })
    }
}

impl Drop for SdlApp {
    fn drop(&mut self) {
        for window in self.windows.get_mut().values() {
            unsafe { sdl::SDL_DestroyWindow(window.0) };
        }
        unsafe { sdl::SDL_Quit() };
    }
}

impl App for SdlApp {
    fn create_window(&self, desc: &crate::WindowDesc) -> Window {
        let title = CString::new(desc.title).unwrap();
        let window = unsafe {
            sdl::SDL_CreateWindow(
                title.as_ptr(),
                0,
                0,
                desc.width as i32,
                desc.height as i32,
                sdl::WINDOW_VULKAN | sdl::WINDOW_SHOWN | sdl::WINDOW_RESIZABLE,
            )
        };
        assert!(!window.is_null());
        let window_id = unsafe { sdl::SDL_GetWindowID(window) };

        let mut window_id_to_handle = self.window_id_to_handle.lock();
        let mut windows = self.windows.lock();

        let handle = Window(windows.insert(SdlWindow(window)));
        window_id_to_handle.insert(window_id, handle);
        handle
    }

    fn destroy_window(&self, window: Window) {
        if let Some(window) = self.windows.lock().remove(window.0) {
            unsafe { sdl::SDL_DestroyWindow(window.0) };
        }
    }

    fn vk_get_loader(&self) -> *mut c_void {
        unsafe {
            sdl::SDL_Vulkan_LoadLibrary(std::ptr::null());
            sdl::SDL_Vulkan_GetVkGetInstanceProcAddr()
        }
    }

    fn vk_instance_extensions(&self) -> Vec<&'static CStr> {
        let mut count: u32 = 0;
        let ret = unsafe {
            sdl::SDL_Vulkan_GetInstanceExtensions(
                std::ptr::null_mut(),
                &mut count,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(ret, 1, "failed to query instance extensions");
        if count == 0 {
            return Vec::new();
        }

        let mut names: Vec<*const c_char> = vec![std::ptr::null(); count as usize];
        let ret = unsafe {
            sdl::SDL_Vulkan_GetInstanceExtensions(
                std::ptr::null_mut(),
                &mut count,
                names.as_mut_ptr(),
            )
        };
        assert_eq!(ret, 1, "failed to query instance extensions");

        names
            .iter()
            .map(|&val| unsafe { CStr::from_ptr(val) })
            .collect()
    }

    fn vk_create_surface(&self, window: Window, instance: u64) -> u64 {
        let windows = self.windows.lock();
        let window = windows.get(window.0).unwrap();
        let mut surface = !0;
        let ret = unsafe { sdl::SDL_Vulkan_CreateSurface(window.0, instance, &mut surface) };
        assert_eq!(ret, 1, "failed to create vulkan surface");
        surface
    }

    fn vk_get_surface_extent(&self, window: Window) -> (u32, u32) {
        let windows = self.windows.lock();
        let window = windows.get(window.0).unwrap();
        let mut w = 0;
        let mut h = 0;
        unsafe {
            sdl::SDL_Vulkan_GetDrawableSize(window.0, &mut w, &mut h);
        }
        (w as u32, h as u32)
    }

    fn poll_event(&self) -> Option<Event> {
        let mut event = MaybeUninit::uninit();
        if unsafe { sdl::SDL_PollEvent(event.as_mut_ptr()) } == 0 {
            return None;
        }

        let event = unsafe { event.assume_init() };
        let e = match unsafe { event.r#type } {
            sdl2_sys::EventType::QUIT => Event::Quit,
            sdl2_sys::EventType::WINDOWEVENT => match unsafe { event.window.event } {
                sdl::WindowEventId::None => Event::Unknown,
                sdl::WindowEventId::Shown => Event::Unknown,
                sdl::WindowEventId::Hidden => Event::Unknown,
                sdl::WindowEventId::Exposed => Event::Unknown,
                sdl::WindowEventId::Moved => Event::Unknown,
                sdl::WindowEventId::Resized => Event::Unknown,
                sdl::WindowEventId::SizeChanged => Event::Unknown,
                sdl::WindowEventId::Minimized => Event::Unknown,
                sdl::WindowEventId::Maximized => Event::Unknown,
                sdl::WindowEventId::Restored => Event::Unknown,
                sdl::WindowEventId::Enter => Event::Unknown,
                sdl::WindowEventId::Leave => Event::Unknown,
                sdl::WindowEventId::FocusGained => Event::Unknown,
                sdl::WindowEventId::FocusLost => Event::Unknown,
                sdl::WindowEventId::Close => {
                    let handle = self
                        .window_id_to_handle
                        .lock()
                        .get(&unsafe { event.window.window_id })
                        .copied()
                        .unwrap_or_else(|| Window(Handle::null()));
                    Event::WindowClose(handle)
                }
                sdl::WindowEventId::TakeFocus => Event::Unknown,
                sdl::WindowEventId::HitTest => Event::Unknown,
                sdl::WindowEventId::IccprofChanged => Event::Unknown,
                sdl::WindowEventId::DisplayChanged => Event::Unknown,
            },
            _ => Event::Unknown,
        };

        Some(e)
    }
}
