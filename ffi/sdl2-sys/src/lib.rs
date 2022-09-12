#![allow(non_camel_case_types)]
use std::{ffi::c_void, os::raw::c_char};

#[repr(C)]
pub struct Window {
    _unused: [u8; 0],
}

pub type JoystickID = i32;
pub type TouchID = i64;
pub type FingerID = i64;
pub type GestureID = i64;
pub type Keycode = i32;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Scancode {
    UNKNOWN = 0,

    /**
     *  \name Usage page 0x07
     *
     *  These values are from usage page 0x07 (USB keyboard page).
     */
    /* @{ */
    A = 4,
    B = 5,
    C = 6,
    D = 7,
    E = 8,
    F = 9,
    G = 10,
    H = 11,
    I = 12,
    J = 13,
    K = 14,
    L = 15,
    M = 16,
    N = 17,
    O = 18,
    P = 19,
    Q = 20,
    R = 21,
    S = 22,
    T = 23,
    U = 24,
    V = 25,
    W = 26,
    X = 27,
    Y = 28,
    Z = 29,

    SCANCODE_1 = 30,
    SCANCODE_2 = 31,
    SCANCODE_3 = 32,
    SCANCODE_4 = 33,
    SCANCODE_5 = 34,
    SCANCODE_6 = 35,
    SCANCODE_7 = 36,
    SCANCODE_8 = 37,
    SCANCODE_9 = 38,
    SCANCODE_0 = 39,

    RETURN = 40,
    ESCAPE = 41,
    BACKSPACE = 42,
    TAB = 43,
    SPACE = 44,

    MINUS = 45,
    EQUALS = 46,
    LEFTBRACKET = 47,
    RIGHTBRACKET = 48,
    BACKSLASH = 49,
    /**< Located at the lower left of the return
     *   key on ISO keyboards and at the right end
     *   of the QWERTY row on ANSI keyboards.
     *   Produces REVERSE SOLIDUS (backslash) and
     *   VERTICAL LINE in a US layout, REVERSE
     *   SOLIDUS and VERTICAL LINE in a UK Mac
     *   layout, NUMBER SIGN and TILDE in a UK
     *   Windows layout, DOLLAR SIGN and POUND SIGN
     *   in a Swiss German layout, NUMBER SIGN and
     *   APOSTROPHE in a German layout, GRAVE
     *   ACCENT and POUND SIGN in a French Mac
     *   layout, and ASTERISK and MICRO SIGN in a
     *   French Windows layout.
     */
    NONUSHASH = 50,
    /**< ISO USB keyboards actually use this code
     *   instead of 49 for the same key, but all
     *   OSes I've seen treat the two codes
     *   identically. So, as an implementor, unless
     *   your keyboard generates both of those
     *   codes and your OS treats them differently,
     *   you should generate BACKSLASH
     *   instead of this code. As a user, you
     *   should not rely on this code because SDL
     *   will never generate it with most (all?)
     *   keyboards.
     */
    SEMICOLON = 51,
    APOSTROPHE = 52,
    GRAVE = 53,
    /**< Located in the top left corner (on both ANSI
     *   and ISO keyboards). Produces GRAVE ACCENT and
     *   TILDE in a US Windows layout and in US and UK
     *   Mac layouts on ANSI keyboards, GRAVE ACCENT
     *   and NOT SIGN in a UK Windows layout, SECTION
     *   SIGN and PLUS-MINUS SIGN in US and UK Mac
     *   layouts on ISO keyboards, SECTION SIGN and
     *   DEGREE SIGN in a Swiss German layout (Mac:
     *   only on ISO keyboards), CIRCUMFLEX ACCENT and
     *   DEGREE SIGN in a German layout (Mac: only on
     *   ISO keyboards), SUPERSCRIPT TWO and TILDE in a
     *   French Windows layout, COMMERCIAL AT and
     *   NUMBER SIGN in a French Mac layout on ISO
     *   keyboards, and LESS-THAN SIGN and GREATER-THAN
     *   SIGN in a Swiss German, German, or French Mac
     *   layout on ANSI keyboards.
     */
    COMMA = 54,
    PERIOD = 55,
    SLASH = 56,

    CAPSLOCK = 57,

    F1 = 58,
    F2 = 59,
    F3 = 60,
    F4 = 61,
    F5 = 62,
    F6 = 63,
    F7 = 64,
    F8 = 65,
    F9 = 66,
    F10 = 67,
    F11 = 68,
    F12 = 69,

    PRINTSCREEN = 70,
    SCROLLLOCK = 71,
    PAUSE = 72,
    INSERT = 73,
    /**< insert on PC, help on some Mac keyboards (but
    does send code 73, not 117) */
    HOME = 74,
    PAGEUP = 75,
    DELETE = 76,
    END = 77,
    PAGEDOWN = 78,
    RIGHT = 79,
    LEFT = 80,
    DOWN = 81,
    UP = 82,

    NUMLOCKCLEAR = 83,
    /**< num lock on PC, clear on Mac keyboards
     */
    KP_DIVIDE = 84,
    KP_MULTIPLY = 85,
    KP_MINUS = 86,
    KP_PLUS = 87,
    KP_ENTER = 88,
    KP_1 = 89,
    KP_2 = 90,
    KP_3 = 91,
    KP_4 = 92,
    KP_5 = 93,
    KP_6 = 94,
    KP_7 = 95,
    KP_8 = 96,
    KP_9 = 97,
    KP_0 = 98,
    KP_PERIOD = 99,

    NONUSBACKSLASH = 100,
    /**< This is the additional key that ISO
     *   keyboards have over ANSI ones,
     *   located between left shift and Y.
     *   Produces GRAVE ACCENT and TILDE in a
     *   US or UK Mac layout, REVERSE SOLIDUS
     *   (backslash) and VERTICAL LINE in a
     *   US or UK Windows layout, and
     *   LESS-THAN SIGN and GREATER-THAN SIGN
     *   in a Swiss German, German, or French
     *   layout. */
    APPLICATION = 101,
    /**< windows contextual menu, compose */
    POWER = 102,
    /**< The USB document says this is a status flag,
     *   not a physical key - but some Mac keyboards
     *   do have a power key. */
    KP_EQUALS = 103,
    F13 = 104,
    F14 = 105,
    F15 = 106,
    F16 = 107,
    F17 = 108,
    F18 = 109,
    F19 = 110,
    F20 = 111,
    F21 = 112,
    F22 = 113,
    F23 = 114,
    F24 = 115,
    EXECUTE = 116,
    HELP = 117,
    MENU = 118,
    SELECT = 119,
    STOP = 120,
    AGAIN = 121,
    /**< redo */
    UNDO = 122,
    CUT = 123,
    COPY = 124,
    PASTE = 125,
    FIND = 126,
    MUTE = 127,
    VOLUMEUP = 128,
    VOLUMEDOWN = 129,
    /* not sure whether there's a reason to enable these */
    /*     LOCKINGCAPSLOCK = 130,  */
    /*     LOCKINGNUMLOCK = 131, */
    /*     LOCKINGSCROLLLOCK = 132, */
    KP_COMMA = 133,
    KP_EQUALSAS400 = 134,

    INTERNATIONAL1 = 135,
    /**< used on Asian keyboards, see
    footnotes in USB doc */
    INTERNATIONAL2 = 136,
    INTERNATIONAL3 = 137,
    /**< Yen */
    INTERNATIONAL4 = 138,
    INTERNATIONAL5 = 139,
    INTERNATIONAL6 = 140,
    INTERNATIONAL7 = 141,
    INTERNATIONAL8 = 142,
    INTERNATIONAL9 = 143,
    LANG1 = 144,
    /**< Hangul/English toggle */
    LANG2 = 145,
    /**< Hanja conversion */
    LANG3 = 146,
    /**< Katakana */
    LANG4 = 147,
    /**< Hiragana */
    LANG5 = 148,
    /**< Zenkaku/Hankaku */
    LANG6 = 149,
    /**< reserved */
    LANG7 = 150,
    /**< reserved */
    LANG8 = 151,
    /**< reserved */
    LANG9 = 152,
    /**< reserved */
    ALTERASE = 153,
    /**< Erase-Eaze */
    SYSREQ = 154,
    CANCEL = 155,
    CLEAR = 156,
    PRIOR = 157,
    RETURN2 = 158,
    SEPARATOR = 159,
    OUT = 160,
    OPER = 161,
    CLEARAGAIN = 162,
    CRSEL = 163,
    EXSEL = 164,

    KP_00 = 176,
    KP_000 = 177,
    THOUSANDSSEPARATOR = 178,
    DECIMALSEPARATOR = 179,
    CURRENCYUNIT = 180,
    CURRENCYSUBUNIT = 181,
    KP_LEFTPAREN = 182,
    KP_RIGHTPAREN = 183,
    KP_LEFTBRACE = 184,
    KP_RIGHTBRACE = 185,
    KP_TAB = 186,
    KP_BACKSPACE = 187,
    KP_A = 188,
    KP_B = 189,
    KP_C = 190,
    KP_D = 191,
    KP_E = 192,
    KP_F = 193,
    KP_XOR = 194,
    KP_POWER = 195,
    KP_PERCENT = 196,
    KP_LESS = 197,
    KP_GREATER = 198,
    KP_AMPERSAND = 199,
    KP_DBLAMPERSAND = 200,
    KP_VERTICALBAR = 201,
    KP_DBLVERTICALBAR = 202,
    KP_COLON = 203,
    KP_HASH = 204,
    KP_SPACE = 205,
    KP_AT = 206,
    KP_EXCLAM = 207,
    KP_MEMSTORE = 208,
    KP_MEMRECALL = 209,
    KP_MEMCLEAR = 210,
    KP_MEMADD = 211,
    KP_MEMSUBTRACT = 212,
    KP_MEMMULTIPLY = 213,
    KP_MEMDIVIDE = 214,
    KP_PLUSMINUS = 215,
    KP_CLEAR = 216,
    KP_CLEARENTRY = 217,
    KP_BINARY = 218,
    KP_OCTAL = 219,
    KP_DECIMAL = 220,
    KP_HEXADECIMAL = 221,

    LCTRL = 224,
    LSHIFT = 225,
    LALT = 226,
    /**< alt, option */
    LGUI = 227,
    /**< windows, command (apple), meta */
    RCTRL = 228,
    RSHIFT = 229,
    RALT = 230,
    /**< alt gr, option */
    RGUI = 231,
    /**< windows, command (apple), meta */
    MODE = 257,
    /**< I'm not sure if this is really not covered
     *   by any of the above, but since there's a
     *   special KMOD_MODE for it I'm adding it here
     */

    /* @} *//* Usage page 0x07 */

    /**
     *  \name Usage page 0x0C
     *
     *  These values are mapped from usage page 0x0C (USB consumer page).
     */
    /* @{ */
    AUDIONEXT = 258,
    AUDIOPREV = 259,
    AUDIOSTOP = 260,
    AUDIOPLAY = 261,
    AUDIOMUTE = 262,
    MEDIASELECT = 263,
    WWW = 264,
    MAIL = 265,
    CALCULATOR = 266,
    COMPUTER = 267,
    AC_SEARCH = 268,
    AC_HOME = 269,
    AC_BACK = 270,
    AC_FORWARD = 271,
    AC_STOP = 272,
    AC_REFRESH = 273,
    AC_BOOKMARKS = 274,

    /* @} *//* Usage page 0x0C */
    /**
     *  \name Walther keys
     *
     *  These are values that Christian Walther added (for mac keyboard?).
     */
    /* @{ */
    BRIGHTNESSDOWN = 275,
    BRIGHTNESSUP = 276,
    DISPLAYSWITCH = 277,
    /**< display mirroring/dual display
    switch, video mode switch */
    KBDILLUMTOGGLE = 278,
    KBDILLUMDOWN = 279,
    KBDILLUMUP = 280,
    EJECT = 281,
    SLEEP = 282,

    APP1 = 283,
    APP2 = 284,

    /* @} *//* Walther keys */
    /**
     *  \name Usage page 0x0C (additional media keys)
     *
     *  These values are mapped from usage page 0x0C (USB consumer page).
     */
    /* @{ */
    AUDIOREWIND = 285,
    AUDIOFASTFORWARD = 286,

    /* @} *//* Usage page 0x0C (additional media keys) */

    /* Add any other keys here. */
    NUM_SCANCODES = 512,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Keysym {
    pub scancode: Scancode,
    pub sym: Keycode,
    pub modifiers: u16,
    pub _unused: u32,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    FIRSTEVENT = 0,
    QUIT = 0x100,

    APP_TERMINATING,
    APP_LOWMEMORY,
    APP_WILLENTERBACKGROUND,
    APP_DIDENTERBACKGROUND,
    APP_WILLENTERFOREGROUND,
    APP_DIDENTERFOREGROUND,

    LOCALECHANGED,

    DISPLAYEVENT = 0x150,

    WINDOWEVENT = 0x200,
    SYSWMEVENT,

    KEYDOWN = 0x300,
    KEYUP,
    TEXTEDITING,
    TEXTINPUT,
    KEYMAPCHANGED,

    MOUSEMOTION = 0x400,
    MOUSEBUTTONDOWN,
    MOUSEBUTTONUP,
    MOUSEWHEEL,

    JOYAXISMOTION = 0x600,
    JOYBALLMOTION,
    JOYHATMOTION,
    JOYBUTTONDOWN,
    JOYBUTTONUP,
    JOYDEVICEADDED,
    JOYDEVICEREMOVED,

    CONTROLLERAXISMOTION = 0x650,
    CONTROLLERBUTTONDOWN,
    CONTROLLERBUTTONUP,
    CONTROLLERDEVICEADDED,
    CONTROLLERDEVICEREMOVED,
    CONTROLLERDEVICEREMAPPED,
    CONTROLLERTOUCHPADDOWN,
    CONTROLLERTOUCHPADMOTION,
    CONTROLLERTOUCHPADUP,
    CONTROLLERSENSORUPDATE,

    FINGERDOWN = 0x700,
    FINGERUP,
    FINGERMOTION,

    DOLLARGESTURE = 0x800,
    DOLLARRECORD,
    MULTIGESTURE,

    CLIPBOARDUPDATE = 0x900,

    DROPFILE = 0x1000,
    DROPTEXT,
    DROPBEGIN,
    DROPCOMPLETE,

    AUDIODEVICEADDED = 0x1100,
    AUDIODEVICEREMOVED,

    SENSORUPDATE = 0x1200,
    RENDER_TARGETS_RESET = 0x2000,
    RENDER_DEVICE_RESET,

    POLLSENTINEL = 0x7F00,

    USEREVENT = 0x8000,
    LASTEVENT = 0xFFFF,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CommonEvent {
    pub r#type: EventType,
    pub timestamp: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DisplayEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub display: u32,
    pub event: u8,
    pub _padding1: u8,
    pub _padding2: u8,
    pub _padding3: u8,
    pub data1: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WindowEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub window_id: u32,
    pub event: WindowEventId,
    pub padding1: u8,
    pub padding2: u8,
    pub padding3: u8,
    pub data1: i32,
    pub data2: i32,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WindowEventId {
    /// Never used *
    None,
    /// Window has been shown *
    Shown,
    /// Window has been hidden *
    Hidden,
    /// Window has been exposed and should be redrawn *
    Exposed,
    /// Window has been moved to data1, data2 *
    Moved,
    /// Window has been resized to data1xdata2 *
    Resized,
    /// The window size has changed, either as a result of an API call or through the system or user changing the window size. *
    SizeChanged,
    /// Window has been minimized *
    Minimized,
    /// Window has been maximized *
    Maximized,
    /// Window has been restored to normal size and position *
    Restored,
    /// Window has gained mouse focus *
    Enter,
    /// Window has lost mouse focus *
    Leave,
    /// Window has gained keyboard focus *
    FocusGained,
    /// Window has lost keyboard focus *
    FocusLost,
    /// The window manager requests that the window be closed *
    Close,
    /// Window is being offered a focus (should SetWindowInputFocus() on itself or a subwindow, or ignore) *
    TakeFocus,
    /// Window had a hit test that wasn't SDL_HITTEST_NORMAL. *
    HitTest,
    /// The ICC profile of the window's display has changed. *
    IccprofChanged,
    /// Window has been moved to display data1. *
    DisplayChanged,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyboardEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub window_id: u32,
    pub state: u8,
    pub repeat: u8,
    pub _padding2: u8,
    pub _padding3: u8,
    pub keysym: Keysym,
}

const TEXTEDITINGEVENT_TEXT_SIZE: usize = 32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TextEditingEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub window_id: u32,
    pub text: [u8; TEXTEDITINGEVENT_TEXT_SIZE],
    pub start: i32,
    pub length: i32,
}

const TEXTINPUTEVENT_TEXT_SIZE: usize = 32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TextInputEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub window_id: u32,
    pub text: [u8; TEXTINPUTEVENT_TEXT_SIZE],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MouseMotionEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub state: u32,
    pub x: i32,
    pub y: i32,
    pub xrel: i32,
    pub yrel: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MouseButtonEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub button: u8,
    pub state: u8,
    pub clicks: u8,
    pub padding1: u8,
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MouseWheelEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub x: i32,
    pub y: i32,
    pub direction: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JoyAxisEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: JoystickID,
    pub axis: u8,
    pub padding1: u8,
    pub padding2: u8,
    pub padding3: u8,
    pub value: i16,
    pub padding4: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JoyBallEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: JoystickID,
    pub ball: u8,
    pub padding1: u8,
    pub padding2: u8,
    pub padding3: u8,
    pub xrel: i16,
    pub yrel: i16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JoyHatEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: JoystickID,
    pub hat: u8,
    pub value: u8,
    pub padding1: u8,
    pub padding2: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JoyButtonEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: JoystickID,
    pub button: u8,
    pub state: u8,
    pub padding1: u8,
    pub padding2: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JoyDeviceEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ControllerAxisEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: JoystickID,
    pub axis: u8,
    pub padding1: u8,
    pub padding2: u8,
    pub padding3: u8,
    pub value: i16,
    pub padding4: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ControllerButtonEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: JoystickID,
    pub button: u8,
    pub state: u8,
    pub padding1: u8,
    pub padding2: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ControllerDeviceEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ControllerTouchpadEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: JoystickID,
    pub touchpad: i32,
    pub finger: i32,
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ControllerSensorEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: JoystickID,
    pub sensor: i32,
    pub data: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AudioDeviceEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: u32,
    pub iscapture: u8,
    pub padding1: u8,
    pub padding2: u8,
    pub padding3: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TouchFingerEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub touch_id: TouchID,
    pub finger_id: FingerID,
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub pressure: f32,
    pub window_id: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MultiGestureEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub touch_id: TouchID,
    pub d_theta: f32,
    pub d_dist: f32,
    pub x: f32,
    pub y: f32,
    pub num_fingers: u16,
    pub padding: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DollarGestureEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub touch_id: TouchID,
    pub gesture_id: GestureID,
    pub num_fingers: u32,
    pub error: f32,
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DropEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub file: *const c_char,
    pub window_id: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SensorEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub which: i32,
    pub data: [f32; 6],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QuitEvent {
    pub r#type: EventType,
    pub timestamp: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OSEvent {
    pub r#type: EventType,
    pub timestamp: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UserEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub window_id: u32,
    pub code: i32,
    pub data1: *mut c_void,
    pub data2: *mut c_void,
}

#[repr(C)]
pub struct SysWMmsg {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SysWMEvent {
    pub r#type: EventType,
    pub timestamp: u32,
    pub msg: *mut SysWMmsg,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union Event {
    pub r#type: EventType,
    pub common: CommonEvent,
    pub display: DisplayEvent,
    pub window: WindowEvent,
    pub key: KeyboardEvent,
    pub edit: TextEditingEvent,
    pub text: TextInputEvent,
    pub motion: MouseMotionEvent,
    pub button: MouseButtonEvent,
    pub wheel: MouseWheelEvent,
    pub jaxis: JoyAxisEvent,
    pub jball: JoyBallEvent,
    pub jhat: JoyHatEvent,
    pub jbutton: JoyButtonEvent,
    pub jdevice: JoyDeviceEvent,
    pub caxis: ControllerAxisEvent,
    pub cbutton: ControllerButtonEvent,
    pub cdevice: ControllerDeviceEvent,
    pub ctouchpad: ControllerTouchpadEvent,
    pub csensor: ControllerSensorEvent,
    pub adevice: AudioDeviceEvent,
    pub sensor: SensorEvent,
    pub quit: QuitEvent,
    pub user: UserEvent,
    pub syswm: SysWMEvent,
    pub tfinger: TouchFingerEvent,
    pub mgesture: MultiGestureEvent,
    pub dgesture: DollarGestureEvent,
    pub r#drop: DropEvent,
}

extern "C" {
    pub fn SDL_Init(flags: u32) -> i32;
    pub fn SDL_Quit();

    pub fn SDL_CreateWindow(
        title: *const c_char,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        flags: u32,
    ) -> *mut Window;
    pub fn SDL_DestroyWindow(window: *mut Window);

    pub fn SDL_GetWindowID(window: *mut Window) -> u32;
    pub fn SDL_GetWindowFromID(id: u32) -> *mut Window;

    pub fn SDL_PollEvent(event: *mut Event) -> i32;

    pub fn SDL_Vulkan_LoadLibrary(path: *const c_char) -> i32;
    pub fn SDL_Vulkan_GetInstanceExtensions(
        window: *mut Window,
        count: &mut u32,
        names: *mut *const c_char,
    ) -> i32;
    pub fn SDL_Vulkan_GetVkGetInstanceProcAddr() -> *mut c_void;
    pub fn SDL_Vulkan_CreateSurface(window: *mut Window, instance: u64, surface: *mut u64) -> i32;
    pub fn SDL_Vulkan_GetDrawableSize(window: *mut Window, w: *mut i32, h: *mut i32);
}

pub const INIT_VIDEO: u32 = 0x0000_0020;
pub const WINDOW_SHOWN: u32 = 0x0000_0004;
pub const WINDOW_RESIZABLE: u32 = 0x0000_0020;
pub const WINDOW_VULKAN: u32 = 0x1000_0000;
