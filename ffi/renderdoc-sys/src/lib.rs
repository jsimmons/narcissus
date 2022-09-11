use std::{
    ffi::{c_void, CStr, CString},
    mem::MaybeUninit,
    os::raw::c_char,
};

mod helpers;

#[repr(C)]
pub enum Version {
    Version1_0_0 = 10000,
    Version1_0_1 = 10001,
    Version1_0_2 = 10002,
    Version1_1_0 = 10100,
    Version1_1_1 = 10101,
    Version1_1_2 = 10102,
    Version1_2_0 = 10200,
    Version1_3_0 = 10300,
    Version1_4_0 = 10400,
    Version1_4_1 = 10401,
    Version1_4_2 = 10402,
    Version1_5_0 = 10500,
}

#[repr(C)]
pub enum CaptureOption {
    // Allow the application to enable vsync
    //
    // Default - enabled
    //
    // 1 - The application can enable or disable vsync at will
    // 0 - vsync is force disabled
    AllowVSync = 0,

    // Allow the application to enable fullscreen
    //
    // Default - enabled
    //
    // 1 - The application can enable or disable fullscreen at will
    // 0 - fullscreen is force disabled
    AllowFullscreen = 1,

    // Record API debugging events and messages
    //
    // Default - disabled
    //
    // 1 - Enable built-in API debugging features and records the results into
    //     the capture, which is matched up with events on replay
    // 0 - no API debugging is forcibly enabled
    APIValidation = 2,

    // Capture CPU callstacks for API events
    //
    // Default - disabled
    //
    // 1 - Enables capturing of callstacks
    // 0 - no callstacks are captured
    CaptureCallstacks = 3,

    // When capturing CPU callstacks, only capture them from actions.
    // This option does nothing without the above option being enabled
    //
    // Default - disabled
    //
    // 1 - Only captures callstacks for actions.
    //     Ignored if CaptureCallstacks is disabled
    // 0 - Callstacks, if enabled, are captured for every event.
    CaptureCallstacksOnlyDraws = 4,

    // Specify a delay in seconds to wait for a debugger to attach, after
    // creating or injecting into a process, before continuing to allow it to run.
    //
    // 0 indicates no delay, and the process will run immediately after injection
    //
    // Default - 0 seconds
    //
    DelayForDebugger = 5,

    // Verify buffer access. This includes checking the memory returned by a Map() call to
    // detect any out-of-bounds modification, as well as initialising buffers with undefined contents
    // to a marker value to catch use of uninitialised memory.
    //
    // NOTE: This option is only valid for OpenGL and D3D11. Explicit APIs such as D3D12 and Vulkan do
    // not do the same kind of interception & checking and undefined contents are really undefined.
    //
    // Default - disabled
    //
    // 1 - Verify buffer access
    // 0 - No verification is performed, and overwriting bounds may cause crashes or corruption in
    //     RenderDoc.
    VerifyBufferAccess = 6,

    // Hooks any system API calls that create child processes, and injects
    // RenderDoc into them recursively with the same options.
    //
    // Default - disabled
    //
    // 1 - Hooks into spawned child processes
    // 0 - Child processes are not hooked by RenderDoc
    HookIntoChildren = 7,

    // By default RenderDoc only includes resources in the final capture necessary
    // for that frame, this allows you to override that behaviour.
    //
    // Default - disabled
    //
    // 1 - all live resources at the time of capture are included in the capture
    //     and available for inspection
    // 0 - only the resources referenced by the captured frame are included
    RefAllResources = 8,

    // **NOTE**: As of RenderDoc v1.1 this option has been deprecated. Setting or
    // getting it will be ignored, to allow compatibility with older versions.
    // In v1.1 the option acts as if it's always enabled.
    //
    // By default RenderDoc skips saving initial states for resources where the
    // previous contents don't appear to be used, assuming that writes before
    // reads indicate previous contents aren't used.
    //
    // Default - disabled
    //
    // 1 - initial contents at the start of each captured frame are saved, even if
    //     they are later overwritten or cleared before being used.
    // 0 - unless a read is detected, initial contents will not be saved and will
    //     appear as black or empty data.
    SaveAllInitials = 9,

