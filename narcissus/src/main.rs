use narcissus_app::{create_app, Event, Window, WindowDesc};
use narcissus_core::cstr;
use narcissus_gpu::{
    create_vulkan_device, ClearValue, Device, FrameToken, GraphicsPipelineDesc,
    GraphicsPipelineLayout, LoadOp, MemoryLocation, Pipeline, RenderingAttachment, RenderingDesc,
    Scissor, ShaderDesc, StoreOp, TextureDesc, TextureDimension, TextureFormat, TextureUsageFlags,
    TextureViewDesc, ThreadToken, Viewport,
};

fn render_window(
    device: &dyn Device,
    frame_token: &FrameToken,
    thread_token: &mut ThreadToken,
    pipeline: Pipeline,
    window: Window,
) {
    let (width, height, swapchain_image) =
        device.acquire_swapchain(&frame_token, window, TextureFormat::BGRA8_SRGB);

    let mut command_buffer_token = device.request_command_buffer(&frame_token, thread_token);
    device.cmd_begin_rendering(
        &mut command_buffer_token,
        &RenderingDesc {
            x: 0,
            y: 0,
            width,
            height,
            color_attachments: &[RenderingAttachment {
                texture: swapchain_image,
                load_op: LoadOp::Clear(ClearValue::ColorF32([0.392157, 0.584314, 0.929412, 1.0])),
                store_op: StoreOp::Store,
            }],
            depth_attachment: None,
            stencil_attachment: None,
        },
    );
    device.cmd_bind_pipeline(&mut command_buffer_token, pipeline);
    device.cmd_set_scissors(
        &mut command_buffer_token,
        &[Scissor {
            x: 0,
            y: 0,
            width,
            height,
        }],
    );
    device.cmd_set_viewports(
        &mut command_buffer_token,
        &[Viewport {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }],
    );
    device.cmd_draw(&mut command_buffer_token, 3, 1, 0, 0);
    device.cmd_end_rendering(&mut command_buffer_token);

    device.submit(command_buffer_token);
}

pub fn main() {
    let app = create_app();

    let device = create_vulkan_device(app.as_ref());
    let mut thread_token = ThreadToken::new();

    #[repr(align(4))]
    struct Spirv<const LEN: usize>([u8; LEN]);

    let vert_shader_spv = Spirv(*include_bytes!("shaders/triangle.vert.spv"));
    let frag_shader_spv = Spirv(*include_bytes!("shaders/triangle.frag.spv"));

    let pipeline = device.create_graphics_pipeline(&GraphicsPipelineDesc {
        vertex_shader: ShaderDesc {
            entrypoint_name: cstr!("main"),
            code: &vert_shader_spv.0,
        },
        fragment_shader: ShaderDesc {
            entrypoint_name: cstr!("main"),
            code: &frag_shader_spv.0,
        },
        layout: GraphicsPipelineLayout {
            color_attachment_formats: &[TextureFormat::BGRA8_SRGB],
            depth_attachment_format: None,
            stencil_attachment_format: None,
        },
    });

    let mut windows = (0..4)
        .map(|i| {
            let title = format!("Narcissus {}", i);
            let title = title.as_str();
            app.create_window(&WindowDesc {
                title,
                width: 800,
                height: 600,
            })
        })
        .collect::<Vec<_>>();

    let texture = device.create_texture(&TextureDesc {
        memory_location: MemoryLocation::PreferDevice,
        usage: TextureUsageFlags::SAMPLED,
        dimension: TextureDimension::Type2d,
        format: TextureFormat::BGRA8_SRGB,
        width: 800,
        height: 600,
        depth: 1,
        layers: 1,
        mip_levels: 1,
    });

    let texture2 = device.create_texture_view(&TextureViewDesc {
        texture,
        dimension: TextureDimension::Type2d,
        format: TextureFormat::BGRA8_SRGB,
        base_mip: 0,
        mip_count: 1,
        base_layer: 0,
        layer_count: 1,
    });

    let frame_token = device.begin_frame();
    device.destroy_texture(&frame_token, texture);
    device.destroy_texture(&frame_token, texture2);
    device.end_frame(frame_token);

    let mut should_quit = false;

    while !should_quit {
        let frame_token = device.begin_frame();

        while let Some(event) = app.poll_event() {
            use Event::*;
            match event {
                Quit => {
                    should_quit = true;
                    break;
                }
                WindowClose(window) => {
                    if let Some(index) = windows.iter().position(|&w| window == w) {
                        device.destroy_window(windows.swap_remove(index));
                    }
                }
                _ => {}
            }
        }

        for &window in windows.iter() {
            render_window(
                device.as_ref(),
                &frame_token,
                &mut thread_token,
                pipeline,
                window,
            );
        }

        device.end_frame(frame_token);
    }
}
