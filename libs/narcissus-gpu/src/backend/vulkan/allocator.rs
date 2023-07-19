use std::{
    collections::HashSet,
    sync::atomic::{AtomicU32, Ordering},
};

use narcissus_core::{default, BitIter, Mutex, Widen};

use vulkan_sys as vk;

use crate::{
    backend::vulkan::VULKAN_CONSTANTS,
    tlsf::{self, Tlsf},
    vk_check, MemoryLocation,
};

use super::{VulkanDevice, VulkanFrame};

#[derive(Default)]
pub struct VulkanAllocator {
    tlsf: [Mutex<Tlsf<VulkanAllocationInfo>>; vk::MAX_MEMORY_TYPES as usize],
    dedicated: Mutex<HashSet<vk::DeviceMemory>>,

    allocation_count: AtomicU32,
}

#[derive(Clone, Copy)]
pub struct VulkanAllocationInfo {
    memory: vk::DeviceMemory,
    mapped_ptr: *mut u8,
}

#[derive(Clone)]
pub struct VulkanMemoryDedicated {
    memory: vk::DeviceMemory,
    mapped_ptr: *mut u8,
    size: u64,
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
    pub fn allocate_memory(
        &self,
        memory_location: MemoryLocation,
        host_mapped: bool,
        memory_requirements: &vk::MemoryRequirements,
        memory_dedicated_requirements: &vk::MemoryDedicatedRequirements,
        memory_dedicated_allocate_info: &vk::MemoryDedicatedAllocateInfo,
    ) -> VulkanMemory {
        let required_memory_property_flags = if host_mapped {
            vk::MemoryPropertyFlags::HOST_VISIBLE
        } else {
            vk::MemoryPropertyFlags::default()
        };

        let mut preferred_memory_property_flags = match memory_location {
            MemoryLocation::Host => vk::MemoryPropertyFlags::HOST_VISIBLE,
            MemoryLocation::Device => vk::MemoryPropertyFlags::DEVICE_LOCAL,
        };

        let size = memory_requirements.size;
        let align = memory_requirements.alignment;

        fn allocate(
            device: &VulkanDevice,
            host_mapped: bool,
            allocation_size: u64,
            memory_type_index: u32,
            memory_dedicated_allocate_info: Option<&vk::MemoryDedicatedAllocateInfo>,
        ) -> Option<(vk::DeviceMemory, *mut u8)> {
            if device.allocator.allocation_count.load(Ordering::Relaxed)
                >= device
                    .physical_device_properties
                    .properties
                    .limits
                    .max_memory_allocation_count
            {
                return None;
            }

            let mut allocate_info = vk::MemoryAllocateInfo {
                allocation_size,
                memory_type_index,
                ..default()
            };

            if let Some(memory_dedicated_allocate_info) = memory_dedicated_allocate_info {
                allocate_info._next = memory_dedicated_allocate_info
                    as *const vk::MemoryDedicatedAllocateInfo
                    as *const _;
            }

            let mut memory = vk::DeviceMemory::null();
            let memory = match unsafe {
                device
                    .device_fn
                    .allocate_memory(device.device, &allocate_info, None, &mut memory)
            } {
                vk::Result::Success => memory,
                vk::Result::ErrorOutOfDeviceMemory | vk::Result::ErrorOutOfHostMemory => {
                    return None
                }
                _ => panic!(),
            };

            device
                .allocator
                .allocation_count
                .fetch_add(1, Ordering::AcqRel);

            let mapped_ptr = if host_mapped {
                let mut data = std::ptr::null_mut();
                vk_check!(device.device_fn.map_memory(
                    device.device,
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

            Some((memory, mapped_ptr))
        }

        // Outer loop here so that if we fail the first time around, we can clear the
        // preferred memory property flags and try again.
        loop {
            for memory_type_index in
                BitIter::new(std::iter::once(memory_requirements.memory_type_bits))
            {
                let memory_type =
                    &self.physical_device_memory_properties.memory_types[memory_type_index];

                if !memory_type
                    .property_flags
                    .contains(required_memory_property_flags)
                {
                    continue;
                }

                if !memory_type
                    .property_flags
                    .contains(preferred_memory_property_flags)
                {
                    continue;
                }

                // Does the driver want a dedicated allocation?
                if memory_dedicated_requirements.requires_dedicated_allocation == vk::Bool32::True
                    || memory_dedicated_requirements.prefers_dedicated_allocation
                        == vk::Bool32::True
                {
                    if let Some((memory, mapped_ptr)) = allocate(
                        self,
                        host_mapped,
                        size,
                        memory_type_index as u32,
                        Some(memory_dedicated_allocate_info),
                    ) {
                        self.allocator.dedicated.lock().insert(memory);

                        return VulkanMemory::Dedicated(VulkanMemoryDedicated {
                            memory,
                            mapped_ptr,
                            size,
                        });
                    }
                }

                let block_size = VULKAN_CONSTANTS.tlsf_maximum_block_size;

                // If the allocation is smaller than the TLSF super-block size for this
                // allocation type, we should attempt sub-allocation.
                if size <= block_size {
                    let mut tlsf = self.allocator.tlsf[memory_type_index].lock();

                    if let Some(allocation) = tlsf.alloc(size, align) {
                        return VulkanMemory::SubAlloc(VulkanMemorySubAlloc {
                            allocation,
                            size,
                            memory_type_index: memory_type_index as u32,
                        });
                    } else {
                        // When allocating backing storage for TLSF super-blocks, ensure that all memory
                        // is mapped if the memory type supports host mapping. This ensures we never
                        // have to map a super-block later if an individual allocation desires it.
                        if let Some((memory, mapped_ptr)) = allocate(
                            self,
                            memory_type
                                .property_flags
                                .contains(vk::MemoryPropertyFlags::HOST_VISIBLE),
                            block_size,
                            memory_type_index as u32,
                            None,
                        ) {
                            tlsf.insert_super_block(
                                block_size,
                                VulkanAllocationInfo { memory, mapped_ptr },
                            );

                            // After inserting a new super-block we should always be able to service the
                            // allocation request since the outer condition checks `size` <= `block_size`.
                            let allocation = tlsf.alloc(size, align).unwrap();

                            return VulkanMemory::SubAlloc(VulkanMemorySubAlloc {
                                allocation,
                                size,
                                memory_type_index: memory_type_index as u32,
                            });
                        }
                    }
                }

                // If sub-allocation failed, and we were unable to allocate a new super-block,
                // OR
                // If the requested allocation size was too large for the TLSF allocator,
                //
                // Attempt a dedicated allocation for the exact requested size.
                if let Some((memory, mapped_ptr)) =
                    allocate(self, host_mapped, size, memory_type_index as u32, None)
                {
                    self.allocator.dedicated.lock().insert(memory);

                    return VulkanMemory::Dedicated(VulkanMemoryDedicated {
                        memory,
                        mapped_ptr,
                        size,
                    });
                }
            }

            // If we have any preferred flags, then try clearing those and trying again.
            // If there's no preferred flags left, then we couldn't allocate any memory.
            if preferred_memory_property_flags == default() {
                panic!("allocation failure")
            } else {
                preferred_memory_property_flags = default()
            }
        }
    }

    pub fn allocator_begin_frame(&self, frame: &mut VulkanFrame) {
        for allocation in frame.destroyed_allocations.get_mut().drain(..) {
            match allocation {
                VulkanMemory::Dedicated(dedicated) => {
                    self.allocator.dedicated.lock().remove(&dedicated.memory);
                    unsafe {
                        self.device_fn
                            .free_memory(self.device, dedicated.memory, None)
                    }
                }
                VulkanMemory::SubAlloc(sub_alloc) => {
                    let mut allocator =
                        self.allocator.tlsf[sub_alloc.memory_type_index.widen()].lock();
                    allocator.free(sub_alloc.allocation)
                }
            }
        }
    }

    pub fn allocator_drop(&mut self) {
        for tlsf in self.allocator.tlsf.iter_mut() {
            // Clear out all memory blocks held by the TLSF allocators.
            let tlsf = tlsf.get_mut();
            for super_block in tlsf.super_blocks() {
                unsafe {
                    self.device_fn
                        .free_memory(self.device, super_block.user_data.memory, None)
                }
            }
        }

        // Clear out all dedicated allocations.
        let dedicated = self.allocator.dedicated.get_mut();
        for memory in dedicated.iter() {
            unsafe { self.device_fn.free_memory(self.device, *memory, None) }
        }
    }
}
