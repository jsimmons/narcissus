use std::{
    collections::HashMap,
    ffi::{c_void, CStr, CString},
    mem::MaybeUninit,
    os::raw::c_char,
};

use crate::{App, Button, Event, Key, ModifierFlags, PressedState, Window};

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

    fn window_from_window_id(&self, window_id: u32) -> Window {
        self.window_id_to_handle
            .lock()
            .get(&window_id)
            .copied()
            .unwrap_or_else(|| Window(Handle::null()))
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
        assert_eq!(ret, sdl::Bool::True, "failed to create vulkan surface");
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
            sdl::EventType::QUIT => Event::Quit,
            sdl::EventType::WINDOWEVENT => match unsafe { event.window.event } {
                sdl::WindowEventId::None => Event::Unknown,
                sdl::WindowEventId::Shown => Event::Unknown,
                sdl::WindowEventId::Hidden => Event::Unknown,
                sdl::WindowEventId::Exposed => Event::Unknown,
                sdl::WindowEventId::Moved => Event::Unknown,
                sdl::WindowEventId::Resized => {
                    let handle = self.window_from_window_id(unsafe { event.window.window_id });
                    Event::Resize {
                        window: handle,
                        width: unsafe { event.window.data1 } as u32,
                        height: unsafe { event.window.data2 } as u32,
                    }
                }
                sdl::WindowEventId::SizeChanged => Event::Unknown,
                sdl::WindowEventId::Minimized => Event::Unknown,
                sdl::WindowEventId::Maximized => Event::Unknown,
                sdl::WindowEventId::Restored => Event::Unknown,
                sdl::WindowEventId::Enter => {
                    let handle = self.window_from_window_id(unsafe { event.window.window_id });
                    Event::MouseEnter {
                        window: handle,
                        x: unsafe { event.window.data1 },
                        y: unsafe { event.window.data2 },
                    }
                }
                sdl::WindowEventId::Leave => {
                    let handle = self.window_from_window_id(unsafe { event.window.window_id });
                    Event::MouseLeave {
                        window: handle,
                        x: unsafe { event.window.data1 },
                        y: unsafe { event.window.data2 },
                    }
                }
                sdl::WindowEventId::FocusGained => {
                    let handle = self.window_from_window_id(unsafe { event.window.window_id });
                    Event::FocusIn { window: handle }
                }
                sdl::WindowEventId::FocusLost => {
                    let handle = self.window_from_window_id(unsafe { event.window.window_id });
                    Event::FocusOut { window: handle }
                }
                sdl::WindowEventId::Close => {
                    let handle = self.window_from_window_id(unsafe { event.window.window_id });
                    Event::Close { window: handle }
                }
                sdl::WindowEventId::TakeFocus => Event::Unknown,
                sdl::WindowEventId::HitTest => Event::Unknown,
                sdl::WindowEventId::IccprofChanged => Event::Unknown,
                sdl::WindowEventId::DisplayChanged => Event::Unknown,
            },
            sdl::EventType::KEYUP | sdl::EventType::KEYDOWN => {
                let handle = self.window_from_window_id(unsafe { event.key.window_id });
                let scancode = unsafe { event.key.keysym.scancode };
                let modifiers = unsafe { event.key.keysym.modifiers };
                let state = unsafe { event.key.state };
                let key = map_sdl_scancode(scancode);
                let modifiers = map_sdl_modifiers(modifiers);
                let pressed = map_sdl_pressed_state(state);
                Event::KeyPress {
                    window: handle,
                    key,
                    pressed,
                    modifiers,
                }
            }
            sdl::EventType::MOUSEBUTTONUP | sdl::EventType::MOUSEBUTTONDOWN => {
                let handle = self.window_from_window_id(unsafe { event.button.window_id });
                let button = unsafe { event.button.button };
                let state = unsafe { event.button.state };
                let button = map_sdl_button(button);
                let pressed = map_sdl_pressed_state(state);
                Event::ButtonPress {
                    window: handle,
                    button,
                    pressed,
                }
            }
            sdl::EventType::MOUSEMOTION => {
                let handle = self.window_from_window_id(unsafe { event.window.window_id });
                Event::MouseMotion {
                    window: handle,
                    x: unsafe { event.window.data1 },
                    y: unsafe { event.window.data2 },
                }
            }
            _ => Event::Unknown,
        };

        Some(e)
    }
}

