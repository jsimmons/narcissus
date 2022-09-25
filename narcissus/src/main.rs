use narcissus_app::{create_app, Event, Window, WindowDesc};
use narcissus_core::{cstr, obj, slice, Image};
use narcissus_gpu::{
    create_vulkan_device, ClearValue, Device, FrameToken, GraphicsPipelineDesc,
    GraphicsPipelineLayout, LoadOp, MemoryLocation, Pipeline, RenderingAttachment, RenderingDesc,
    Scissor, ShaderDesc, StoreOp, TextureDesc, TextureDimension, TextureFormat, TextureUsageFlags,
    TextureViewDesc, ThreadToken, Viewport,
};
use narcissus_maths::{Vec2, Vec3};

pub fn main() {
    let blåhaj_obj = std::fs::File::open("narcissus/data/blåhaj.obj").unwrap();

    #[derive(Default)]
    struct ObjVisitor {
        position: Vec<Vec3>,
        normals: Vec<Vec3>,
        texcoords: Vec<Vec2>,
        indices: Vec<[(i32, i32, i32); 3]>,
    }

    impl obj::Visitor for ObjVisitor {
        fn visit_position(&mut self, x: f32, y: f32, z: f32, _w: f32) {
            self.position.push(Vec3::new(x, y, z))
        }

        fn visit_texcoord(&mut self, u: f32, v: f32, _w: f32) {
            self.texcoords.push(Vec2::new(u, v));
        }

        fn visit_normal(&mut self, x: f32, y: f32, z: f32) {
            self.normals.push(Vec3::new(x, y, z))
        }

        fn visit_face(&mut self, indices: &[(i32, i32, i32)]) {
            for triangle in slice::array_windows::<_, 3>(indices) {
                self.indices.push(*triangle)
            }
        }

        fn visit_object(&mut self, _name: &str) {}
        fn visit_group(&mut self, _name: &str) {}
        fn visit_smooth_group(&mut self, _group: i32) {}
    }

    let mut obj_visitor = ObjVisitor::default();
    let mut obj_parser = obj::Parser::new(blåhaj_obj);

    let start = std::time::Instant::now();
    obj_parser.visit(&mut obj_visitor).unwrap();
    let end = std::time::Instant::now();

    println!(
        "loaded {} vertices, {} normals, {} texcoords, and {} indices",
        obj_visitor.position.len(),
        obj_visitor.normals.len(),
        obj_visitor.texcoords.len(),
        obj_visitor.indices.len(),
    );
    println!("took {:?}", end - start);

    let start = std::time::Instant::now();
    let blåhaj_image = std::fs::read("narcissus/data/blåhaj.png").unwrap();
    let blåhaj = Image::from_buffer(&blåhaj_image).unwrap();
    let end = std::time::Instant::now();

    println!(
        "loaded blåhaj width: {}, height: {}, components: {}",
        blåhaj.width(),
        blåhaj.height(),
        blåhaj.components()
    );
    println!("took {:?}", end - start);

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

    let window = app.create_window(&WindowDesc {
        title: "narcissus",
        width: 800,
        height: 600,
    });

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

    'main: loop {
        let frame_token = device.begin_frame();

        while let Some(event) = app.poll_event() {
            use Event::*;
            match event {
                Quit => {
                    break 'main;
                }
                WindowClose(w) => {
                    assert_eq!(window, w);
                    device.destroy_window(window);
                    break 'main;
                }
                _ => {}
            }
        }

        render_window(
            device.as_ref(),
            &frame_token,
            &mut thread_token,
            pipeline,
            window,
        );

        device.end_frame(frame_token);
    }
}

fn render_window(
    device: &dyn Device,
    frame_token: &FrameToken,
    thread_token: &mut ThreadToken,
    pipeline: Pipeline,
    window: Window,
) {
    let (width, height, swapchain_image) =
        device.acquire_swapchain(frame_token, window, TextureFormat::BGRA8_SRGB);
    let mut command_buffer_token = device.request_command_buffer(frame_token, thread_token);
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
