//! An alternative vulkan syncronization approach based on
//! <https://github.com/Tobski/simple_vulkan_synchronization>

use narcissus_core::default;
use vulkan_sys as vk;

use crate::{Access, GlobalBarrier, ImageBarrier, ImageLayout};

pub struct VulkanAccessInfo {
    stages: vk::PipelineStageFlags2,
    access: vk::AccessFlags2,
    layout: vk::ImageLayout,
}

#[must_use]
pub fn vulkan_access_info(access: Access) -> VulkanAccessInfo {
    match access {
        Access::None => VulkanAccessInfo {
            stages: default(),
            access: default(),
            layout: vk::ImageLayout::Undefined,
        },

        Access::IndirectBuffer => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::DRAW_INDIRECT,
            access: vk::AccessFlags2::INDIRECT_COMMAND_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::IndexBuffer => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_INPUT,
            access: vk::AccessFlags2::INDEX_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::VertexBuffer => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_INPUT,
            access: vk::AccessFlags2::VERTEX_ATTRIBUTE_READ,
            layout: vk::ImageLayout::Undefined,
        },

        Access::VertexShaderUniformBufferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_SHADER,
            access: vk::AccessFlags2::UNIFORM_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::VertexShaderSampledImageRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_SHADER,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::ReadOnlyOptimal,
        },
        Access::VertexShaderOtherRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_SHADER,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::General,
        },

        Access::FragmentShaderUniformBufferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            access: vk::AccessFlags2::UNIFORM_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::FragmentShaderSampledImageRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::ReadOnlyOptimal,
        },
        Access::FragmentShaderOtherRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::General,
        },

        Access::ColorAttachmentRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            access: vk::AccessFlags2::COLOR_ATTACHMENT_READ,
            layout: vk::ImageLayout::AttachmentOptimal,
        },
        Access::DepthStencilAttachmentRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
            layout: vk::ImageLayout::AttachmentOptimal,
        },

        Access::ComputeOtherRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COMPUTE_SHADER,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::General,
        },

        Access::ShaderUniformBufferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::UNIFORM_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::ShaderUniformBufferOrVertexBufferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::UNIFORM_READ | vk::AccessFlags2::VERTEX_ATTRIBUTE_READ,
            layout: vk::ImageLayout::Undefined,
        },
        Access::ShaderSampledImageRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::ReadOnlyOptimal,
        },
        Access::ShaderOtherRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::SHADER_READ,
            layout: vk::ImageLayout::General,
        },

        Access::TransferRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::TRANSFER,
            access: vk::AccessFlags2::TRANSFER_READ,
            layout: vk::ImageLayout::TransferSrcOptimal,
        },
        Access::HostRead => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::HOST,
            access: vk::AccessFlags2::HOST_READ,
            layout: vk::ImageLayout::General,
        },

        Access::PresentRead => VulkanAccessInfo {
            stages: default(),
            access: default(),
            layout: vk::ImageLayout::PresentSrcKhr,
        },

        Access::VertexShaderWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::VERTEX_SHADER,
            access: vk::AccessFlags2::SHADER_WRITE,
            layout: vk::ImageLayout::General,
        },
        Access::FragmentShaderWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            access: vk::AccessFlags2::SHADER_WRITE,
            layout: vk::ImageLayout::General,
        },
        Access::ColorAttachmentWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            layout: vk::ImageLayout::ColorAttachmentOptimal,
        },
        Access::DepthStencilAttachmentWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
            access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
            layout: vk::ImageLayout::DepthAttachmentOptimal,
        },

        Access::ComputeWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COMPUTE_SHADER,
            access: vk::AccessFlags2::SHADER_WRITE,
            layout: vk::ImageLayout::General,
        },

        Access::ShaderWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::SHADER_WRITE,
            layout: vk::ImageLayout::General,
        },
        Access::TransferWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::TRANSFER,
            access: vk::AccessFlags2::TRANSFER_WRITE,
            layout: vk::ImageLayout::TransferDstOptimal,
        },
        Access::HostPreInitializedWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::HOST,
            access: vk::AccessFlags2::HOST_WRITE,
            layout: vk::ImageLayout::Preinitialized,
        },
        Access::HostWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::HOST,
            access: vk::AccessFlags2::HOST_WRITE,
            layout: vk::ImageLayout::General,
        },
        Access::ColorAttachmentReadWrite => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            access: vk::AccessFlags2::COLOR_ATTACHMENT_READ
                | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            layout: vk::ImageLayout::AttachmentOptimal,
        },
        Access::General => VulkanAccessInfo {
            stages: vk::PipelineStageFlags2::ALL_COMMANDS,
            access: vk::AccessFlags2::COLOR_ATTACHMENT_READ
                | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            layout: vk::ImageLayout::General,
        },
    }
}