    // In APIs that allow for the recording of command lists to be replayed later,
    // RenderDoc may choose to not capture command lists before a frame capture is
    // triggered, to reduce overheads. This means any command lists recorded once
    // and replayed many times will not be available and may cause a failure to
    // capture.
    //
    // NOTE: This is only true for APIs where multithreading is difficult or
    // discouraged. Newer APIs like Vulkan and D3D12 will ignore this option
    // and always capture all command lists since the API is heavily oriented
    // around it and the overheads have been reduced by API design.
    //
    // 1 - All command lists are captured from the start of the application
    // 0 - Command lists are only captured if their recording begins during
    //     the period when a frame capture is in progress.
    CaptureAllCmdLists = 10,

    // Mute API debugging output when the API validation mode option is enabled
    //
    // Default - enabled
    //
    // 1 - Mute any API debug messages from being displayed or passed through
    // 0 - API debugging is displayed as normal
    DebugOutputMute = 11,

    // Option to allow vendor extensions to be used even when they may be
    // incompatible with RenderDoc and cause corrupted replays or crashes.
    //
    // Default - inactive
    //
    // No values are documented, this option should only be used when absolutely
    // necessary as directed by a RenderDoc developer.
    AllowUnsupportedVendorExtensions = 12,
}

#[repr(C)]
pub enum InputButton {
    // '0' - '9' matches ASCII values
    Key0 = 0x30,
    Key1 = 0x31,
    Key2 = 0x32,
    Key3 = 0x33,
    Key4 = 0x34,
    Key5 = 0x35,
    Key6 = 0x36,
    Key7 = 0x37,
    Key8 = 0x38,
    Key9 = 0x39,

    // 'A' - 'Z' matches ASCII values
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,

    // leave the rest of the ASCII range free
    // in case we want to use it later
    NonPrintable = 0x100,

    Divide,
    Multiply,
    Subtract,
    Plus,

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

    Home,
    End,
    Insert,
    Delete,
    PageUp,
    PageDn,

    Backspace,
    Tab,
    PrtScrn,
    Pause,

    Max,
}

#[repr(C)]
pub struct OverlayBits(u32);

impl OverlayBits {
    // This single bit controls whether the overlay is enabled or disabled globally
    pub const ENABLED: Self = Self(0x1);

    // Show the average framerate over several seconds as well as min/max
    pub const FRAME_RATE: Self = Self(0x2);

    // Show the current frame number
    pub const FRAME_NUMBER: Self = Self(0x4);

    // Show a list of recent captures, and how many captures have been made
    pub const CAPTURE_LIST: Self = Self(0x8);

    // Default values for the overlay mask
    pub const DEFAULT: Self =
        Self(Self::ENABLED.0 | Self::FRAME_RATE.0 | Self::FRAME_NUMBER.0 | Self::CAPTURE_LIST.0);

    // Enable all bits
    pub const ALL: Self = Self(!0);

    // Disable all bits
    pub const NONE: Self = Self(0);
}

