use std::collections::HashSet;

use narcissus_core::{default, Mutex, Widen};
use vulkan_sys as vk;

use crate::{
    backend::vulkan::VULKAN_CONSTANTS,
    tlsf::{self, Tlsf},
    vk_check, MemoryLocation,
};

use super::{VulkanDevice, VulkanFrame};

#[derive(Default)]
pub struct VulkanAllocator {
    tlsf: Mutex<Tlsf<VulkanAllocationInfo>>,
    dedicated: Mutex<HashSet<vk::DeviceMemory>>,
}

#[derive(Clone, Copy)]
pub struct VulkanAllocationInfo {
    memory: vk::DeviceMemory,
    mapped_ptr: *mut u8,
}

pub enum VulkanMemoryDedicatedDesc {
    Image(vk::Image),
    Buffer(vk::Buffer),
}

pub struct VulkanMemoryDesc {
    pub requirements: vk::MemoryRequirements,
    pub memory_location: MemoryLocation,
    pub _linear: bool,
}

#[derive(Clone)]
pub struct VulkanMemoryDedicated {
    memory: vk::DeviceMemory,
    mapped_ptr: *mut u8,
    size: u64,
    memory_type_index: u32,
}

#[derive(Clone)]
pub struct VulkanMemorySubAlloc {
    allocation: tlsf::Allocation<VulkanAllocationInfo>,
    size: u64,
    memory_type_index: u32,
}

#[derive(Clone)]
pub enum VulkanMemory {
    Dedicated(VulkanMemoryDedicated),
    SubAlloc(VulkanMemorySubAlloc),
}

impl VulkanMemory {
    #[inline(always)]
    pub fn device_memory(&self) -> vk::DeviceMemory {
        match self {
            VulkanMemory::Dedicated(dedicated) => dedicated.memory,
            VulkanMemory::SubAlloc(sub_alloc) => sub_alloc.allocation.user_data().memory,
        }
    }

    #[inline(always)]
    pub fn offset(&self) -> u64 {
        match self {
            VulkanMemory::Dedicated(_) => 0,
            VulkanMemory::SubAlloc(sub_alloc) => sub_alloc.allocation.offset(),
        }
    }

    #[inline(always)]
    pub fn size(&self) -> u64 {
        match self {
            VulkanMemory::Dedicated(dedicated) => dedicated.size,
            VulkanMemory::SubAlloc(sub_alloc) => sub_alloc.size,
        }
    }

    #[inline(always)]
    pub fn mapped_ptr(&self) -> *mut u8 {
        match self {
            VulkanMemory::Dedicated(dedicated) => dedicated.mapped_ptr,
            VulkanMemory::SubAlloc(sub_alloc) => {
                let user_data = sub_alloc.allocation.user_data();
                if user_data.mapped_ptr.is_null() {
                    std::ptr::null_mut()
                } else {
                    user_data
                        .mapped_ptr
                        .wrapping_add(sub_alloc.allocation.offset() as usize)
                }
            }
        }
    }
}

impl VulkanDevice {
    pub fn find_memory_type_index(&self, filter: u32, flags: vk::MemoryPropertyFlags) -> u32 {
        (0..self.physical_device_memory_properties.memory_type_count)
            .map(|memory_type_index| {
                (
                    memory_type_index,
                    self.physical_device_memory_properties.memory_types[memory_type_index.widen()],
                )
            })
            .find(|(i, memory_type)| {
                (filter & (1 << i)) != 0 && memory_type.property_flags.contains(flags)
            })
            .expect("could not find memory type matching flags")
            .0
    }

    pub fn allocate_memory_dedicated(
        &self,
        desc: &VulkanMemoryDesc,
        dedicated_desc: &VulkanMemoryDedicatedDesc,
    ) -> VulkanMemory {
        let memory_property_flags = match desc.memory_location {
            MemoryLocation::HostMapped => vk::MemoryPropertyFlags::HOST_VISIBLE,
            MemoryLocation::Device => vk::MemoryPropertyFlags::DEVICE_LOCAL,
        };

        let memory_type_index =
            self.find_memory_type_index(desc.requirements.memory_type_bits, memory_property_flags);

        let allocator = self.allocators[memory_type_index.widen()]
            .as_ref()
            .expect("returned a memory type index that has no associated allocator");

        let mut allocate_info = vk::MemoryAllocateInfo {
            allocation_size: desc.requirements.size,
            memory_type_index,
            ..default()
        };

        let mut dedicated_allocate_info = vk::MemoryDedicatedAllocateInfo::default();

        match *dedicated_desc {
            VulkanMemoryDedicatedDesc::Image(image) => {
                dedicated_allocate_info.image = image;
            }
            VulkanMemoryDedicatedDesc::Buffer(buffer) => dedicated_allocate_info.buffer = buffer,
        }
        allocate_info._next =
            &dedicated_allocate_info as *const vk::MemoryDedicatedAllocateInfo as *const _;

        let mut memory = vk::DeviceMemory::null();
        vk_check!(self
            .device_fn
            .allocate_memory(self.device, &allocate_info, None, &mut memory));

        allocator.dedicated.lock().insert(memory);

        let mapped_ptr = if self.physical_device_memory_properties.memory_types
            [memory_type_index.widen()]
        .property_flags
        .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        {
            let mut data = std::ptr::null_mut();
            vk_check!(self.device_fn.map_memory(
                self.device,
                memory,
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::default(),
                &mut data
            ));
            data as *mut u8
        } else {
            std::ptr::null_mut()
        };

        VulkanMemory::Dedicated(VulkanMemoryDedicated {
            memory,
            mapped_ptr,
            size: desc.requirements.size,
            memory_type_index,
        })
    }

