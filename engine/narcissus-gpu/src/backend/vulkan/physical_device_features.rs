use vulkan_sys::{self as vk};

#[derive(Default)]
pub struct VulkanPhysicalDeviceFeatures {
    features: vk::PhysicalDeviceFeatures2,
    features_11: vk::PhysicalDeviceVulkan11Features,
    features_12: vk::PhysicalDeviceVulkan12Features,
    features_13: vk::PhysicalDeviceVulkan13Features,
    features_swapchain_maintenance1: vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT,
}

impl VulkanPhysicalDeviceFeatures {
    pub fn compute_full_subgroups(&self) -> bool {
        self.features_13.compute_full_subgroups == vk::Bool32::True
    }

    pub fn descriptor_binding_partially_bound(&self) -> bool {
        self.features_12.descriptor_binding_partially_bound == vk::Bool32::True
    }

    pub fn descriptor_indexing(&self) -> bool {
        self.features_12.descriptor_indexing == vk::Bool32::True
    }

    pub fn draw_indirect_count(&self) -> bool {
        self.features_12.draw_indirect_count == vk::Bool32::True
    }

    pub fn dynamic_rendering(&self) -> bool {
        self.features_13.dynamic_rendering == vk::Bool32::True
    }

    pub fn subgroup_size_control(&self) -> bool {
        self.features_13.subgroup_size_control == vk::Bool32::True
    }

    pub fn maintenance4(&self) -> bool {
        self.features_13.maintenance4 == vk::Bool32::True
    }

    pub fn timeline_semaphore(&self) -> bool {
        self.features_12.timeline_semaphore == vk::Bool32::True
    }

    pub fn uniform_buffer_standard_layout(&self) -> bool {
        self.features_12.uniform_buffer_standard_layout == vk::Bool32::True
    }

    pub fn set_buffer_device_address(&mut self, arg: bool) {
        self.features_12.buffer_device_address = vk::Bool32::from(arg)
    }

    pub fn set_compute_full_subgroups(&mut self, arg: bool) {
        self.features_13.compute_full_subgroups = vk::Bool32::from(arg)
    }

    pub fn set_descriptor_binding_partially_bound(&mut self, arg: bool) {
        self.features_12.descriptor_binding_partially_bound = vk::Bool32::from(arg)
    }

    pub fn set_descriptor_indexing(&mut self, arg: bool) {
        self.features_12.descriptor_indexing = vk::Bool32::from(arg)
    }

    pub fn set_draw_indirect_count(&mut self, arg: bool) {
        self.features_12.draw_indirect_count = vk::Bool32::from(arg)
    }

    pub fn set_dynamic_rendering(&mut self, arg: bool) {
        self.features_13.dynamic_rendering = vk::Bool32::from(arg)
    }

    pub fn set_maintenance4(&mut self, arg: bool) {
        self.features_13.maintenance4 = vk::Bool32::from(arg)
    }

    pub fn set_shader_storage_image_read_without_format(&mut self, arg: bool) {
        self.features
            .features
            .shader_storage_image_read_without_format = vk::Bool32::from(arg)
    }

    pub fn set_shader_storage_image_write_without_format(&mut self, arg: bool) {
        self.features
            .features
            .shader_storage_image_write_without_format = vk::Bool32::from(arg)
    }

    pub fn set_subgroup_size_control(&mut self, arg: bool) {
        self.features_13.subgroup_size_control = vk::Bool32::from(arg)
    }

    pub fn set_swapchain_maintenance1(&mut self, arg: bool) {
        self.features_swapchain_maintenance1.swapchain_maintenance1 = vk::Bool32::from(arg)
    }

    pub fn set_synchronization2(&mut self, arg: bool) {
        self.features_13.synchronization2 = vk::Bool32::from(arg)
    }

    pub fn set_timeline_semaphore(&mut self, arg: bool) {
        self.features_12.timeline_semaphore = vk::Bool32::from(arg)
    }

    pub fn set_uniform_buffer_standard_layout(&mut self, arg: bool) {
        self.features_12.uniform_buffer_standard_layout = vk::Bool32::from(arg)
    }

    pub fn link(&mut self) -> &mut vk::PhysicalDeviceFeatures2 {
        self.features_13._next = &mut self.features_swapchain_maintenance1 as *mut _ as *mut _;
        self.features_12._next = &mut self.features_13 as *mut _ as *mut _;
        self.features_11._next = &mut self.features_12 as *mut _ as *mut _;
        self.features._next = &mut self.features_11 as *mut _ as *mut _;
        &mut self.features
    }
}