fn map_sdl_button(button: sdl::MouseButton) -> Button {
    match button {
        sdl::MouseButton::Left => Button::Left,
        sdl::MouseButton::Middle => Button::Middle,
        sdl::MouseButton::Right => Button::Right,
        sdl::MouseButton::X1 => Button::X1,
        sdl::MouseButton::X2 => Button::X2,
    }
}

fn map_sdl_pressed_state(pressed_state: sdl::PressedState) -> PressedState {
    match pressed_state {
        sdl::PressedState::Released => PressedState::Released,
        sdl::PressedState::Pressed => PressedState::Pressed,
    }
}

fn map_sdl_modifiers(modifiers: sdl::Keymod) -> ModifierFlags {
    let mut flags = ModifierFlags::default();
    if modifiers.0 & sdl::Keymod::ALT.0 != 0 {
        flags &= ModifierFlags::ALT
    }
    if modifiers.0 & sdl::Keymod::SHIFT.0 != 0 {
        flags &= ModifierFlags::SHIFT
    }
    if modifiers.0 & sdl::Keymod::CTRL.0 != 0 {
        flags &= ModifierFlags::CTRL
    }
    if modifiers.0 & sdl::Keymod::GUI.0 != 0 {
        flags &= ModifierFlags::META
    }
    flags
}

