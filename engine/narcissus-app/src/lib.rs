mod key;
mod sdl;

use std::rc::Rc;

use narcissus_core::{flags_def, raw_window::AsRawWindow};

pub use key::Key;

flags_def!(ButtonFlags);
impl ButtonFlags {
    pub const LEFT: Self = Self(1 << 0);
    pub const MIDDLE: Self = Self(1 << 1);
    pub const RIGHT: Self = Self(1 << 2);
    pub const X1: Self = Self(1 << 3);
    pub const X2: Self = Self(1 << 4);
}

flags_def!(ModifierFlags);
impl ModifierFlags {
    pub const ALT: Self = Self(1 << 0);
    pub const CTRL: Self = Self(1 << 1);
    pub const SHIFT: Self = Self(1 << 2);
    pub const META: Self = Self(1 << 3);
}

pub struct WindowDesc<'a> {
    pub title: &'a str,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WindowId(u64);

pub trait Window: AsRawWindow {
    fn id(&self) -> WindowId;

    fn size(&self) -> (u32, u32);
    fn size_in_pixels(&self) -> (u32, u32);

    fn display_scale(&self) -> f32;
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[non_exhaustive]
pub enum Event {
    Unknown,
    Quit,

    KeyPress {
        window_id: WindowId,
        key: Key,
        repeat: bool,
        down: bool,
        modifiers: ModifierFlags,
    },

    ButtonPress {
        window_id: WindowId,
        buttons: ButtonFlags,
        down: bool,
    },

    MouseMotion {
        window_id: WindowId,
        x: f32,
        y: f32,
    },

    /// A window has gained mouse focus.
    MouseEnter {
        window_id: WindowId,
        x: i32,
        y: i32,
    },

    /// A window has lost moust focus.
    MouseLeave {
        window_id: WindowId,
        x: i32,
        y: i32,
    },

    /// A window has gained keyboard focus.
    FocusGained {
        window_id: WindowId,
    },

    /// A window has lost keyboard focus.
    FocusLost {
        window_id: WindowId,
    },

    /// A window has been resized.
    ///
    /// `width` contains the new window size in logical pixel units.
    /// `height` contains the new window size in logical pixel units.
    Resize {
        window_id: WindowId,
        width: u32,
        height: u32,
    },

    /// A window has been resized.
    ///
    /// `width` contains the new window size in native pixel units.
    /// `height` contains the new window size in native pixel units.
    ResizePixels {
        window_id: WindowId,
        width: u32,
        height: u32,
    },

    /// A window has changed display scale.
    ScaleChanged {
        window_id: WindowId,
    },

    /// The close button has been pressed on the window.
    CloseRequested {
        window_id: WindowId,
    },

    /// The window has been destroyed.
    Destroy {
        window_id: WindowId,
    },
}

pub trait App {
    fn create_window(&self, desc: &WindowDesc) -> Rc<dyn Window>;
    fn destroy_window(&self, window: Rc<dyn Window>);

    fn window(&self, window_id: WindowId) -> Rc<dyn Window>;

    fn poll_event(&self) -> Option<Event>;
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
