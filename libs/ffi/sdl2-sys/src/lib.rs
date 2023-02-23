#![allow(non_camel_case_types)]
use std::{ffi::c_void, os::raw::c_char};

pub const MAJOR_VERSION: u8 = 2;
pub const MINOR_VERSION: u8 = 24;
pub const PATCH_VERSION: u8 = 2;

#[repr(C)]
pub struct Window {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bool {
    False = 0,
    True = 1,
}

pub type JoystickID = i32;
pub type TouchID = i64;
pub type FingerID = i64;
pub type GestureID = i64;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PressedState {
    Released = 0,
    Pressed = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left = 1,
    Middle = 2,
    Right = 3,
    X1 = 4,
    X2 = 5,
}

#[repr(i32)]
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

const fn keycode_from_scancode(scancode: Scancode) -> i32 {
    scancode as i32 | 1 << 30
}

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum Keycode {
    UNKNOWN = 0,

    RETURN = '\r' as i32,
    ESCAPE = '\x1B' as i32,
    BACKSPACE = '\x08' as i32,
    TAB = '\t' as i32,
    SPACE = ' ' as i32,
    EXCLAIM = '!' as i32,
    QUOTEDBL = '"' as i32,
    HASH = '#' as i32,
    PERCENT = '%' as i32,
    DOLLAR = '$' as i32,
    AMPERSAND = '&' as i32,
    QUOTE = '\'' as i32,
    LEFTPAREN = '(' as i32,
    RIGHTPAREN = ')' as i32,
    ASTERISK = '*' as i32,
    PLUS = '+' as i32,
    COMMA = ',' as i32,
    MINUS = '-' as i32,
    PERIOD = '.' as i32,
    SLASH = '/' as i32,
    KEY_0 = '0' as i32,
    KEY_1 = '1' as i32,
    KEY_2 = '2' as i32,
    KEY_3 = '3' as i32,
    KEY_4 = '4' as i32,
    KEY_5 = '5' as i32,
    KEY_6 = '6' as i32,
    KEY_7 = '7' as i32,
    KEY_8 = '8' as i32,
    KEY_9 = '9' as i32,
    COLON = ':' as i32,
    SEMICOLON = ';' as i32,
    LESS = '<' as i32,
    EQUALS = '=' as i32,
    GREATER = '>' as i32,
    QUESTION = '?' as i32,
    AT = '@' as i32,

    /*
      Skip uppercase letters
    */
    LEFTBRACKET = '[' as i32,
    BACKSLASH = '\\' as i32,
    RIGHTBRACKET = ']' as i32,
    CARET = '^' as i32,
    UNDERSCORE = '_' as i32,
    BACKQUOTE = '`' as i32,
    a = 'a' as i32,
    b = 'b' as i32,
    c = 'c' as i32,
    d = 'd' as i32,
    e = 'e' as i32,
    f = 'f' as i32,
    g = 'g' as i32,
    h = 'h' as i32,
    i = 'i' as i32,
    j = 'j' as i32,
    k = 'k' as i32,
    l = 'l' as i32,
    m = 'm' as i32,
    n = 'n' as i32,
    o = 'o' as i32,
    p = 'p' as i32,
    q = 'q' as i32,
    r = 'r' as i32,
    s = 's' as i32,
    t = 't' as i32,
    u = 'u' as i32,
    v = 'v' as i32,
    w = 'w' as i32,
    x = 'x' as i32,
    y = 'y' as i32,
    z = 'z' as i32,

    CAPSLOCK = keycode_from_scancode(Scancode::CAPSLOCK),

    F1 = keycode_from_scancode(Scancode::F1),
    F2 = keycode_from_scancode(Scancode::F2),
    F3 = keycode_from_scancode(Scancode::F3),
    F4 = keycode_from_scancode(Scancode::F4),
    F5 = keycode_from_scancode(Scancode::F5),
    F6 = keycode_from_scancode(Scancode::F6),
    F7 = keycode_from_scancode(Scancode::F7),
    F8 = keycode_from_scancode(Scancode::F8),
    F9 = keycode_from_scancode(Scancode::F9),
    F10 = keycode_from_scancode(Scancode::F10),
    F11 = keycode_from_scancode(Scancode::F11),
    F12 = keycode_from_scancode(Scancode::F12),

    PRINTSCREEN = keycode_from_scancode(Scancode::PRINTSCREEN),
    SCROLLLOCK = keycode_from_scancode(Scancode::SCROLLLOCK),
    PAUSE = keycode_from_scancode(Scancode::PAUSE),
    INSERT = keycode_from_scancode(Scancode::INSERT),
    HOME = keycode_from_scancode(Scancode::HOME),
    PAGEUP = keycode_from_scancode(Scancode::PAGEUP),
    DELETE = '\x7F' as i32,
    END = keycode_from_scancode(Scancode::END),
    PAGEDOWN = keycode_from_scancode(Scancode::PAGEDOWN),
    RIGHT = keycode_from_scancode(Scancode::RIGHT),
    LEFT = keycode_from_scancode(Scancode::LEFT),
    DOWN = keycode_from_scancode(Scancode::DOWN),
    UP = keycode_from_scancode(Scancode::UP),

    NUMLOCKCLEAR = keycode_from_scancode(Scancode::NUMLOCKCLEAR),
    KP_DIVIDE = keycode_from_scancode(Scancode::KP_DIVIDE),
    KP_MULTIPLY = keycode_from_scancode(Scancode::KP_MULTIPLY),
    KP_MINUS = keycode_from_scancode(Scancode::KP_MINUS),
    KP_PLUS = keycode_from_scancode(Scancode::KP_PLUS),
    KP_ENTER = keycode_from_scancode(Scancode::KP_ENTER),
    KP_1 = keycode_from_scancode(Scancode::KP_1),
    KP_2 = keycode_from_scancode(Scancode::KP_2),
    KP_3 = keycode_from_scancode(Scancode::KP_3),
    KP_4 = keycode_from_scancode(Scancode::KP_4),
    KP_5 = keycode_from_scancode(Scancode::KP_5),
    KP_6 = keycode_from_scancode(Scancode::KP_6),
    KP_7 = keycode_from_scancode(Scancode::KP_7),
    KP_8 = keycode_from_scancode(Scancode::KP_8),
    KP_9 = keycode_from_scancode(Scancode::KP_9),
    KP_0 = keycode_from_scancode(Scancode::KP_0),
    KP_PERIOD = keycode_from_scancode(Scancode::KP_PERIOD),

    APPLICATION = keycode_from_scancode(Scancode::APPLICATION),
    POWER = keycode_from_scancode(Scancode::POWER),
    KP_EQUALS = keycode_from_scancode(Scancode::KP_EQUALS),
    F13 = keycode_from_scancode(Scancode::F13),
    F14 = keycode_from_scancode(Scancode::F14),
    F15 = keycode_from_scancode(Scancode::F15),
    F16 = keycode_from_scancode(Scancode::F16),
    F17 = keycode_from_scancode(Scancode::F17),
    F18 = keycode_from_scancode(Scancode::F18),
    F19 = keycode_from_scancode(Scancode::F19),
    F20 = keycode_from_scancode(Scancode::F20),
    F21 = keycode_from_scancode(Scancode::F21),
    F22 = keycode_from_scancode(Scancode::F22),
    F23 = keycode_from_scancode(Scancode::F23),
    F24 = keycode_from_scancode(Scancode::F24),
    EXECUTE = keycode_from_scancode(Scancode::EXECUTE),
    HELP = keycode_from_scancode(Scancode::HELP),
    MENU = keycode_from_scancode(Scancode::MENU),
    SELECT = keycode_from_scancode(Scancode::SELECT),
    STOP = keycode_from_scancode(Scancode::STOP),
    AGAIN = keycode_from_scancode(Scancode::AGAIN),
    UNDO = keycode_from_scancode(Scancode::UNDO),
    CUT = keycode_from_scancode(Scancode::CUT),
    COPY = keycode_from_scancode(Scancode::COPY),
    PASTE = keycode_from_scancode(Scancode::PASTE),
    FIND = keycode_from_scancode(Scancode::FIND),
    MUTE = keycode_from_scancode(Scancode::MUTE),
    VOLUMEUP = keycode_from_scancode(Scancode::VOLUMEUP),
    VOLUMEDOWN = keycode_from_scancode(Scancode::VOLUMEDOWN),
    KP_COMMA = keycode_from_scancode(Scancode::KP_COMMA),
    KP_EQUALSAS400 = keycode_from_scancode(Scancode::KP_EQUALSAS400),

    ALTERASE = keycode_from_scancode(Scancode::ALTERASE),
    SYSREQ = keycode_from_scancode(Scancode::SYSREQ),
    CANCEL = keycode_from_scancode(Scancode::CANCEL),
    CLEAR = keycode_from_scancode(Scancode::CLEAR),
    PRIOR = keycode_from_scancode(Scancode::PRIOR),
    RETURN2 = keycode_from_scancode(Scancode::RETURN2),
    SEPARATOR = keycode_from_scancode(Scancode::SEPARATOR),
    OUT = keycode_from_scancode(Scancode::OUT),
    OPER = keycode_from_scancode(Scancode::OPER),
    CLEARAGAIN = keycode_from_scancode(Scancode::CLEARAGAIN),
    CRSEL = keycode_from_scancode(Scancode::CRSEL),
    EXSEL = keycode_from_scancode(Scancode::EXSEL),

    KP_00 = keycode_from_scancode(Scancode::KP_00),
    KP_000 = keycode_from_scancode(Scancode::KP_000),
    THOUSANDSSEPARATOR = keycode_from_scancode(Scancode::THOUSANDSSEPARATOR),
    DECIMALSEPARATOR = keycode_from_scancode(Scancode::DECIMALSEPARATOR),
    CURRENCYUNIT = keycode_from_scancode(Scancode::CURRENCYUNIT),
    CURRENCYSUBUNIT = keycode_from_scancode(Scancode::CURRENCYSUBUNIT),
    KP_LEFTPAREN = keycode_from_scancode(Scancode::KP_LEFTPAREN),
    KP_RIGHTPAREN = keycode_from_scancode(Scancode::KP_RIGHTPAREN),
    KP_LEFTBRACE = keycode_from_scancode(Scancode::KP_LEFTBRACE),
    KP_RIGHTBRACE = keycode_from_scancode(Scancode::KP_RIGHTBRACE),
    KP_TAB = keycode_from_scancode(Scancode::KP_TAB),
    KP_BACKSPACE = keycode_from_scancode(Scancode::KP_BACKSPACE),
    KP_A = keycode_from_scancode(Scancode::KP_A),
    KP_B = keycode_from_scancode(Scancode::KP_B),
    KP_C = keycode_from_scancode(Scancode::KP_C),
    KP_D = keycode_from_scancode(Scancode::KP_D),
    KP_E = keycode_from_scancode(Scancode::KP_E),
    KP_F = keycode_from_scancode(Scancode::KP_F),
    KP_XOR = keycode_from_scancode(Scancode::KP_XOR),
    KP_POWER = keycode_from_scancode(Scancode::KP_POWER),
    KP_PERCENT = keycode_from_scancode(Scancode::KP_PERCENT),
    KP_LESS = keycode_from_scancode(Scancode::KP_LESS),
    KP_GREATER = keycode_from_scancode(Scancode::KP_GREATER),
    KP_AMPERSAND = keycode_from_scancode(Scancode::KP_AMPERSAND),
    KP_DBLAMPERSAND = keycode_from_scancode(Scancode::KP_DBLAMPERSAND),
    KP_VERTICALBAR = keycode_from_scancode(Scancode::KP_VERTICALBAR),
    KP_DBLVERTICALBAR = keycode_from_scancode(Scancode::KP_DBLVERTICALBAR),
    KP_COLON = keycode_from_scancode(Scancode::KP_COLON),
    KP_HASH = keycode_from_scancode(Scancode::KP_HASH),
    KP_SPACE = keycode_from_scancode(Scancode::KP_SPACE),
    KP_AT = keycode_from_scancode(Scancode::KP_AT),
    KP_EXCLAM = keycode_from_scancode(Scancode::KP_EXCLAM),
    KP_MEMSTORE = keycode_from_scancode(Scancode::KP_MEMSTORE),
    KP_MEMRECALL = keycode_from_scancode(Scancode::KP_MEMRECALL),
    KP_MEMCLEAR = keycode_from_scancode(Scancode::KP_MEMCLEAR),
    KP_MEMADD = keycode_from_scancode(Scancode::KP_MEMADD),
    KP_MEMSUBTRACT = keycode_from_scancode(Scancode::KP_MEMSUBTRACT),
    KP_MEMMULTIPLY = keycode_from_scancode(Scancode::KP_MEMMULTIPLY),
    KP_MEMDIVIDE = keycode_from_scancode(Scancode::KP_MEMDIVIDE),
    KP_PLUSMINUS = keycode_from_scancode(Scancode::KP_PLUSMINUS),
    KP_CLEAR = keycode_from_scancode(Scancode::KP_CLEAR),
    KP_CLEARENTRY = keycode_from_scancode(Scancode::KP_CLEARENTRY),
    KP_BINARY = keycode_from_scancode(Scancode::KP_BINARY),
    KP_OCTAL = keycode_from_scancode(Scancode::KP_OCTAL),
    KP_DECIMAL = keycode_from_scancode(Scancode::KP_DECIMAL),
    KP_HEXADECIMAL = keycode_from_scancode(Scancode::KP_HEXADECIMAL),

    LCTRL = keycode_from_scancode(Scancode::LCTRL),
    LSHIFT = keycode_from_scancode(Scancode::LSHIFT),
    LALT = keycode_from_scancode(Scancode::LALT),
    LGUI = keycode_from_scancode(Scancode::LGUI),
    RCTRL = keycode_from_scancode(Scancode::RCTRL),
    RSHIFT = keycode_from_scancode(Scancode::RSHIFT),
    RALT = keycode_from_scancode(Scancode::RALT),
    RGUI = keycode_from_scancode(Scancode::RGUI),

    MODE = keycode_from_scancode(Scancode::MODE),

    AUDIONEXT = keycode_from_scancode(Scancode::AUDIONEXT),
    AUDIOPREV = keycode_from_scancode(Scancode::AUDIOPREV),
    AUDIOSTOP = keycode_from_scancode(Scancode::AUDIOSTOP),
    AUDIOPLAY = keycode_from_scancode(Scancode::AUDIOPLAY),
    AUDIOMUTE = keycode_from_scancode(Scancode::AUDIOMUTE),
    MEDIASELECT = keycode_from_scancode(Scancode::MEDIASELECT),
    WWW = keycode_from_scancode(Scancode::WWW),
    MAIL = keycode_from_scancode(Scancode::MAIL),
    CALCULATOR = keycode_from_scancode(Scancode::CALCULATOR),
    COMPUTER = keycode_from_scancode(Scancode::COMPUTER),
    AC_SEARCH = keycode_from_scancode(Scancode::AC_SEARCH),
    AC_HOME = keycode_from_scancode(Scancode::AC_HOME),
    AC_BACK = keycode_from_scancode(Scancode::AC_BACK),
    AC_FORWARD = keycode_from_scancode(Scancode::AC_FORWARD),
    AC_STOP = keycode_from_scancode(Scancode::AC_STOP),
    AC_REFRESH = keycode_from_scancode(Scancode::AC_REFRESH),
    AC_BOOKMARKS = keycode_from_scancode(Scancode::AC_BOOKMARKS),

    BRIGHTNESSDOWN = keycode_from_scancode(Scancode::BRIGHTNESSDOWN),
    BRIGHTNESSUP = keycode_from_scancode(Scancode::BRIGHTNESSUP),
    DISPLAYSWITCH = keycode_from_scancode(Scancode::DISPLAYSWITCH),
    KBDILLUMTOGGLE = keycode_from_scancode(Scancode::KBDILLUMTOGGLE),
    KBDILLUMDOWN = keycode_from_scancode(Scancode::KBDILLUMDOWN),
    KBDILLUMUP = keycode_from_scancode(Scancode::KBDILLUMUP),
    EJECT = keycode_from_scancode(Scancode::EJECT),
    SLEEP = keycode_from_scancode(Scancode::SLEEP),
    APP1 = keycode_from_scancode(Scancode::APP1),
    APP2 = keycode_from_scancode(Scancode::APP2),

    AUDIOREWIND = keycode_from_scancode(Scancode::AUDIOREWIND),
    AUDIOFASTFORWARD = keycode_from_scancode(Scancode::AUDIOFASTFORWARD),
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Keymod(pub u16);

impl Keymod {
    pub const NONE: Self = Self(0x0000);
    pub const LSHIFT: Self = Self(0x0001);
    pub const RSHIFT: Self = Self(0x0002);
    pub const LCTRL: Self = Self(0x0040);
    pub const RCTRL: Self = Self(0x0080);
    pub const LALT: Self = Self(0x0100);
    pub const RALT: Self = Self(0x0200);
    pub const LGUI: Self = Self(0x0400);
    pub const RGUI: Self = Self(0x0800);
    pub const NUM: Self = Self(0x1000);
    pub const CAPS: Self = Self(0x2000);
    pub const MODE: Self = Self(0x4000);
    pub const SCROLL: Self = Self(0x8000);

    pub const CTRL: Self = Self(Self::LCTRL.0 | Self::RCTRL.0);
    pub const SHIFT: Self = Self(Self::LSHIFT.0 | Self::RSHIFT.0);
    pub const ALT: Self = Self(Self::LALT.0 | Self::RALT.0);
    pub const GUI: Self = Self(Self::LGUI.0 | Self::RGUI.0);
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Keysym {
    pub scancode: Scancode,
    pub sym: Keycode,
    pub modifiers: Keymod,
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
    pub state: PressedState,
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
    pub button: MouseButton,
    pub state: PressedState,
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

#[repr(C)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub const fn current() -> Self {
        Self {
            major: MAJOR_VERSION,
            minor: MINOR_VERSION,
            patch: PATCH_VERSION,
        }
    }
}

#[repr(C)]
pub enum SysWMType {
    UNKNOWN,
    WINDOWS,
    X11,
    DIRECTFB,
    COCOA,
    UIKIT,
    WAYLAND,
    MIR, /* no longer available, left for API/ABI compatibility. Remove in 2.1! */
    WINRT,
    ANDROID,
    VIVANTE,
    OS2,
    HAIKU,
    KMSDRM,
    RISCOS,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SysWMTypeX11 {
    pub display: *mut c_void,
    pub window: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SysWMTypeWayland {
    pub display: *mut c_void,
    pub surface: *mut c_void,
    pub shell_surface: *mut c_void,
    pub egl_window: *mut c_void,
    pub xdg_surface: *mut c_void,
    pub xdg_toplevel: *mut c_void,
    pub xdg_popup: *mut c_void,
    pub xdg_positioner: *mut c_void,
}

#[repr(C)]
pub union SysWMTypeUnion {
    pub x11: SysWMTypeX11,
    pub wayland: SysWMTypeWayland,
    dummy: [u8; 64],
}

#[repr(C)]
pub struct SysWMinfo {
    pub version: Version,
    pub subsystem: SysWMType,
    pub info: SysWMTypeUnion,
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

    pub fn SDL_GetKeyFromScancode(scancode: Scancode) -> Keycode;

    pub fn SDL_PollEvent(event: *mut Event) -> i32;

    pub fn SDL_GetWindowWMInfo(window: *mut Window, info: *mut SysWMinfo) -> Bool;

    pub fn SDL_Vulkan_LoadLibrary(path: *const c_char) -> i32;
    pub fn SDL_Vulkan_GetInstanceExtensions(
        window: *mut Window,
        count: &mut u32,
        names: *mut *const c_char,
    ) -> i32;
    pub fn SDL_Vulkan_GetVkGetInstanceProcAddr() -> *mut c_void;
    pub fn SDL_Vulkan_CreateSurface(window: *mut Window, instance: u64, surface: *mut u64) -> Bool;
    pub fn SDL_Vulkan_GetDrawableSize(window: *mut Window, w: *mut i32, h: *mut i32);
}

pub const INIT_VIDEO: u32 = 0x0000_0020;
pub const WINDOW_SHOWN: u32 = 0x0000_0004;
pub const WINDOW_RESIZABLE: u32 = 0x0000_0020;
pub const WINDOW_VULKAN: u32 = 0x1000_0000;
