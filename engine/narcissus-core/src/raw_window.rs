use std::ffi::{c_int, c_void};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct XcbWindow {
    pub connection: *mut c_void,
    pub window: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct XlibWindow {
    pub display: *mut c_void,
    pub window: c_int,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WaylandWindow {
    pub display: *mut c_void,
    pub surface: *mut c_void,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawWindow {
    Xcb(XcbWindow),
    Xlib(XlibWindow),
    Wayland(WaylandWindow),
}

pub trait AsRawWindow {
    fn as_raw_window(&self) -> RawWindow;
}
