use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    mem::MaybeUninit,
    rc::Rc,
};

use crate::{App, ButtonFlags, Event, Key, ModifierFlags, Window, WindowId};

use narcissus_core::{
    raw_window::{AsRawWindow, RawWindow, WaylandWindow, XlibWindow},
    Mutex, Upcast,
};
use sdl3_sys::{
    events::{
        SDL_EventType, SDL_PollEvent, SDL_EVENT_KEY_DOWN, SDL_EVENT_KEY_UP,
        SDL_EVENT_MOUSE_BUTTON_DOWN, SDL_EVENT_MOUSE_BUTTON_UP, SDL_EVENT_MOUSE_MOTION,
        SDL_EVENT_QUIT, SDL_EVENT_WINDOW_CLOSE_REQUESTED, SDL_EVENT_WINDOW_DISPLAY_SCALE_CHANGED,
        SDL_EVENT_WINDOW_FOCUS_GAINED, SDL_EVENT_WINDOW_FOCUS_LOST, SDL_EVENT_WINDOW_MOUSE_ENTER,
        SDL_EVENT_WINDOW_MOUSE_LEAVE, SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED,
        SDL_EVENT_WINDOW_RESIZED,
    },
    init::{SDL_InitSubSystem, SDL_Quit, SDL_INIT_VIDEO},
    keycode::*,
    mouse::{
        SDL_BUTTON_LMASK, SDL_BUTTON_MMASK, SDL_BUTTON_RMASK, SDL_BUTTON_X1MASK, SDL_BUTTON_X2MASK,
    },
    properties::{SDL_GetNumberProperty, SDL_GetPointerProperty},
    scancode::*,
    video::{
        SDL_CreateWindow, SDL_DestroyWindow, SDL_GetCurrentVideoDriver, SDL_GetWindowDisplayScale,
        SDL_GetWindowID, SDL_GetWindowProperties, SDL_GetWindowSize, SDL_GetWindowSizeInPixels,
        SDL_Window, SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER,
        SDL_PROP_WINDOW_WAYLAND_SURFACE_POINTER, SDL_PROP_WINDOW_X11_DISPLAY_POINTER,
        SDL_PROP_WINDOW_X11_WINDOW_NUMBER, SDL_WINDOW_HIGH_PIXEL_DENSITY, SDL_WINDOW_RESIZABLE,
        SDL_WINDOW_VULKAN,
    },
};

fn sdl_window_id(window_id: u32) -> WindowId {
    WindowId(window_id as u64)
}

struct SdlWindow {
    window: *mut SDL_Window,
}

impl Window for SdlWindow {
    fn id(&self) -> WindowId {
        sdl_window_id(unsafe { SDL_GetWindowID(self.window) })
    }

    fn size(&self) -> (u32, u32) {
        let mut width = 0;
        let mut height = 0;
        if !unsafe { SDL_GetWindowSize(self.window, &mut width, &mut height) } {
            #[cfg(debug_assertions)]
            panic!("failed to retreive window size");
        }
        (width as u32, height as u32)
    }

    fn size_in_pixels(&self) -> (u32, u32) {
        let mut width = 0;
        let mut height = 0;
        if !unsafe { SDL_GetWindowSizeInPixels(self.window, &mut width, &mut height) } {
            #[cfg(debug_assertions)]
            panic!("failed to retreive window size in pixels");
        }
        (width as u32, height as u32)
    }

    fn display_scale(&self) -> f32 {
        unsafe { SDL_GetWindowDisplayScale(self.window) }
    }
}