impl std::ops::BitOr for OverlayBits {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

pub type DevicePointer = *mut c_void;
pub type WindowHandle = *mut c_void;

pub type FnGetApi = extern "system" fn(version: Version, out_pointers: *mut *mut c_void) -> i32;
pub type FnGetApiVersion = extern "system" fn(major: &mut i32, minor: &mut i32, patch: &mut i32);
pub type FnSetCaptureOptionU32 =
    extern "system" fn(capture_option: CaptureOption, value: u32) -> i32;
pub type FnSetCaptureOptionF32 =
    extern "system" fn(capture_option: CaptureOption, value: f32) -> i32;
pub type FnGetCaptureOptionU32 = extern "system" fn(capture_option: CaptureOption) -> u32;
pub type FnGetCaptureOptionF32 = extern "system" fn(capture_option: CaptureOption) -> f32;
pub type FnSetFocusToggleKeys = extern "system" fn(keys: *const InputButton, num_keys: i32);
pub type FnSetCaptureKeys = extern "system" fn(keys: *const InputButton, num_keys: i32);
pub type FnGetOverlayBits = extern "system" fn() -> OverlayBits;
pub type FnMaskOverlayBits = extern "system" fn(and: OverlayBits, or: OverlayBits);
pub type FnRemoveHooks = extern "system" fn();
pub type FnUnloadCrashHandler = extern "system" fn();
pub type FnSetCaptureFilePathTemplate = extern "system" fn(path_template: *const c_char);
pub type FnGetCaptureFilePathTemplate = extern "system" fn() -> *const c_char;
pub type FnGetNumCaptures = extern "system" fn() -> u32;
pub type FnGetCapture = extern "system" fn(
    index: u32,
    filename: *mut c_char,
    path_length: Option<&mut u32>,
    timestamp: Option<&mut u64>,
) -> u32;
pub type FnTriggerCapture = extern "system" fn();
pub type FnTriggerMultiFrameCapture = extern "system" fn(num_frames: u32);
pub type FnIsTargetControlConnected = extern "system" fn() -> u32;
pub type FnLaunchReplayUI =
    extern "system" fn(connect_target_control: u32, command_line: *const c_char) -> u32;
pub type FnSetActiveWindow = extern "system" fn(device: DevicePointer, window: WindowHandle);
pub type FnStartFrameCapture = extern "system" fn(device: DevicePointer, window: WindowHandle);
pub type FnIsFrameCapturing = extern "system" fn() -> u32;
pub type FnEndFrameCapture = extern "system" fn(device: DevicePointer, window: WindowHandle) -> u32;
pub type FnDiscardFrameCapture =
    extern "system" fn(device: DevicePointer, window: WindowHandle) -> u32;
pub type FnSetCaptureFileComments =
    extern "system" fn(filepath: *const c_char, comments: *const c_char);
pub type FnShowReplayUI = extern "system" fn() -> u32;

#[repr(C)]
pub struct RenderdocApi1_5_0 {
    get_api_version: FnGetApiVersion,
    set_capture_option_u32: FnSetCaptureOptionU32,
    set_capture_option_f32: FnSetCaptureOptionF32,
    get_capture_option_u32: FnGetCaptureOptionU32,
    get_capture_option_f32: FnGetCaptureOptionF32,
    set_focus_toggle_keys: FnSetFocusToggleKeys,
    set_capture_keys: FnSetCaptureKeys,
    get_overlay_bits: FnGetOverlayBits,
    mask_overlay_bits: FnMaskOverlayBits,
    remove_hooks: FnRemoveHooks,
    unload_crash_handler: FnUnloadCrashHandler,
    set_capture_file_path_template: FnSetCaptureFilePathTemplate,
    get_capture_file_path_template: FnGetCaptureFilePathTemplate,
    get_num_captures: FnGetNumCaptures,
    get_capture: FnGetCapture,
    trigger_capture: FnTriggerCapture,
    is_target_control_connected: FnIsTargetControlConnected,
    launch_replay_ui: FnLaunchReplayUI,
    set_active_window: FnSetActiveWindow,
    start_frame_capture: FnStartFrameCapture,
    is_frame_capturing: FnIsFrameCapturing,
    end_frame_capture: FnEndFrameCapture,
    trigger_multi_frame_capture: FnTriggerMultiFrameCapture,
    set_capture_file_comments: FnSetCaptureFileComments,
    discard_frame_capture: FnDiscardFrameCapture,
    show_replay_ui: FnShowReplayUI,
}

impl RenderdocApi1_5_0 {
    pub fn load() -> Option<Self> {
        unsafe {
            let module = libc::dlopen(
                cstr!("librenderdoc.so").as_ptr(),
                libc::RTLD_NOW | libc::RTLD_NOLOAD,
            );
            if module.is_null() {
                return None;
            }
            let get_api_ptr = libc::dlsym(module, cstr!("RENDERDOC_GetAPI").as_ptr());
            if get_api_ptr.is_null() {
                return None;
            }
            let get_api = std::mem::transmute::<_, FnGetApi>(get_api_ptr);

            let mut rdoc_api = MaybeUninit::<Self>::uninit();
            let ret = get_api(
                Version::Version1_5_0,
                rdoc_api.as_mut_ptr() as *mut *mut c_void,
            );
            if ret == 0 {
                return None;
            }
            Some(rdoc_api.assume_init())
        }
    }

    /// RenderDoc can return a higher version than requested if it's backwards compatible,
    /// this function returns the actual version returned.
    pub fn get_api_version(&self) -> (i32, i32, i32) {
        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;
        (self.get_api_version)(&mut major, &mut minor, &mut patch);
        (major, minor, patch)
    }

    /// Sets an option that controls how RenderDoc behaves on capture.
    ///
    /// Returns true if the option and value are valid
    /// Returns false if either is invalid and the option is unchanged
    pub fn set_capture_option_f32(&self, option: CaptureOption, value: f32) -> bool {
        (self.set_capture_option_f32)(option, value) == 1
    }

    /// Sets an option that controls how RenderDoc behaves on capture.
    ///
    /// Returns true if the option and value are valid
    /// Returns false if either is invalid and the option is unchanged
    pub fn set_capture_option_u32(&self, option: CaptureOption, value: u32) -> bool {
        (self.set_capture_option_u32)(option, value) == 1
    }

