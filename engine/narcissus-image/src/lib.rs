use std::ptr::NonNull;

use stb_image_sys::{stbi_image_free, stbi_load_from_memory};

#[derive(Debug)]
pub struct LoadError;

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadError").finish()
    }
}

impl std::error::Error for LoadError {}

pub struct Image {
    width: usize,
    height: usize,
    components: usize,
    len: usize,
    buffer: NonNull<u8>,
}

impl Image {
    pub fn from_buffer(buffer: &[u8]) -> Result<Image, LoadError> {
        let mut x = 0;
        let mut y = 0;
        let mut components = 0;
        let required_components = 0;
        let buffer = unsafe {
            stbi_load_from_memory(
                buffer.as_ptr(),
                buffer.len() as i32,
                &mut x,
                &mut y,
                &mut components,
                required_components,
            )
        };

        if buffer.is_null() {
            return Err(LoadError);
        }

        let x = x as usize;
        let y = y as usize;
        let components = components as usize;
        let len = x * y * components;

        Ok(Image {
            width: x,
            height: y,
            components,
            len,
            // SAFETY: We just checked that buffer is not null above.
            buffer: unsafe { NonNull::new_unchecked(buffer) },
        })
    }

    /// Returns the texture's width in pixels.
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the texture's height in pixels.
    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the number of components in this texture.
    #[inline]
    pub fn components(&self) -> usize {
        self.components
    }

    /// The pixel data consists of [`Self::height()`] scanlines of [`Self::width()`] pixels,
    /// with each pixel consisting of [`Self::components()`] interleaved 8-bit components; the first
    /// pixel pointed to is top-left-most in the texture. There is no padding between
    /// texture scanlines or between pixels, regardless of format.
    ///
    /// An output texture with N components has the following components interleaved
    /// in this order in each pixel:
    ///
    /// |  N |   Components            |
    /// |----|-------------------------|
    /// | 1  | grey                    |
    /// | 2  | grey, alpha             |
    /// | 3  | red, green, blue        |
    /// | 4  | red, green, blue, alpha |
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: Slice size is calculated when creating `Texture`.
        unsafe { std::slice::from_raw_parts(self.buffer.as_ptr(), self.len) }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        // SAFETY: Always allocated by `stbi_load_xxx` functions.
        unsafe { stbi_image_free(self.buffer.as_ptr() as *mut _) }
    }
}
