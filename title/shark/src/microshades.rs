//! microshades
//! ===
//!
//! The microshades package is designed to provide custom color shading palettes
//! that improve accessibility and data organization. Approximately 300 million
//! people in the world have Color Vision Deficiency (CVD), which is comparable
//! to the most recent estimate of the US population. When creating figures and
//! graphics that use color, it is important to consider that individuals with
//! CVD will interact with this material, and may not perceive all of the
//! information tied to the colors as intended. This package includes carefully
//! crafted palettes that improve CVD accessibility and may be applied to any
//! plot.
//!
//! <https://karstenslab.github.io/project/microshades/>

pub const GRAY_RGBA8: [u32; 5] = [0xd9d9_d9ff, 0xbdbdbdff, 0x969696ff, 0x737373ff, 0x525252ff];
pub const BROWN_RGBA8: [u32; 5] = [0xd8c7beff, 0xcaa995ff, 0xb78560ff, 0x9e5c00ff, 0x7d3200ff];
pub const GREEN_RGBA8: [u32; 5] = [0xc7e9c0ff, 0xa1d99bff, 0x74c476ff, 0x41ab5dff, 0x238b45ff];
pub const ORANGE_RGBA8: [u32; 5] = [0xfeeda0ff, 0xfec44fff, 0xfdae6bff, 0xfe9929ff, 0xff7f00ff];
pub const BLUE_RGBA8: [u32; 5] = [0xeff3ffff, 0xc6dbefff, 0x9ecae1ff, 0x6baed6ff, 0x4292c6ff];
pub const PURPLE_RGBA8: [u32; 5] = [0xdadaebff, 0xbcbddcff, 0x9e9ac8ff, 0x807dbaff, 0x6a51a3ff];

pub const GRAY_RGBA_F32: [[f32; 4]; 5] = [
    [0.8509804, 0.8509804, 0.8509804, 1.0],
    [0.7411765, 0.7411765, 0.7411765, 1.0],
    [0.5882353, 0.5882353, 0.5882353, 1.0],
    [0.4509804, 0.4509804, 0.4509804, 1.0],
    [0.32156864, 0.32156864, 0.32156864, 1.0],
];

pub const BROWN_RGBA_F32: [[f32; 4]; 5] = [
    [0.84705883, 0.78039217, 0.74509805, 1.0],
    [0.7921569, 0.6627451, 0.58431375, 1.0],
    [0.7176471, 0.52156866, 0.3764706, 1.0],
    [0.61960787, 0.36078432, 0.0, 1.0],
    [0.49019608, 0.19607843, 0.0, 1.0],
];

pub const GREEN_RGBA_F32: [[f32; 4]; 5] = [
    [0.78039217, 0.9137255, 0.7529412, 1.0],
    [0.6313726, 0.8509804, 0.60784316, 1.0],
    [0.45490196, 0.76862746, 0.4627451, 1.0],
    [0.25490198, 0.67058825, 0.3647059, 1.0],
    [0.13725491, 0.54509807, 0.27058825, 1.0],
];

pub const ORANGE_RGBA_F32: [[f32; 4]; 5] = [
    [0.99607843, 0.92941177, 0.627451, 1.0],
    [0.99607843, 0.76862746, 0.30980393, 1.0],
    [0.99215686, 0.68235296, 0.41960785, 1.0],
    [0.99607843, 0.6, 0.16078432, 1.0],
    [1.0, 0.49803922, 0.0, 1.0],
];

pub const BLUE_RGBA_F32: [[f32; 4]; 5] = [
    [0.9372549, 0.9529412, 1.0, 1.0],
    [0.7764706, 0.85882354, 0.9372549, 1.0],
    [0.61960787, 0.7921569, 0.88235295, 1.0],
    [0.41960785, 0.68235296, 0.8392157, 1.0],
    [0.25882354, 0.57254905, 0.7764706, 1.0],
];

pub const PURPLE_RGBA_F32: [[f32; 4]; 5] = [
    [0.85490197, 0.85490197, 0.92156863, 1.0],
    [0.7372549, 0.7411765, 0.8627451, 1.0],
    [0.61960787, 0.6039216, 0.78431374, 1.0],
    [0.5019608, 0.49019608, 0.7294118, 1.0],
    [0.41568628, 0.31764707, 0.6392157, 1.0],
];