pub fn vulkan_memory_barrier(barrier: &GlobalBarrier) -> vk::MemoryBarrier2 {
    let mut src_stage_mask = default();
    let mut src_access_mask = default();
    let mut dst_stage_mask = default();
    let mut dst_access_mask = default();

    for &access in barrier.prev_access {
        debug_assert!(
            access.is_read() || barrier.prev_access.len() == 1,
            "write access types must be on their own"
        );

        let info = vulkan_access_info(access);
        src_stage_mask |= info.stages;

        // For writes, add availability operations.
        if access.is_write() {
            src_access_mask |= info.access;
        }
    }

    for &access in barrier.next_access {
        debug_assert!(
            access.is_read() || barrier.prev_access.len() == 1,
            "write access types must be on their own"
        );

        let info = vulkan_access_info(access);
        dst_stage_mask |= info.stages;

        // Add visibility operations if necessary.
        //
        // If the src access mask is zero, this is a write-after-read hazard (or for
        // some reason, a read-after-read hazard), so the dst access mask can be safely
        // zeroed as these don't need visibility.
        if src_access_mask != default() {
            dst_access_mask |= info.access;
        }
    }

    if src_stage_mask == default() {
        src_stage_mask = vk::PipelineStageFlags2::NONE;
    }

    if dst_stage_mask == default() {
        dst_stage_mask = vk::PipelineStageFlags2::NONE;
    }

    vk::MemoryBarrier2 {
        src_stage_mask,
        src_access_mask,
        dst_stage_mask,
        dst_access_mask,
        ..default()
    }
}

pub fn vulkan_image_memory_barrier(
    barrier: &ImageBarrier,
    image: vk::Image,
    subresource_range: vk::ImageSubresourceRange,
) -> vk::ImageMemoryBarrier2 {
    let mut src_stage_mask = default();
    let mut src_access_mask = default();
    let mut dst_stage_mask = default();
    let mut dst_access_mask = default();
    let mut old_layout = vk::ImageLayout::Undefined;
    let mut new_layout = vk::ImageLayout::Undefined;

    for &access in barrier.prev_access {
        debug_assert!(
            access.is_read() || barrier.prev_access.len() == 1,
            "write access types must be on their own"
        );

        let info = vulkan_access_info(access);
        src_stage_mask |= info.stages;

        // For writes, add availability operations.
        if access.is_write() {
            src_access_mask |= info.access;
        }

        old_layout = if barrier.discard_contents {
            vk::ImageLayout::Undefined
        } else {
            let layout = match barrier.prev_layout {
                ImageLayout::Optimal => info.layout,
                ImageLayout::General => {
                    if access == Access::PresentRead {
                        vk::ImageLayout::PresentSrcKhr
                    } else {
                        vk::ImageLayout::General
                    }
                }
            };

            debug_assert!(
                old_layout == vk::ImageLayout::Undefined || old_layout == layout,
                "mixed image layout"
            );

            layout
        };
    }

    for &access in barrier.next_access {
        debug_assert!(
            access.is_read() || barrier.prev_access.len() == 1,
            "write access types must be on their own"
        );

        let info = vulkan_access_info(access);
        dst_stage_mask |= info.stages;

        // Open question whether this is always required.
        // <https://github.com/Tobski/simple_vulkan_synchronization/issues/19#issuecomment-1460325972>
        dst_access_mask |= info.access;

        let layout = match barrier.next_layout {
            ImageLayout::Optimal => info.layout,
            ImageLayout::General => {
                if access == Access::PresentRead {
                    vk::ImageLayout::PresentSrcKhr
                } else {
                    vk::ImageLayout::General
                }
            }
        };

        debug_assert!(
            new_layout == vk::ImageLayout::Undefined || new_layout == layout,
            "mixed image layout"
        );

        new_layout = layout;
    }

    vk::ImageMemoryBarrier2 {
        src_stage_mask,
        src_access_mask,
        dst_stage_mask,
        dst_access_mask,
        old_layout,
        new_layout,
        image,
        subresource_range,
        ..default()
    }
}
