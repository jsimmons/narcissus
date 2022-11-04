use std::time::Instant;

use narcissus_app::{create_app, Event, WindowDesc};
use narcissus_core::{cstr, obj, slice, Image};
use narcissus_gpu::{
    create_vulkan_device, Bind, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType, Buffer,
    BufferDesc, BufferUsageFlags, ClearValue, Device, GraphicsPipelineDesc, GraphicsPipelineLayout,
    LoadOp, MemoryLocation, RenderingAttachment, RenderingDesc, Scissor, ShaderDesc,
    ShaderStageFlags, StoreOp, TextureDesc, TextureDimension, TextureFormat, TextureUsageFlags,
    TextureViewDesc, ThreadToken, TypedBind, Viewport,
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

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDesc {
        entries: &[BindGroupLayoutEntryDesc {
            slot: 0,
            stages: ShaderStageFlags::ALL,
            binding_type: BindingType::UniformBuffer,
            count: 1,
        }],
    });

    let pipeline = device.create_graphics_pipeline(&GraphicsPipelineDesc {
        vertex_shader: ShaderDesc {
            entrypoint_name: cstr!("main"),
            code: &vert_shader_spv.0,
        },
        fragment_shader: ShaderDesc {
            entrypoint_name: cstr!("main"),
            code: &frag_shader_spv.0,
        },
        bind_group_layouts: &[bind_group_layout],
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

    struct UniformBufferMap<'a> {
        device: &'a dyn Device,
        buffer: Buffer,
        slice: &'a mut [u8],
    }

    impl<'a> UniformBufferMap<'a> {
        pub fn new(device: &'a dyn Device, len: usize) -> Self {
            let buffer = device.create_buffer(&BufferDesc {
                memory_location: MemoryLocation::PreferHost,
                usage: BufferUsageFlags::UNIFORM,
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

        pub fn write_f32(&mut self, value: f32) {
            self.slice.copy_from_slice(&value.to_le_bytes());
        }
    }

    impl<'a> Drop for UniformBufferMap<'a> {
        fn drop(&mut self) {
            // Safety: Make sure we don't have the slice outlive the mapping.
            unsafe {
                self.device.unmap_buffer(self.buffer);
            }
        }
    }

    let mut uniforms = UniformBufferMap::new(device.as_ref(), 4);

    let start_time = Instant::now();
    'main: loop {
        let frame_token = device.begin_frame();

        let frame_start = Instant::now() - start_time;
        let frame_start = frame_start.as_secs_f32();

        uniforms.write_f32(frame_start);

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

        let (width, height, swapchain_image) =
            device.acquire_swapchain(&frame_token, window, TextureFormat::BGRA8_SRGB);

        let mut command_buffer_token =
            device.create_command_buffer(&frame_token, &mut thread_token);

        device.cmd_begin_rendering(
            &frame_token,
            &mut thread_token,
            &mut command_buffer_token,
            &RenderingDesc {
                x: 0,
                y: 0,
                width,
                height,
                color_attachments: &[RenderingAttachment {
                    texture: swapchain_image,
                    load_op: LoadOp::Clear(ClearValue::ColorF32([
                        0.392157, 0.584314, 0.929412, 1.0,
                    ])),
                    store_op: StoreOp::Store,
                }],
                depth_attachment: None,
                stencil_attachment: None,
            },
        );

        device.cmd_set_pipeline(&mut command_buffer_token, pipeline);
        device.cmd_set_bind_group(
            &frame_token,
            &mut thread_token,
            &mut command_buffer_token,
            pipeline,
            bind_group_layout,
            0,
            &[Bind {
                binding: 0,
                array_element: 0,
                typed: TypedBind::Buffer(&[uniforms.buffer()]),
            }],
        );

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

        device.submit(&frame_token, &mut thread_token, command_buffer_token);

        device.end_frame(frame_token);
    }
}