    pub fn allocate_memory(&self, desc: &VulkanMemoryDesc) -> VulkanMemory {
        let memory_property_flags = match desc.memory_location {
            MemoryLocation::HostMapped => vk::MemoryPropertyFlags::HOST_VISIBLE,
            MemoryLocation::Device => vk::MemoryPropertyFlags::DEVICE_LOCAL,
        };

        let memory_type_index =
            self.find_memory_type_index(desc.requirements.memory_type_bits, memory_property_flags);

        let allocator = self.allocators[memory_type_index.widen()]
            .as_ref()
            .expect("returned a memory type index that has no associated allocator");

        let mut tlsf = allocator.tlsf.lock();

        let allocation = {
            if let Some(allocation) =
                tlsf.alloc(desc.requirements.size, desc.requirements.alignment)
            {
                allocation
            } else {
                let allocate_info = vk::MemoryAllocateInfo {
                    allocation_size: VULKAN_CONSTANTS.tlsf_block_size,
                    memory_type_index,
                    ..default()
                };

                let mut memory = vk::DeviceMemory::null();
                vk_check!(self.device_fn.allocate_memory(
                    self.device,
                    &allocate_info,
                    None,
                    &mut memory
                ));

                let mapped_ptr = if self.physical_device_memory_properties.memory_types
                    [memory_type_index.widen()]
                .property_flags
                .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
                {
                    let mut data = std::ptr::null_mut();
                    vk_check!(self.device_fn.map_memory(
                        self.device,
                        memory,
                        0,
                        vk::WHOLE_SIZE,
                        vk::MemoryMapFlags::default(),
                        &mut data
                    ));
                    data as *mut u8
                } else {
                    std::ptr::null_mut()
                };

                tlsf.insert_super_block(
                    VULKAN_CONSTANTS.tlsf_block_size,
                    VulkanAllocationInfo { memory, mapped_ptr },
                );

                tlsf.alloc(desc.requirements.size, desc.requirements.alignment)
                    .expect("failed to allocate")
            }
        };

        VulkanMemory::SubAlloc(VulkanMemorySubAlloc {
            allocation,
            size: desc.requirements.size,
            memory_type_index,
        })
    }

    pub fn allocator_begin_frame(&self, frame: &mut VulkanFrame) {
        for allocation in frame.destroyed_allocations.get_mut().drain(..) {
            match allocation {
                VulkanMemory::Dedicated(dedicated) => {
                    let allocator = self.allocators[dedicated.memory_type_index.widen()]
                        .as_ref()
                        .unwrap();
                    allocator.dedicated.lock().remove(&dedicated.memory);
                    unsafe {
                        self.device_fn
                            .free_memory(self.device, dedicated.memory, None)
                    }
                }
                VulkanMemory::SubAlloc(sub_alloc) => {
                    let allocator = self.allocators[sub_alloc.memory_type_index.widen()]
                        .as_ref()
                        .unwrap();
                    allocator.tlsf.lock().free(sub_alloc.allocation)
                }
            }
        }
    }

    pub fn allocator_drop(&mut self) {
        for allocator in self.allocators.iter_mut().flatten() {
            // Clear out all memory blocks held by the TLSF allocators.
            let tlsf = allocator.tlsf.get_mut();
            for super_block in tlsf.super_blocks() {
                unsafe {
                    self.device_fn
                        .free_memory(self.device, super_block.user_data.memory, None)
                }
            }

            // Clear out all dedicated allocations.
            let dedicated = allocator.dedicated.get_mut();
            for memory in dedicated.iter() {
                unsafe { self.device_fn.free_memory(self.device, *memory, None) }
            }
        }
    }
}