impl AsRawWindow for SdlWindow {
    fn as_raw_window(&self) -> RawWindow {
        let properties = unsafe { SDL_GetWindowProperties(self.window) };

        #[cfg(target_os = "linux")]
        {
            let current_video_driver = unsafe { SDL_GetCurrentVideoDriver() };
            assert_ne!(
                current_video_driver,
                core::ptr::null(),
                "no video driver initialized"
            );

            // Safety: null-checked above, SDL ensures return value is a null-terminated ascii string.
            let current_video_driver = unsafe {
                CStr::from_ptr(current_video_driver)
                    .to_str()
                    .expect("invalid video driver")
            };

            match current_video_driver {
                "wayland" => {
                    let display = unsafe {
                        SDL_GetPointerProperty(
                            properties,
                            SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER,
                            core::ptr::null_mut(),
                        )
                    };
                    let surface = unsafe {
                        SDL_GetPointerProperty(
                            properties,
                            SDL_PROP_WINDOW_WAYLAND_SURFACE_POINTER,
                            core::ptr::null_mut(),
                        )
                    };
                    RawWindow::Wayland(WaylandWindow { display, surface })
                }
                "x11" => {
                    let display = unsafe {
                        SDL_GetPointerProperty(
                            properties,
                            SDL_PROP_WINDOW_X11_DISPLAY_POINTER,
                            core::ptr::null_mut(),
                        )
                    };
                    let window = unsafe {
                        SDL_GetNumberProperty(properties, SDL_PROP_WINDOW_X11_WINDOW_NUMBER, 0)
                    } as i32;
                    RawWindow::Xlib(XlibWindow { display, window })
                }
                _ => {
                    panic!("unknown sdl video driver")
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        panic!("unsupported os")
    }
}

impl Upcast<dyn AsRawWindow> for SdlWindow {
    fn upcast(&self) -> &(dyn AsRawWindow + 'static) {
        self
    }
}

pub struct SdlApp {
    windows: Mutex<HashMap<WindowId, Rc<SdlWindow>>>,
}

impl SdlApp {
    pub fn new() -> Result<Self, ()> {
        if !unsafe { SDL_InitSubSystem(SDL_INIT_VIDEO) } {
            panic!("failed to initalize sdl");
        }

        Ok(Self {
            windows: Mutex::new(HashMap::new()),
        })
    }
}

impl Drop for SdlApp {
    fn drop(&mut self) {
        for window in self.windows.get_mut().values() {
            unsafe { SDL_DestroyWindow(window.window) };
        }
        unsafe { SDL_Quit() };
    }
}

impl App for SdlApp {
    fn create_window(&self, desc: &crate::WindowDesc) -> Rc<dyn Window> {
        let title = CString::new(desc.title).unwrap();
        let window = unsafe {
            SDL_CreateWindow(
                title.as_ptr(),
                desc.width as i32,
                desc.height as i32,
                SDL_WINDOW_VULKAN | SDL_WINDOW_HIGH_PIXEL_DENSITY | SDL_WINDOW_RESIZABLE,
            )
        };
        assert!(!window.is_null());
        let window_id = WindowId(unsafe { SDL_GetWindowID(window) } as u64);
        let window = Rc::new(SdlWindow { window });
        self.windows.lock().insert(window_id, window.clone());
        window
    }

    fn destroy_window(&self, window: Rc<dyn Window>) {
        let window_id = window.id();
        drop(window);
        if let Some(mut window) = self.windows.lock().remove(&window_id) {
            let window = Rc::get_mut(&mut window)
                .expect("tried to destroy a window while there are outstanding references");
            unsafe { SDL_DestroyWindow(window.window) };
        }
    }

    fn poll_event(&self) -> Option<Event> {
        let event = unsafe {
            let mut event = MaybeUninit::uninit();
            if !SDL_PollEvent(event.as_mut_ptr()) {
                return None;
            }
            event.assume_init()
        };

        let e = match SDL_EventType(unsafe { event.r#type }) {
            SDL_EVENT_QUIT => Event::Quit,
            SDL_EVENT_WINDOW_RESIZED => Event::Resize {
                window_id: sdl_window_id(unsafe { event.window.windowID }),
                width: unsafe { event.window.data1 } as u32,
                height: unsafe { event.window.data2 } as u32,
            },
            SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED => Event::ResizePixels {
                window_id: sdl_window_id(unsafe { event.window.windowID }),
                width: unsafe { event.window.data1 } as u32,
                height: unsafe { event.window.data2 } as u32,
            },
            SDL_EVENT_WINDOW_DISPLAY_SCALE_CHANGED => Event::ScaleChanged {
                window_id: sdl_window_id(unsafe { event.window.windowID }),
            },
            SDL_EVENT_WINDOW_MOUSE_ENTER => Event::MouseEnter {
                window_id: sdl_window_id(unsafe { event.window.windowID }),
                x: unsafe { event.window.data1 },
                y: unsafe { event.window.data2 },
            },
            SDL_EVENT_WINDOW_MOUSE_LEAVE => Event::MouseLeave {
                window_id: sdl_window_id(unsafe { event.window.windowID }),
                x: unsafe { event.window.data1 },
                y: unsafe { event.window.data2 },
            },
            SDL_EVENT_WINDOW_FOCUS_GAINED => Event::FocusGained {
                window_id: sdl_window_id(unsafe { event.window.windowID }),
            },
            SDL_EVENT_WINDOW_FOCUS_LOST => Event::FocusLost {
                window_id: sdl_window_id(unsafe { event.window.windowID }),
            },
            SDL_EVENT_WINDOW_CLOSE_REQUESTED => Event::CloseRequested {
                window_id: sdl_window_id(unsafe { event.window.windowID }),
            },
            SDL_EVENT_KEY_UP | SDL_EVENT_KEY_DOWN => {
                let scancode = unsafe { event.key.scancode };
                let modifiers = unsafe { event.key.r#mod };
                let repeat = unsafe { event.key.repeat };
                let down = unsafe { event.key.down };
                let key = map_sdl_scancode(scancode);
                let modifiers = map_sdl_modifiers(modifiers);
                Event::KeyPress {
                    window_id: sdl_window_id(unsafe { event.window.windowID }),
                    key,
                    repeat,
                    down,
                    modifiers,
                }
            }
            SDL_EVENT_MOUSE_BUTTON_UP | SDL_EVENT_MOUSE_BUTTON_DOWN => {
                let button = unsafe { event.button.button };
                let down = unsafe { event.button.down };
                let buttons = map_sdl_buttons(button as u32);
                Event::ButtonPress {
                    window_id: sdl_window_id(unsafe { event.window.windowID }),
                    buttons,
                    down,
                }
            }
            SDL_EVENT_MOUSE_MOTION => Event::MouseMotion {
                window_id: sdl_window_id(unsafe { event.motion.windowID }),
                x: unsafe { event.motion.x },
                y: unsafe { event.motion.y },
            },
            _ => Event::Unknown,
        };

        Some(e)
    }

    fn window(&self, window_id: WindowId) -> Rc<dyn Window> {
        self.windows.lock().get(&window_id).unwrap().clone()
    }
}

fn map_sdl_buttons(buttons: u32) -> ButtonFlags {
    let mut flags = ButtonFlags::default();
    if buttons & SDL_BUTTON_LMASK != 0 {
        flags |= ButtonFlags::LEFT
    }
    if buttons & SDL_BUTTON_MMASK != 0 {
        flags |= ButtonFlags::MIDDLE
    }
    if buttons & SDL_BUTTON_RMASK != 0 {
        flags |= ButtonFlags::RIGHT
    }
    if buttons & SDL_BUTTON_X1MASK != 0 {
        flags |= ButtonFlags::X1
    }
    if buttons & SDL_BUTTON_X2MASK != 0 {
        flags |= ButtonFlags::X2
    }
    flags
}

fn map_sdl_modifiers(modifiers: SDL_Keymod) -> ModifierFlags {
    let mut flags = ModifierFlags::default();
    if modifiers & SDL_KMOD_ALT != 0 {
        flags |= ModifierFlags::ALT
    }
    if modifiers & SDL_KMOD_SHIFT != 0 {
        flags |= ModifierFlags::SHIFT
    }
    if modifiers & SDL_KMOD_CTRL != 0 {
        flags |= ModifierFlags::CTRL
    }
    if modifiers & SDL_KMOD_GUI != 0 {
        flags |= ModifierFlags::META
    }
    flags
}

fn map_sdl_scancode(scancode: SDL_Scancode) -> Key {
    match scancode {
        SDL_SCANCODE_A => Key::A,
        SDL_SCANCODE_B => Key::B,
        SDL_SCANCODE_C => Key::C,
        SDL_SCANCODE_D => Key::D,
        SDL_SCANCODE_E => Key::E,
        SDL_SCANCODE_F => Key::F,
        SDL_SCANCODE_G => Key::G,
        SDL_SCANCODE_H => Key::H,
        SDL_SCANCODE_I => Key::I,
        SDL_SCANCODE_J => Key::J,
        SDL_SCANCODE_K => Key::K,
        SDL_SCANCODE_L => Key::L,
        SDL_SCANCODE_M => Key::M,
        SDL_SCANCODE_N => Key::N,
        SDL_SCANCODE_O => Key::O,
        SDL_SCANCODE_P => Key::P,
        SDL_SCANCODE_Q => Key::Q,
        SDL_SCANCODE_R => Key::R,
        SDL_SCANCODE_S => Key::S,
        SDL_SCANCODE_T => Key::T,
        SDL_SCANCODE_U => Key::U,
        SDL_SCANCODE_V => Key::V,
        SDL_SCANCODE_W => Key::W,
        SDL_SCANCODE_X => Key::X,
        SDL_SCANCODE_Y => Key::Y,
        SDL_SCANCODE_Z => Key::Z,

        SDL_SCANCODE_1 => Key::Key1,
        SDL_SCANCODE_2 => Key::Key2,
        SDL_SCANCODE_3 => Key::Key3,
        SDL_SCANCODE_4 => Key::Key4,
        SDL_SCANCODE_5 => Key::Key5,
        SDL_SCANCODE_6 => Key::Key6,
        SDL_SCANCODE_7 => Key::Key7,
        SDL_SCANCODE_8 => Key::Key8,
        SDL_SCANCODE_9 => Key::Key9,
        SDL_SCANCODE_0 => Key::Key0,

        SDL_SCANCODE_RETURN => Key::Return,
        SDL_SCANCODE_ESCAPE => Key::Escape,
        SDL_SCANCODE_BACKSPACE => Key::Backspace,
        SDL_SCANCODE_DELETE => Key::Delete,
        SDL_SCANCODE_TAB => Key::Tab,
        SDL_SCANCODE_SPACE => Key::Space,
        SDL_SCANCODE_MINUS => Key::Minus,
        SDL_SCANCODE_EQUALS => Key::Equal,
        SDL_SCANCODE_LEFTBRACKET => Key::LeftBrace,
        SDL_SCANCODE_RIGHTBRACKET => Key::RightBrace,
        SDL_SCANCODE_BACKSLASH => Key::Backslash,
        SDL_SCANCODE_SEMICOLON => Key::Semicolon,
        SDL_SCANCODE_APOSTROPHE => Key::Apostrophe,
        SDL_SCANCODE_GRAVE => Key::Grave,
        SDL_SCANCODE_COMMA => Key::Comma,
        SDL_SCANCODE_PERIOD => Key::Period,
        SDL_SCANCODE_SLASH => Key::Slash,
        SDL_SCANCODE_CAPSLOCK => Key::CapsLock,

        SDL_SCANCODE_F1 => Key::F1,
        SDL_SCANCODE_F2 => Key::F2,
        SDL_SCANCODE_F3 => Key::F3,
        SDL_SCANCODE_F4 => Key::F4,
        SDL_SCANCODE_F5 => Key::F5,
        SDL_SCANCODE_F6 => Key::F6,
        SDL_SCANCODE_F7 => Key::F7,
        SDL_SCANCODE_F8 => Key::F8,
        SDL_SCANCODE_F9 => Key::F9,
        SDL_SCANCODE_F10 => Key::F10,
        SDL_SCANCODE_F11 => Key::F11,
        SDL_SCANCODE_F12 => Key::F12,
        SDL_SCANCODE_F13 => Key::F13,
        SDL_SCANCODE_F14 => Key::F14,
        SDL_SCANCODE_F15 => Key::F15,
        SDL_SCANCODE_F16 => Key::F16,
        SDL_SCANCODE_F17 => Key::F17,
        SDL_SCANCODE_F18 => Key::F18,
        SDL_SCANCODE_F19 => Key::F19,
        SDL_SCANCODE_F20 => Key::F20,
        SDL_SCANCODE_F21 => Key::F21,
        SDL_SCANCODE_F22 => Key::F22,
        SDL_SCANCODE_F23 => Key::F23,
        SDL_SCANCODE_F24 => Key::F24,

        SDL_SCANCODE_SCROLLLOCK => Key::ScrollLock,
        SDL_SCANCODE_INSERT => Key::Insert,
        SDL_SCANCODE_HOME => Key::Home,
        SDL_SCANCODE_END => Key::End,
        SDL_SCANCODE_PAGEUP => Key::PageUp,
        SDL_SCANCODE_PAGEDOWN => Key::PageDown,

        SDL_SCANCODE_LEFT => Key::Left,
        SDL_SCANCODE_RIGHT => Key::Right,
        SDL_SCANCODE_UP => Key::Up,
        SDL_SCANCODE_DOWN => Key::Down,

        SDL_SCANCODE_NUMLOCKCLEAR => Key::NumLock,
        SDL_SCANCODE_KP_DIVIDE => Key::NumpadDivide,
        SDL_SCANCODE_KP_MULTIPLY => Key::NumpadMultiply,
        SDL_SCANCODE_KP_MINUS => Key::NumpadSubtract,
        SDL_SCANCODE_KP_PLUS => Key::NumpadAdd,
        SDL_SCANCODE_KP_ENTER => Key::NumpadEnter,
        SDL_SCANCODE_KP_1 => Key::Numpad1,
        SDL_SCANCODE_KP_2 => Key::Numpad2,
        SDL_SCANCODE_KP_3 => Key::Numpad3,
        SDL_SCANCODE_KP_4 => Key::Numpad4,
        SDL_SCANCODE_KP_5 => Key::Numpad5,
        SDL_SCANCODE_KP_6 => Key::Numpad6,
        SDL_SCANCODE_KP_7 => Key::Numpad7,
        SDL_SCANCODE_KP_8 => Key::Numpad8,
        SDL_SCANCODE_KP_9 => Key::Numpad9,
        SDL_SCANCODE_KP_0 => Key::Numpad0,
        SDL_SCANCODE_KP_PERIOD => Key::NumpadPeriod,
        SDL_SCANCODE_KP_EQUALS => Key::NumpadEquals,
        SDL_SCANCODE_KP_LEFTPAREN => Key::NumpadLeftParen,
        SDL_SCANCODE_KP_RIGHTPAREN => Key::NumpadRightParen,
        SDL_SCANCODE_KP_PLUSMINUS => Key::NumpadPlusMinus,
        SDL_SCANCODE_KP_COMMA => Key::NumpadComma,

        SDL_SCANCODE_MEDIA_EJECT => Key::MediaEject,
        SDL_SCANCODE_STOP => Key::Stop,
        SDL_SCANCODE_MUTE => Key::Mute,
        SDL_SCANCODE_VOLUMEUP => Key::VolumeUp,
        SDL_SCANCODE_VOLUMEDOWN => Key::VolumeDown,
        SDL_SCANCODE_POWER => Key::Power,

        SDL_SCANCODE_APPLICATION => Key::Compose,
        SDL_SCANCODE_SLEEP => Key::Sleep,

        SDL_SCANCODE_LSHIFT => Key::LeftShift,
        SDL_SCANCODE_RSHIFT => Key::RightShift,
        SDL_SCANCODE_LCTRL => Key::LeftControl,
        SDL_SCANCODE_RCTRL => Key::RightControl,
        SDL_SCANCODE_LALT => Key::LeftAlt,
        SDL_SCANCODE_RALT => Key::RightAlt,
        SDL_SCANCODE_LGUI => Key::LeftMeta,
        SDL_SCANCODE_RGUI => Key::RightMeta,

        SDL_SCANCODE_MENU => Key::Menu,
        SDL_SCANCODE_PAUSE => Key::Pause,

        SDL_SCANCODE_NONUSBACKSLASH => Key::NonUSBackslash,
        SDL_SCANCODE_SYSREQ => Key::SysReq,
        SDL_SCANCODE_AGAIN => Key::Again,
        SDL_SCANCODE_UNDO => Key::Undo,
        SDL_SCANCODE_COPY => Key::Copy,
        SDL_SCANCODE_PASTE => Key::Paste,
        SDL_SCANCODE_FIND => Key::Find,
        SDL_SCANCODE_CUT => Key::Cut,
        SDL_SCANCODE_HELP => Key::Help,
        SDL_SCANCODE_ALTERASE => Key::AltErase,
        SDL_SCANCODE_CANCEL => Key::Cancel,

        SDL_SCANCODE_AC_BOOKMARKS => Key::ACBookmarks,
        SDL_SCANCODE_AC_BACK => Key::ACBack,
        SDL_SCANCODE_AC_FORWARD => Key::ACForward,
        SDL_SCANCODE_AC_HOME => Key::ACHome,
        SDL_SCANCODE_AC_REFRESH => Key::ACRefresh,
        SDL_SCANCODE_AC_SEARCH => Key::ACSearch,

        SDL_SCANCODE_MEDIA_NEXT_TRACK => Key::MediaNextTrack,
        SDL_SCANCODE_MEDIA_PLAY => Key::MediaPlay,
        SDL_SCANCODE_MEDIA_PREVIOUS_TRACK => Key::MediaPreviousTrack,
        SDL_SCANCODE_MEDIA_STOP => Key::MediaStop,
        SDL_SCANCODE_MEDIA_REWIND => Key::MediaRewind,
        SDL_SCANCODE_MEDIA_FAST_FORWARD => Key::MediaFastForward,

        SDL_SCANCODE_LANG1 => Key::Language1,
        SDL_SCANCODE_LANG2 => Key::Language2,
        SDL_SCANCODE_LANG3 => Key::Language3,
        SDL_SCANCODE_LANG4 => Key::Language4,
        SDL_SCANCODE_LANG5 => Key::Language5,

        SDL_SCANCODE_INTERNATIONAL1 => Key::International1,
        SDL_SCANCODE_INTERNATIONAL2 => Key::International2,
        SDL_SCANCODE_INTERNATIONAL3 => Key::International3,
        SDL_SCANCODE_INTERNATIONAL4 => Key::International4,
        SDL_SCANCODE_INTERNATIONAL5 => Key::International5,

        _ => Key::Unknown,
    }
}
