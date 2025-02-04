#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Unknown,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,

    Return,
    Escape,
    Backspace,
    Delete,
    Tab,
    Space,
    Minus,
    Equal,
    LeftBrace,
    RightBrace,
    Backslash,
    Semicolon,
    Apostrophe,
    Grave,
    Comma,
    Period,
    Slash,
    CapsLock,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    ScrollLock,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,

    Left,
    Right,
    Up,
    Down,

    NumLock,
    NumpadDivide,
    NumpadMultiply,
    NumpadSubtract,
    NumpadAdd,
    NumpadEnter,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    Numpad0,
    NumpadPeriod,
    NumpadEquals,
    NumpadLeftParen,
    NumpadRightParen,
    NumpadPlusMinus,
    NumpadComma,

    MediaEject,
    Stop,
    Mute,
    VolumeUp,
    VolumeDown,
    Power,

    Compose,
    Sleep,

    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    LeftAlt,
    RightAlt,
    LeftMeta,
    RightMeta,

    Menu,
    Pause,

    NonUSBackslash,
    SysReq,
    Again,
    Undo,
    Copy,
    Paste,
    Find,
    Cut,
    Help,
    AltErase,
    Cancel,

    ACBookmarks,
    ACBack,
    ACForward,
    ACHome,
    ACRefresh,
    ACSearch,

    MediaNextTrack,
    MediaPlay,
    MediaPreviousTrack,
    MediaStop,
    MediaRewind,
    MediaFastForward,

    Language1,
    Language2,
    Language3,
    Language4,
    Language5,

    International1,
    International2,
    International3,
    International4,
    International5,
}
