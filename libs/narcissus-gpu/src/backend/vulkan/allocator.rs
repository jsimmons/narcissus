use std::{
    collections::HashSet,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

use narcissus_core::{default, BitIter, Mutex, Widen};

use vulkan_sys as vk;

use crate::{
    tlsf::{self, Tlsf},
    vk_check, MemoryLocation,
};

use super::{VulkanDevice, VulkanFrame, VULKAN_CONSTANTS};

#[derive(Default, Debug)]
pub struct VulkanMemoryHeap {
    /// The calculated Tlsf super-block size for this memory heap.
    ///
    /// Smaller heaps will require a smaller super block size to prevent excess
    /// memory waste. Calculate a suitable super block size using
    /// `VULKAN_CONSTANTS.tlsf_default_super_block_size` and
    /// `VULKAN_CONSTANTS.tlsf_small_super_block_divisor`.
    tlsf_super_block_size: u64,

    /// Total size in bytes we have allocated against this memory heap.
    total_allocated_bytes: AtomicU64,
}

#[derive(Default)]
pub struct VulkanMemoryType {
    tlsf: Mutex<Tlsf<VulkanSuperBlockInfo>>,
}

#[derive(Default)]
pub struct VulkanAllocator {
    memory_heaps: [VulkanMemoryHeap; vk::MAX_MEMORY_HEAPS as usize],
    memory_types: [VulkanMemoryType; vk::MAX_MEMORY_TYPES as usize],
    dedicated: Mutex<HashSet<vk::DeviceMemory>>,
    allocation_count: AtomicU32,
}

impl VulkanAllocator {
    pub fn new(memory_properties: &vk::PhysicalDeviceMemoryProperties) -> Self {
        let memory_heaps = std::array::from_fn(|memory_heap_index| {
            let memory_heap_properties = &memory_properties.memory_heaps[memory_heap_index];
            let tlsf_super_block_size = if memory_heap_properties.size
                >= VULKAN_CONSTANTS.tlsf_small_super_block_divisor
                    * VULKAN_CONSTANTS.tlsf_default_super_block_size
            {
                VULKAN_CONSTANTS.tlsf_default_super_block_size
            } else {
                memory_heap_properties.size / VULKAN_CONSTANTS.tlsf_small_super_block_divisor
            };
            VulkanMemoryHeap {
                tlsf_super_block_size,
                total_allocated_bytes: default(),
            }
        });

        Self {
            memory_heaps,
            ..default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct VulkanSuperBlockInfo {
    memory: vk::DeviceMemory,
    mapped_ptr: *mut u8,
    memory_type_index: u32,
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
    allocation: tlsf::Allocation<VulkanSuperBlockInfo>,
    size: u64,
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
    fn free_memory(&self, memory: VulkanMemory) {
        match memory {
            VulkanMemory::Dedicated(dedicated) => {
                self.allocator.dedicated.lock().remove(&dedicated.memory);

                let memory_heap = &self.allocator.memory_heaps[self
                    .physical_device_memory_properties
                    .memory_types[dedicated.memory_type_index.widen()]
                .heap_index
                .widen()];

                memory_heap
                    .total_allocated_bytes
                    .fetch_sub(dedicated.size, Ordering::SeqCst);

                self.allocator
                    .allocation_count
                    .fetch_sub(1, Ordering::SeqCst);

                unsafe {
                    self.device_fn
                        .free_memory(self.device, dedicated.memory, None)
                }
            }
            VulkanMemory::SubAlloc(sub_alloc) => {
                let mut allocator = self.allocator.memory_types
                    [sub_alloc.allocation.user_data().memory_type_index.widen()]
                .tlsf
                .lock();
                allocator.free(sub_alloc.allocation)
            }
        }
    }

    fn try_allocate_memory(
        &self,
        host_mapped: bool,
        allocation_size: u64,
        memory_type_index: u32,
        memory_dedicated_allocate_info: Option<&vk::MemoryDedicatedAllocateInfo>,
    ) -> Option<(vk::DeviceMemory, *mut u8)> {
        // Can't allocate if we would blow the global allocation limit.
        if self.allocator.allocation_count.load(Ordering::Relaxed)
            >= self
                .physical_device_properties
                .properties
                .limits
                .max_memory_allocation_count
        {
            return None;
        }

        let heap_index = self.physical_device_memory_properties.memory_types
            [memory_type_index.widen()]
        .heap_index;

        let memory_heap_properties =
            &self.physical_device_memory_properties.memory_heaps[heap_index.widen()];
        let memory_heap = &self.allocator.memory_heaps[heap_index.widen()];

        // Can't allocate if we would blow this heap's size.
        let current_allocated_bytes = memory_heap.total_allocated_bytes.load(Ordering::Relaxed);
        if current_allocated_bytes + allocation_size > memory_heap_properties.size {
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
            self.device_fn
                .allocate_memory(self.device, &allocate_info, None, &mut memory)
        } {
            vk::Result::Success => memory,
            vk::Result::ErrorOutOfDeviceMemory | vk::Result::ErrorOutOfHostMemory => return None,
            _ => panic!(),
        };

        // Update allocation statistics.
        self.allocator
            .allocation_count
            .fetch_add(1, Ordering::AcqRel);

        memory_heap
            .total_allocated_bytes
            .fetch_add(allocation_size, Ordering::SeqCst);

        let mapped_ptr = if host_mapped {
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

        Some((memory, mapped_ptr))
    }

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

        // Outer loop here so that if we fail the first time around, we can clear the
        // preferred memory property flags and try again.
        loop {
            for memory_type_index in
                BitIter::new(std::iter::once(memory_requirements.memory_type_bits))
            {
                let memory_type_properties =
                    &self.physical_device_memory_properties.memory_types[memory_type_index];
                let memory_heap_index = memory_type_properties.heap_index.widen();

                let memory_type_property_flags = memory_type_properties.property_flags;
                if !memory_type_property_flags
                    .contains(required_memory_property_flags | preferred_memory_property_flags)
                {
                    continue;
                }

                let memory_type = &self.allocator.memory_types[memory_type_index];
                let memory_heap = &self.allocator.memory_heaps[memory_heap_index];

                // Does the driver want a dedicated allocation?
                if memory_dedicated_requirements.requires_dedicated_allocation == vk::Bool32::True
                    || memory_dedicated_requirements.prefers_dedicated_allocation
                        == vk::Bool32::True
                {
                    if let Some((memory, mapped_ptr)) = self.try_allocate_memory(
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
                            memory_type_index: memory_type_index as u32,
                        });
                    }
                }

                // If the allocation is smaller than the Tlsf super-block size for this
                // allocation type, we should attempt sub-allocation.
                if size <= memory_heap.tlsf_super_block_size {
                    let mut tlsf = memory_type.tlsf.lock();

                    if let Some(allocation) = tlsf.alloc(size, align) {
                        return VulkanMemory::SubAlloc(VulkanMemorySubAlloc { allocation, size });
                    } else {
                        // When allocating backing storage for Tlsf super-blocks, ensure that all memory
                        // is mapped if the memory type supports host mapping. This ensures we never
                        // have to map a super-block later if an individual allocation desires it.
                        if let Some((memory, mapped_ptr)) = self.try_allocate_memory(
                            memory_type_property_flags
                                .contains(vk::MemoryPropertyFlags::HOST_VISIBLE),
                            memory_heap.tlsf_super_block_size,
                            memory_type_index as u32,
                            None,
                        ) {
                            tlsf.insert_super_block(
                                memory_heap.tlsf_super_block_size,
                                VulkanSuperBlockInfo {
                                    memory,
                                    mapped_ptr,
                                    memory_type_index: memory_type_index as u32,
                                },
                            );

                            // After inserting a new super-block we should always be able to service the
                            // allocation request since the outer condition checks `size` <= `block_size`.
                            let allocation = tlsf.alloc(size, align).unwrap();

                            return VulkanMemory::SubAlloc(VulkanMemorySubAlloc {
                                allocation,
                                size,
                            });
                        }
                    }
                }

                // If sub-allocation failed, and we were unable to allocate a new super-block,
                // OR
                // If the requested allocation size was too large for the Tlsf allocator,
                //
                // Attempt a dedicated allocation for the exact requested size.
                if let Some((memory, mapped_ptr)) =
                    self.try_allocate_memory(host_mapped, size, memory_type_index as u32, None)
                {
                    self.allocator.dedicated.lock().insert(memory);

                    return VulkanMemory::Dedicated(VulkanMemoryDedicated {
                        memory,
                        mapped_ptr,
                        size,
                        memory_type_index: memory_type_index as u32,
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
            self.free_memory(allocation);
        }
    }

    pub fn allocator_drop(&mut self) {
        println!(
            "{:?}",
            &self.allocator.memory_heaps[..self
                .physical_device_memory_properties
                .memory_heap_count
                .widen()]
        );

        println!(
            "count: {}",
            self.allocator.allocation_count.load(Ordering::Relaxed)
        );

        for memory_type in self.allocator.memory_types.iter_mut() {
            // Clear out all memory blocks held by the Tlsf allocators.
            let tlsf = memory_type.tlsf.get_mut();
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
