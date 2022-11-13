mod button;
mod key;
mod sdl;

use std::ffi::{c_void, CStr};

use narcissus_core::{flags_def, Handle};

pub use button::Button;
pub use key::Key;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PressedState {
    Released,
    Pressed,
}

flags_def!(ModifierFlags);
impl ModifierFlags {
    pub const ALT: Self = Self(1 << 0);
    pub const CTRL: Self = Self(1 << 1);
    pub const SHIFT: Self = Self(1 << 2);
    pub const META: Self = Self(1 << 3);
}

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

    KeyPress {
        window: Window,
        key: Key,
        pressed: PressedState,
        modifiers: ModifierFlags,
    },

    ButtonPress {
        window: Window,
        button: Button,
        pressed: PressedState,
    },

    MouseMotion {
        window: Window,
        x: i32,
        y: i32,
    },

    /// A window has gained mouse focus.
    MouseEnter {
        window: Window,
        x: i32,
        y: i32,
    },

    /// A window has lost moust focus.
    MouseLeave {
        window: Window,
        x: i32,
        y: i32,
    },

    /// A window has gained keyboard focus.
    FocusIn {
        window: Window,
    },

    /// A window has lost keyboard focus.
    FocusOut {
        window: Window,
    },

    /// The window has been resized.
    Resize {
        window: Window,
        width: u32,
        height: u32,
    },

    // The close button has been pressed on the window.
    Close {
        window: Window,
    },

    // The window has been destroyed.
    Destroy {
        window: Window,
    },
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
