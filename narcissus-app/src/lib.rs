mod sdl;

use std::ffi::{c_void, CStr};

use narcissus_core::Handle;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Window(Handle);

impl Window {
    pub const fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

pub struct WindowDesc<'a> {
    pub title: &'a str,
    pub width: u32,
    pub height: u32,
}

#[non_exhaustive]
pub enum Event {
    Unknown,
    Quit,
    WindowClose(Window),
}

pub trait App {
    fn create_window(&self, desc: &WindowDesc) -> Window;
    fn destroy_window(&self, window: Window);

    fn poll_event(&self) -> Option<Event>;

    fn vk_get_loader(&self) -> *mut c_void;
    fn vk_instance_extensions(&self) -> Vec<&'static CStr>;
    fn vk_create_surface(&self, window: Window, instance: u64) -> u64;
    fn vk_get_surface_extent(&self, window: Window) -> (u32, u32);
}

pub fn create_app() -> Box<dyn App> {
    Box::new(sdl::SdlApp::new().unwrap())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