fn map_sdl_scancode(scancode: sdl::Scancode) -> Key {
    match scancode {
        sdl::Scancode::A => Key::A,
        sdl::Scancode::B => Key::B,
        sdl::Scancode::C => Key::C,
        sdl::Scancode::D => Key::D,
        sdl::Scancode::E => Key::E,
        sdl::Scancode::F => Key::F,
        sdl::Scancode::G => Key::G,
        sdl::Scancode::H => Key::H,
        sdl::Scancode::I => Key::I,
        sdl::Scancode::J => Key::J,
        sdl::Scancode::K => Key::K,
        sdl::Scancode::L => Key::L,
        sdl::Scancode::M => Key::M,
        sdl::Scancode::N => Key::N,
        sdl::Scancode::O => Key::O,
        sdl::Scancode::P => Key::P,
        sdl::Scancode::Q => Key::Q,
        sdl::Scancode::R => Key::R,
        sdl::Scancode::S => Key::S,
        sdl::Scancode::T => Key::T,
        sdl::Scancode::U => Key::U,
        sdl::Scancode::V => Key::V,
        sdl::Scancode::W => Key::W,
        sdl::Scancode::X => Key::X,
        sdl::Scancode::Y => Key::Y,
        sdl::Scancode::Z => Key::Z,

        sdl::Scancode::SCANCODE_1 => Key::Key1,
        sdl::Scancode::SCANCODE_2 => Key::Key2,
        sdl::Scancode::SCANCODE_3 => Key::Key3,
        sdl::Scancode::SCANCODE_4 => Key::Key4,
        sdl::Scancode::SCANCODE_5 => Key::Key5,
        sdl::Scancode::SCANCODE_6 => Key::Key6,
        sdl::Scancode::SCANCODE_7 => Key::Key7,
        sdl::Scancode::SCANCODE_8 => Key::Key8,
        sdl::Scancode::SCANCODE_9 => Key::Key9,
        sdl::Scancode::SCANCODE_0 => Key::Key0,

        sdl::Scancode::RETURN => Key::Return,
        sdl::Scancode::ESCAPE => Key::Escape,
        sdl::Scancode::BACKSPACE => Key::Backspace,
        sdl::Scancode::DELETE => Key::Delete,
        sdl::Scancode::TAB => Key::Tab,
        sdl::Scancode::SPACE => Key::Space,
        sdl::Scancode::MINUS => Key::Minus,
        sdl::Scancode::EQUALS => Key::Equal,
        sdl::Scancode::LEFTBRACKET => Key::LeftBrace,
        sdl::Scancode::RIGHTBRACKET => Key::RightBrace,
        sdl::Scancode::BACKSLASH => Key::Backslash,
        sdl::Scancode::SEMICOLON => Key::Semicolon,
        sdl::Scancode::APOSTROPHE => Key::Apostrophe,
        sdl::Scancode::GRAVE => Key::Grave,
        sdl::Scancode::COMMA => Key::Comma,
        sdl::Scancode::PERIOD => Key::Period,
        sdl::Scancode::SLASH => Key::Slash,
        sdl::Scancode::CAPSLOCK => Key::CapsLock,

        sdl::Scancode::F1 => Key::F1,
        sdl::Scancode::F2 => Key::F2,
        sdl::Scancode::F3 => Key::F3,
        sdl::Scancode::F4 => Key::F4,
        sdl::Scancode::F5 => Key::F5,
        sdl::Scancode::F6 => Key::F6,
        sdl::Scancode::F7 => Key::F7,
        sdl::Scancode::F8 => Key::F8,
        sdl::Scancode::F9 => Key::F9,
        sdl::Scancode::F10 => Key::F10,
        sdl::Scancode::F11 => Key::F11,
        sdl::Scancode::F12 => Key::F12,
        sdl::Scancode::F13 => Key::F13,
        sdl::Scancode::F14 => Key::F14,
        sdl::Scancode::F15 => Key::F15,
        sdl::Scancode::F16 => Key::F16,
        sdl::Scancode::F17 => Key::F17,
        sdl::Scancode::F18 => Key::F18,
        sdl::Scancode::F19 => Key::F19,
        sdl::Scancode::F20 => Key::F20,
        sdl::Scancode::F21 => Key::F21,
        sdl::Scancode::F22 => Key::F22,
        sdl::Scancode::F23 => Key::F23,
        sdl::Scancode::F24 => Key::F24,

        sdl::Scancode::SCROLLLOCK => Key::ScrollLock,
        sdl::Scancode::INSERT => Key::Insert,
        sdl::Scancode::HOME => Key::Home,
        sdl::Scancode::END => Key::End,
        sdl::Scancode::PAGEUP => Key::PageUp,
        sdl::Scancode::PAGEDOWN => Key::PageDown,

        sdl::Scancode::LEFT => Key::Left,
        sdl::Scancode::RIGHT => Key::Right,
        sdl::Scancode::UP => Key::Up,
        sdl::Scancode::DOWN => Key::Down,

        sdl::Scancode::NUMLOCKCLEAR => Key::NumLock,
        sdl::Scancode::KP_DIVIDE => Key::NumpadDivide,
        sdl::Scancode::KP_MULTIPLY => Key::NumpadMultiply,
        sdl::Scancode::KP_MINUS => Key::NumpadSubtract,
        sdl::Scancode::KP_PLUS => Key::NumpadAdd,
        sdl::Scancode::KP_ENTER => Key::NumpadEnter,
        sdl::Scancode::KP_1 => Key::Numpad1,
        sdl::Scancode::KP_2 => Key::Numpad2,
        sdl::Scancode::KP_3 => Key::Numpad3,
        sdl::Scancode::KP_4 => Key::Numpad4,
        sdl::Scancode::KP_5 => Key::Numpad5,
        sdl::Scancode::KP_6 => Key::Numpad6,
        sdl::Scancode::KP_7 => Key::Numpad7,
        sdl::Scancode::KP_8 => Key::Numpad8,
        sdl::Scancode::KP_9 => Key::Numpad9,
        sdl::Scancode::KP_0 => Key::Numpad0,
        sdl::Scancode::KP_PERIOD => Key::NumpadPeriod,
        sdl::Scancode::KP_EQUALS => Key::NumpadEquals,
        sdl::Scancode::KP_LEFTPAREN => Key::NumpadLeftParen,
        sdl::Scancode::KP_RIGHTPAREN => Key::NumpadRightParen,
        sdl::Scancode::KP_PLUSMINUS => Key::NumpadPlusMinus,
        sdl::Scancode::KP_COMMA => Key::NumpadComma,

        sdl::Scancode::EJECT => Key::Eject,
        sdl::Scancode::STOP => Key::Stop,
        sdl::Scancode::MUTE => Key::Mute,
        sdl::Scancode::VOLUMEUP => Key::VolumeUp,
        sdl::Scancode::VOLUMEDOWN => Key::VolumeDown,
        sdl::Scancode::POWER => Key::Power,

        sdl::Scancode::APPLICATION => Key::Compose,
        sdl::Scancode::SLEEP => Key::Sleep,

        sdl::Scancode::LSHIFT => Key::LeftShift,
        sdl::Scancode::RSHIFT => Key::RightShift,
        sdl::Scancode::LCTRL => Key::LeftControl,
        sdl::Scancode::RCTRL => Key::RightControl,
        sdl::Scancode::LALT => Key::LeftAlt,
        sdl::Scancode::RALT => Key::RightAlt,
        sdl::Scancode::LGUI => Key::LeftMeta,
        sdl::Scancode::RGUI => Key::RightMeta,

        sdl::Scancode::MENU => Key::Menu,
        sdl::Scancode::PAUSE => Key::Pause,

        sdl::Scancode::NONUSBACKSLASH => Key::NonUSBackslash,
        sdl::Scancode::SYSREQ => Key::SysReq,
        sdl::Scancode::AGAIN => Key::Again,
        sdl::Scancode::UNDO => Key::Undo,
        sdl::Scancode::COPY => Key::Copy,
        sdl::Scancode::PASTE => Key::Paste,
        sdl::Scancode::FIND => Key::Find,
        sdl::Scancode::CUT => Key::Cut,
        sdl::Scancode::HELP => Key::Help,
        sdl::Scancode::CALCULATOR => Key::Calculator,
        sdl::Scancode::ALTERASE => Key::AltErase,
        sdl::Scancode::CANCEL => Key::Cancel,

        sdl::Scancode::BRIGHTNESSUP => Key::BrightnessUp,
        sdl::Scancode::BRIGHTNESSDOWN => Key::BrightnessDown,

        sdl::Scancode::DISPLAYSWITCH => Key::SwitchVideoMode,

        sdl::Scancode::KBDILLUMTOGGLE => Key::KeyboardIlluminationToggle,
        sdl::Scancode::KBDILLUMDOWN => Key::KeyboardIlluminationDown,
        sdl::Scancode::KBDILLUMUP => Key::KeyboardIlluminationUp,

        sdl::Scancode::APP1 => Key::App1,
        sdl::Scancode::APP2 => Key::App2,
        sdl::Scancode::WWW => Key::WWW,
        sdl::Scancode::MAIL => Key::Mail,
        sdl::Scancode::COMPUTER => Key::Computer,

        sdl::Scancode::AC_BOOKMARKS => Key::ACBookmarks,
        sdl::Scancode::AC_BACK => Key::ACBack,
        sdl::Scancode::AC_FORWARD => Key::ACForward,
        sdl::Scancode::AC_HOME => Key::ACHome,
        sdl::Scancode::AC_REFRESH => Key::ACRefresh,
        sdl::Scancode::AC_SEARCH => Key::ACSearch,

        sdl::Scancode::AUDIONEXT => Key::AudioNext,
        sdl::Scancode::AUDIOPLAY => Key::AudioPlay,
        sdl::Scancode::AUDIOPREV => Key::AudioPrev,
        sdl::Scancode::AUDIOSTOP => Key::AudioStop,
        sdl::Scancode::AUDIOREWIND => Key::AudioRewind,
        sdl::Scancode::AUDIOFASTFORWARD => Key::AudioFastForward,

        sdl::Scancode::LANG1 => Key::Language1,
        sdl::Scancode::LANG2 => Key::Language2,
        sdl::Scancode::LANG3 => Key::Language3,
        sdl::Scancode::LANG4 => Key::Language4,
        sdl::Scancode::LANG5 => Key::Language5,

        sdl::Scancode::INTERNATIONAL1 => Key::International1,
        sdl::Scancode::INTERNATIONAL2 => Key::International2,
        sdl::Scancode::INTERNATIONAL3 => Key::International3,
        sdl::Scancode::INTERNATIONAL4 => Key::International4,
        sdl::Scancode::INTERNATIONAL5 => Key::International5,

        _ => Key::Unknown,
    }
}
