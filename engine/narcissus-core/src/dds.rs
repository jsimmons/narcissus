#![allow(non_camel_case_types)]

use std::mem::size_of;

use crate::{FourCC, flags_def, fourcc};

const DDS_FOURCC: FourCC = fourcc!("DDS ");
const DX10_FOURCC: FourCC = fourcc!("DX10");

#[derive(Clone, Copy, Debug)]
pub enum LoadError {
    TooSmall,
    BadMagic,
    BadHeader,
}

#[derive(Debug)]
pub struct Dds<'a> {
    pub header: DdsHeader,
    pub header_dxt10: Option<DdsHeaderDxt10>,
    pub data: &'a [u8],
}

impl<'a> Dds<'a> {
    pub fn from_buffer(buf: &'a [u8]) -> Result<Self, LoadError> {
        if buf.len() < 4 + size_of::<DdsHeader>() {
            return Err(LoadError::TooSmall);
        }

        let magic: [u8; 4] = buf[..4].try_into().unwrap();
        let magic: FourCC = magic.into();
        if magic != DDS_FOURCC {
            return Err(LoadError::BadMagic);
        }

        let header = DdsHeader::from_bytes(buf[4..4 + size_of::<DdsHeader>()].try_into().unwrap());

        if header.size != size_of::<DdsHeader>() as u32
            || header.pixel_format.size != size_of::<DdsPixelFormat>() as u32
        {
            return Err(LoadError::BadHeader);
        }

        let header_dxt10 = if header
            .pixel_format
            .flags
            .contains(DdsPixelFormatFlags::FOURCC)
            && header.pixel_format.four_cc == DX10_FOURCC
        {
            if buf.len() < 4 + size_of::<DdsHeader>() + size_of::<DdsHeaderDxt10>() {
                return Err(LoadError::TooSmall);
            }

            Some(DdsHeaderDxt10::from_bytes(
                buf[128..128 + size_of::<DdsHeaderDxt10>()]
                    .try_into()
                    .unwrap(),
            ))
        } else {
            None
        };

        let offset = if header_dxt10.is_some() {
            4 + size_of::<DdsHeader>() + size_of::<DdsHeaderDxt10>()
        } else {
            4 + size_of::<DdsHeader>()
        };

        Ok(Dds {
            header,
            header_dxt10,
            data: &buf[offset..],
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct DdsHeader {
    size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mip_map_count: u32,
    _reserved: [u32; 11],
    pub pixel_format: DdsPixelFormat,
    pub caps: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub caps4: u32,
    _reserved2: u32,
}

impl DdsHeader {
    fn from_bytes(buf: [u8; size_of::<DdsHeader>()]) -> Self {
        // SAFETY:
        // * Array is the exact size of DdsHeader.
        // * All bit patterns are valid for DdsHeader.
        // * We use transmute_copy to resolve alignment concerns.
        unsafe { std::mem::transmute_copy::<[u8; size_of::<DdsHeader>()], DdsHeader>(&buf) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct DdsHeaderDxt10 {
    pub dxgi_format: DxgiFormat,
    pub resource_dimension: D3D10ResourceDimension,
    pub misc_flags: u32,
    pub array_size: u32,
    pub misc_flags2: u32,
}

impl DdsHeaderDxt10 {
    fn from_bytes(buf: [u8; size_of::<DdsHeaderDxt10>()]) -> Self {
        // SAFETY:
        // * Array is the exact size of DdsHeaderDxt10.
        // * All bit patterns are valid for DdsHeaderDxt10.
        // * We use transmute_copy to resolve alignment concerns.
        unsafe {
            std::mem::transmute_copy::<[u8; size_of::<DdsHeaderDxt10>()], DdsHeaderDxt10>(&buf)
        }
    }
}

flags_def!(DdsPixelFormatFlags);

impl DdsPixelFormatFlags {
    pub const ALPHA_PIXELS: Self = Self(0x1);
    pub const ALPHA: Self = Self(0x2);
    pub const FOURCC: Self = Self(0x4);
    pub const RGB: Self = Self(0x40);
    pub const YUV: Self = Self(0x200);
    pub const LUMINANCE: Self = Self(0x20000);
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct DdsPixelFormat {
    size: u32,
    pub flags: DdsPixelFormatFlags,
    pub four_cc: FourCC,
    pub rgb_bit_count: u32,
    pub r_bit_mask: u32,
    pub g_bit_mask: u32,
    pub b_bit_mask: u32,
    pub a_bit_mask: u32,
}

#[repr(u32)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub enum DxgiFormat {
    #[default]
    UNKNOWN = 0,
    R32G32B32A32_TYPELESS = 1,
    R32G32B32A32_FLOAT = 2,
    R32G32B32A32_UINT = 3,
    R32G32B32A32_SINT = 4,
    R32G32B32_TYPELESS = 5,
    R32G32B32_FLOAT = 6,
    R32G32B32_UINT = 7,
    R32G32B32_SINT = 8,
    R16G16B16A16_TYPELESS = 9,
    R16G16B16A16_FLOAT = 10,
    R16G16B16A16_UNORM = 11,
    R16G16B16A16_UINT = 12,
    R16G16B16A16_SNORM = 13,
    R16G16B16A16_SINT = 14,
    R32G32_TYPELESS = 15,
    R32G32_FLOAT = 16,
    R32G32_UINT = 17,
    R32G32_SINT = 18,
    R32G8X24_TYPELESS = 19,
    D32_FLOAT_S8X24_UINT = 20,
    R32_FLOAT_X8X24_TYPELESS = 21,
    X32_TYPELESS_G8X24_UINT = 22,
    R10G10B10A2_TYPELESS = 23,
    R10G10B10A2_UNORM = 24,
    R10G10B10A2_UINT = 25,
    R11G11B10_FLOAT = 26,
    R8G8B8A8_TYPELESS = 27,
    R8G8B8A8_UNORM = 28,
    R8G8B8A8_UNORM_SRGB = 29,
    R8G8B8A8_UINT = 30,
    R8G8B8A8_SNORM = 31,
    R8G8B8A8_SINT = 32,
    R16G16_TYPELESS = 33,
    R16G16_FLOAT = 34,
    R16G16_UNORM = 35,
    R16G16_UINT = 36,
    R16G16_SNORM = 37,
    R16G16_SINT = 38,
    R32_TYPELESS = 39,
    D32_FLOAT = 40,
    R32_FLOAT = 41,
    R32_UINT = 42,
    R32_SINT = 43,
    R24G8_TYPELESS = 44,
    D24_UNORM_S8_UINT = 45,
    R24_UNORM_X8_TYPELESS = 46,
    X24_TYPELESS_G8_UINT = 47,
    R8G8_TYPELESS = 48,
    R8G8_UNORM = 49,
    R8G8_UINT = 50,
    R8G8_SNORM = 51,
    R8G8_SINT = 52,
    R16_TYPELESS = 53,
    R16_FLOAT = 54,
    D16_UNORM = 55,
    R16_UNORM = 56,
    R16_UINT = 57,
    R16_SNORM = 58,
    R16_SINT = 59,
    R8_TYPELESS = 60,
    R8_UNORM = 61,
    R8_UINT = 62,
    R8_SNORM = 63,
    R8_SINT = 64,
    A8_UNORM = 65,
    R1_UNORM = 66,
    R9G9B9E5_SHAREDEXP = 67,
    R8G8_B8G8_UNORM = 68,
    G8R8_G8B8_UNORM = 69,
    BC1_TYPELESS = 70,
    BC1_UNORM = 71,
    BC1_UNORM_SRGB = 72,
    BC2_TYPELESS = 73,
    BC2_UNORM = 74,
    BC2_UNORM_SRGB = 75,
    BC3_TYPELESS = 76,
    BC3_UNORM = 77,
    BC3_UNORM_SRGB = 78,
    BC4_TYPELESS = 79,
    BC4_UNORM = 80,
    BC4_SNORM = 81,
    BC5_TYPELESS = 82,
    BC5_UNORM = 83,
    BC5_SNORM = 84,
    B5G6R5_UNORM = 85,
    B5G5R5A1_UNORM = 86,
    B8G8R8A8_UNORM = 87,
    B8G8R8X8_UNORM = 88,
    R10G10B10_XR_BIAS_A2_UNORM = 89,
    B8G8R8A8_TYPELESS = 90,
    B8G8R8A8_UNORM_SRGB = 91,
    B8G8R8X8_TYPELESS = 92,
    B8G8R8X8_UNORM_SRGB = 93,
    BC6H_TYPELESS = 94,
    BC6H_UF16 = 95,
    BC6H_SF16 = 96,
    BC7_TYPELESS = 97,
    BC7_UNORM = 98,
    BC7_UNORM_SRGB = 99,
    AYUV = 100,
    Y410 = 101,
    Y416 = 102,
    NV12 = 103,
    P010 = 104,
    P016 = 105,
    YUV_420_OPAQUE = 106,
    YUY2 = 107,
    Y210 = 108,
    Y216 = 109,
    NV11 = 110,
    AI44 = 111,
    IA44 = 112,
    P8 = 113,
    A8P8 = 114,
    B4G4R4A4_UNORM = 115,
    P208 = 130,
    V208 = 131,
    V408 = 132,
    SAMPLER_FEEDBACK_MIN_MIP_OPAQUE,
    SAMPLER_FEEDBACK_MIP_REGION_USED_OPAQUE,
    FORCE_UINT = 0xffffffff,
}

#[repr(u32)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub enum D3D10ResourceDimension {
    #[default]
    Unknown = 0,
    Buffer = 1,
    Texture1d = 2,
    Texture2d = 3,
    Texture3d = 4,
}