    /// Gets an option that controls how RenderDoc behaves on capture.
    ///
    /// If the option is invalid, -FLT_MAX is returned
    pub fn get_capture_option_f32(&self, option: CaptureOption) -> f32 {
        (self.get_capture_option_f32)(option)
    }

    /// Gets an option that controls how RenderDoc behaves on capture.
    ///
    /// If the option is invalid, 0xffffffff is returned
    pub fn get_capture_option_u32(&self, option: CaptureOption) -> u32 {
        (self.get_capture_option_u32)(option)
    }

    /// Sets which key or keys can be used to toggle focus between multiple windows.
    ///
    /// If slice is empty toggle keys will be disabled
    pub fn set_focus_toggle_keys(&self, keys: &[InputButton]) {
        (self.set_focus_toggle_keys)(keys.as_ptr(), keys.len() as i32)
    }

    /// Sets which key or keys can be used to capture the next frame.
    ///
    /// If slice is empty capture keys will be disabled
    pub fn set_capture_keys(&self, keys: &[InputButton]) {
        (self.set_capture_keys)(keys.as_ptr(), keys.len() as i32)
    }

    /// Returns the overlay bits that have been set
    pub fn get_overlay_bits(&self) -> OverlayBits {
        (self.get_overlay_bits)()
    }

    /// sets the overlay bits with an and & or mask
    pub fn mask_overlay_bits(&self, and: OverlayBits, or: OverlayBits) {
        (self.mask_overlay_bits)(and, or)
    }

    /// Attempt to remove RenderDoc's hooks in the application.
    ///
    /// Note: that this can only work correctly if done immediately after
    /// the module is loaded, before any API work happens. RenderDoc will remove its
    /// injected hooks and shut down. Behaviour is undefined if this is called
    /// after any API functions have been called, and there is still no guarantee of
    /// success.
    pub fn remove_hooks(&self) {
        (self.remove_hooks)()
    }

    /// Unload RenderDoc's crash handler.
    ///
    /// If you use your own crash handler and don't want RenderDoc's handler to
    /// intercede, you can call this function to unload it and any unhandled
    /// exceptions will pass to the next handler.
    pub fn unload_crash_handler(&self) {
        (self.unload_crash_handler)()
    }

    /// Sets the capture file path template
    ///
    /// pathtemplate is a UTF-8 string that gives a template for how captures will be named
    /// and where they will be saved.
    ///
    /// Any extension is stripped off the path, and captures are saved in the directory
    /// specified, and named with the filename and the frame number appended. If the
    /// directory does not exist it will be created, including any parent directories.
    ///
    /// If pathtemplate is NULL, the template will remain unchanged
    ///
    /// Example:
    ///
    /// SetCaptureFilePathTemplate("my_captures/example");
    ///
    /// Capture #1 -> my_captures/example_frame123.rdc
    /// Capture #2 -> my_captures/example_frame456.rdc
    pub fn set_capture_file_path_template(&self, path_template: &str) {
        let path_template = CString::new(path_template).unwrap();
        (self.set_capture_file_path_template)(path_template.as_ptr())
    }

    /// Gets the capture file path template
    pub fn get_capture_file_path_template(&self) -> String {
        let ptr = (self.get_capture_file_path_template)();
        let str = unsafe { CStr::from_ptr(ptr) };
        str.to_owned().into_string().unwrap()
    }

    /// Returns the number of captures that have been made
    pub fn get_num_captures(&self) -> u32 {
        (self.get_num_captures)()
    }

