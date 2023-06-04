use narcissus_gpu::{Buffer, BufferDesc, BufferUsageFlags, Device, MemoryLocation};

use crate::Blittable;

pub struct MappedBuffer<'a> {
    device: &'a dyn Device,
    buffer: Buffer,
    slice: &'a mut [u8],
}

impl<'a> MappedBuffer<'a> {
    pub fn new(device: &'a dyn Device, usage: BufferUsageFlags, len: usize) -> Self {
        let buffer = device.create_buffer(&BufferDesc {
            location: MemoryLocation::HostMapped,
            usage,
            size: len,
        });
        unsafe {
            let ptr = device.map_buffer(buffer);
            let slice = std::slice::from_raw_parts_mut(ptr, len);
            Self {
                device,
                buffer,
                slice,
            }
        }
    }

    pub fn buffer(&self) -> Buffer {
        self.buffer
    }

    pub fn write<T>(&mut self, value: T)
    where
        T: Blittable,
    {
        unsafe {
            let src = std::slice::from_raw_parts(
                &value as *const T as *const u8,
                std::mem::size_of::<T>(),
            );
            self.slice.copy_from_slice(src)
        }
    }

    pub fn write_slice<T>(&mut self, values: &[T])
    where
        T: Blittable,
    {
        unsafe {
            let len = std::mem::size_of_val(values);
            let src = std::slice::from_raw_parts(values.as_ptr() as *const u8, len);
            self.slice[..len].copy_from_slice(src)
        }
    }
}

impl<'a> Drop for MappedBuffer<'a> {
    fn drop(&mut self) {
        // SAFETY: Make sure we don't have the slice outlive the mapping.
        unsafe {
            self.device.unmap_buffer(self.buffer);
        }
    }
}
