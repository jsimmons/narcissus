use vulkan_sys::{self as vk};

#[derive(Default)]
pub struct VulkanPhysicalDeviceProperties {
    properties: vk::PhysicalDeviceProperties2,
    properties_11: vk::PhysicalDeviceVulkan11Properties,
    properties_12: vk::PhysicalDeviceVulkan12Properties,
    properties_13: vk::PhysicalDeviceVulkan13Properties,
}

impl VulkanPhysicalDeviceProperties {
    pub fn api_version(&self) -> u32 {
        self.properties.properties.api_version
    }

    pub fn required_subgroup_size_stages(&self) -> vk::ShaderStageFlags {
        self.properties_13.required_subgroup_size_stages
    }

    pub fn min_subgroup_size(&self) -> u32 {
        self.properties_13.min_subgroup_size
    }

    pub fn max_subgroup_size(&self) -> u32 {
        self.properties_13.max_subgroup_size
    }

    pub fn limits(&self) -> &vk::PhysicalDeviceLimits {
        &self.properties.properties.limits
    }

    pub fn link(&mut self) -> &mut vk::PhysicalDeviceProperties2 {
        self.properties_12._next = &mut self.properties_13 as *mut _ as *mut _;
        self.properties_11._next = &mut self.properties_12 as *mut _ as *mut _;
        self.properties._next = &mut self.properties_11 as *mut _ as *mut _;
        &mut self.properties
    }
}