    /// This function returns the details of a capture, by index. New captures are added
    /// to the end of the list.
    ///
    /// Returns the absolute path of the capture file and the time of capture in seconds since the
    /// unix epoch, or None, if the capture index is invalid.
    ///
    /// Note: when captures are deleted in the UI they will remain in this list, so the
    /// capture path may not exist anymore.
    pub fn get_capture(&self, index: u32) -> Option<(String, u64)> {
        let mut path_length = 0;
        let mut timestamp = 0;
        let ret = (self.get_capture)(
            index,
            std::ptr::null_mut(),
            Some(&mut path_length),
            Some(&mut timestamp),
        );
        if ret == 0 || path_length <= 1 {
            return None;
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(path_length as usize - 1);
        (self.get_capture)(index, bytes.as_mut_ptr() as *mut c_char, None, None);
        unsafe { bytes.set_len(path_length as usize - 1) };
        Some((String::from_utf8(bytes).unwrap(), timestamp))
    }

    /// Capture the next frame on whichever window and API is currently considered active.
    pub fn trigger_capture(&self) {
        (self.trigger_capture)()
    }

    /// Capture the next N frames on whichever window and API is currently considered active.
    pub fn trigger_multi_frame_capture(&self, num_frames: u32) {
        (self.trigger_multi_frame_capture)(num_frames)
    }

    /// Returns true if the RenderDoc UI is connected to this application, false otherwise
    pub fn is_target_control_connected(&self) -> bool {
        (self.is_target_control_connected)() == 1
    }

    /// This function will launch the Replay UI associated with the RenderDoc library injected
    /// into the running application.
    ///
    /// If connect_target_control is true, the Replay UI will be launched with a command line parameter
    /// to connect to this application
    /// command_line is an optional string containing the rest of the command line. E.g. a captures to open
    ///
    /// Returns the PID of the replay UI if successful, 0 if not successful.
    pub fn launch_replay_ui(
        &self,
        connect_target_control: bool,
        command_line: Option<&str>,
    ) -> u32 {
        let command_line = command_line.map(|s| CString::new(s).unwrap());
        (self.launch_replay_ui)(
            connect_target_control as u32,
            command_line.map_or(std::ptr::null(), |s| s.as_ptr()),
        )
    }

    /// This sets the RenderDoc in-app overlay in the API/window pair as 'active' and it will
    /// respond to keypresses. Neither parameter can be NULL
    pub fn set_active_window(&self, device: DevicePointer, window: WindowHandle) {
        (self.set_active_window)(device, window)
    }

    /// When choosing either a device pointer or a window handle to capture, you can pass NULL.
    /// Passing NULL specifies a 'wildcard' match against anything. This allows you to specify
    /// any API rendering to a specific window, or a specific API instance rendering to any window,
    /// or in the simplest case of one window and one API, you can just pass NULL for both.
    ///
    /// In either case, if there are two or more possible matching (device,window) pairs it
    /// is undefined which one will be captured.
    ///
    /// Note: for headless rendering you can pass NULL for the window handle and either specify
    /// a device pointer or leave it NULL as above.
    ///
    /// Immediately starts capturing API calls on the specified device pointer and window handle.
    ///
    /// If there is no matching thing to capture (e.g. no supported API has been initialised),
    /// this will do nothing.
    ///
    /// The results are undefined (including crashes) if two captures are started overlapping,
    /// even on separate devices and/oror windows.
    pub fn start_frame_capture(&self, device: DevicePointer, window: WindowHandle) {
        (self.start_frame_capture)(device, window)
    }

    /// Returns whether or not a frame capture is currently ongoing anywhere.
    pub fn is_frame_capturing(&self) -> bool {
        (self.is_frame_capturing)() == 1
    }

    /// Ends capturing immediately.
    ///
    /// This will return true if the capture succeeded, and false if there was an error capturing.
    pub fn end_frame_capture(&self, device: DevicePointer, window: WindowHandle) -> bool {
        (self.end_frame_capture)(device, window) == 1
    }

    /// Ends capturing immediately and discard any data stored without saving to disk.
    /// This will return true if the capture was discarded, and false if there was an error or no capture
    /// was in progress
    pub fn discard_frame_capture(&self, device: DevicePointer, window: WindowHandle) -> bool {
        (self.end_frame_capture)(device, window) == 1
    }

    /// Sets the comments associated with a capture file. These comments are displayed in the
    /// UI program when opening.
    ///
    /// file_path should be a path to the capture file to add comments to. If None the most recent capture
    /// file created made will be used instead.
    /// comments should be a string to add as comments.
    ///
    /// Any existing comments will be overwritten.
    pub fn set_capture_file_comments(&self, file_path: Option<&str>, comments: &str) {
        let file_path = file_path.map(|s| CString::new(s).unwrap());
        let comments = CString::new(comments).unwrap();

        (self.set_capture_file_comments)(
            file_path.map_or(std::ptr::null(), |s| s.as_ptr()),
            comments.as_ptr(),
        )
    }

    /// Requests that the replay UI show itself (if hidden or not the current top window). This can be
    /// used in conjunction with IsTargetControlConnected and LaunchReplayUI to intelligently handle
    /// showing the UI after making a capture.
    ///
    /// This will return true if the request was successfully passed on, though it's not guaranteed that
    /// the UI will be on top in all cases depending on OS rules. It will return false if there is no
    /// current target control connection to make such a request, or if there was another error
    pub fn show_replay_ui(&self) -> bool {
        (self.show_replay_ui)() == 1
    }
}
