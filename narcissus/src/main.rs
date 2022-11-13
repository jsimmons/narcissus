use std::{path::Path, time::Instant};

use narcissus_app::{create_app, Event, Key, WindowDesc};
use narcissus_core::{cstr, default, obj, Image};
use narcissus_gpu::{
    create_vulkan_device, Bind, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType, Buffer,
    BufferDesc, BufferUsageFlags, ClearValue, CompareOp, CullingMode, Device, FrontFace,
    GraphicsPipelineDesc, GraphicsPipelineLayout, IndexType, LoadOp, MemoryLocation, PolygonMode,
    RenderingAttachment, RenderingDesc, Scissor, ShaderDesc, ShaderStageFlags, StoreOp,
    TextureDesc, TextureDimension, TextureFormat, TextureUsageFlags, ThreadToken, Topology,
    TypedBind, Viewport,
};
use narcissus_maths::{sin_cos_pi_f32, Deg, Mat4, Point3, Vec2, Vec3, Vec4};

/// Marker trait indicates it's safe to convert a given type directly to an array of bytes.
///
/// # Safety
///
/// Must not be applied to any types with padding
unsafe trait Blittable: Sized {}

#[allow(unused)]
struct Uniform {
    clip_from_model: Mat4,
}

unsafe impl Blittable for Uniform {}

#[allow(unused)]
struct Vertex {
    position: [f32; 4],
    normal: [f32; 4],
    texcoord: [f32; 4],
}

unsafe impl Blittable for Vertex {}
unsafe impl Blittable for u16 {}

fn load_obj<P: AsRef<Path>>(path: P) -> (Vec<Vertex>, Vec<u16>) {
    #[derive(Default)]
    struct ObjVisitor {
        positions: Vec<Vec3>,
        normals: Vec<Vec3>,
        texcoords: Vec<Vec2>,
        indices: Vec<[(i32, i32, i32); 3]>,
    }

    impl obj::Visitor for ObjVisitor {
        fn visit_position(&mut self, x: f32, y: f32, z: f32, _w: f32) {
            self.positions.push(Vec3::new(x, y, z))
        }

        fn visit_texcoord(&mut self, u: f32, v: f32, _w: f32) {
            self.texcoords.push(Vec2::new(u, v));
        }

        fn visit_normal(&mut self, x: f32, y: f32, z: f32) {
            self.normals.push(Vec3::new(x, y, z))
        }

        fn visit_face(&mut self, indices: &[(i32, i32, i32)]) {
            self.indices
                .push(indices.try_into().expect("not a triangle"));
        }

        fn visit_object(&mut self, _name: &str) {}
        fn visit_group(&mut self, _name: &str) {}
        fn visit_smooth_group(&mut self, _group: i32) {}
    }

    let start = std::time::Instant::now();
    let path = path.as_ref();
    let file = std::fs::File::open(path).expect("couldn't open file");
    let mut visitor = ObjVisitor::default();

    obj::Parser::new(file)
        .visit(&mut visitor)
        .expect("failed to parse obj file");

    let (vertices, indices): (Vec<_>, Vec<_>) = visitor
        .indices
        .iter()
        .flatten()
        .enumerate()
        .map(|(index, &(position_index, texcoord_index, normal_index))| {
            let position = visitor.positions[position_index as usize - 1];
            let normal = visitor.normals[normal_index as usize - 1];
            let texcoord = visitor.texcoords[texcoord_index as usize - 1];
            (
                Vertex {
                    position: Vec4::new(position.x, position.y, position.z, 0.0).into(),
                    normal: Vec4::new(normal.x, normal.y, normal.z, 0.0).into(),
                    texcoord: Vec4::new(texcoord.x, texcoord.y, 0.0, 0.0).into(),
                },
                index as u16,
            )
        })
        .unzip();

    println!(
        "parsing obj {path:?} took {:?}",
        std::time::Instant::now() - start
    );

    (vertices, indices)
}

fn load_img<P: AsRef<Path>>(path: P) -> Image {
    let start = std::time::Instant::now();
    let path = path.as_ref();
    let image = Image::from_buffer(std::fs::read(path).expect("failed to read file").as_slice())
        .expect("failed to load image");
    println!(
        "loading image {path:?} took {:?}",
        std::time::Instant::now() - start
    );
    image
}

fn create_buffer_with_data<T>(device: &dyn Device, usage: BufferUsageFlags, data: &[T]) -> Buffer
where
    T: Blittable,
{
    let len = data.len() * std::mem::size_of::<T>();
    let buffer = device.create_buffer(&BufferDesc {
        memory_location: MemoryLocation::PreferHost,
        usage,
        size: len,
    });
    // Safety: T: Blittable which implies it's freely convertable to a byte slice.
    unsafe {
        let dst = std::slice::from_raw_parts_mut(device.map_buffer(buffer), len);
        let src = std::slice::from_raw_parts(data.as_ptr() as *const u8, len);
        dst.copy_from_slice(src);
        device.unmap_buffer(buffer);
    }
    buffer
}

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
}

impl<'a> Drop for UniformBufferMap<'a> {
    fn drop(&mut self) {
        // Safety: Make sure we don't have the slice outlive the mapping.
        unsafe {
            self.device.unmap_buffer(self.buffer);
        }
    }
}

pub fn main() {
    let _blåhaj_image = load_img("narcissus/data/blåhaj.png");
    let (blåhaj_vertices, blåhaj_indices) = load_obj("narcissus/data/blåhaj.obj");

    let app = create_app();
    let main_window = app.create_window(&WindowDesc {
        title: "narcissus",
        width: 800,
        height: 600,
    });

    let device = create_vulkan_device(app.as_ref());
    let mut thread_token = ThreadToken::new();

    #[repr(align(4))]
    struct Spirv<const LEN: usize>([u8; LEN]);

    let vert_spv = Spirv(*include_bytes!("shaders/basic.vert.spv"));
    let frag_spv = Spirv(*include_bytes!("shaders/basic.frag.spv"));

    let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDesc {
        entries: &[BindGroupLayoutEntryDesc {
            slot: 0,
            stages: ShaderStageFlags::ALL,
            binding_type: BindingType::UniformBuffer,
            count: 1,
        }],
    });

    let storage_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDesc {
        entries: &[BindGroupLayoutEntryDesc {
            slot: 0,
            stages: ShaderStageFlags::ALL,
            binding_type: BindingType::StorageBuffer,
            count: 1,
        }],
    });

    let pipeline = device.create_graphics_pipeline(&GraphicsPipelineDesc {
        vertex_shader: ShaderDesc {
            entry: cstr!("main"),
            code: &vert_spv.0,
        },
        fragment_shader: ShaderDesc {
            entry: cstr!("main"),
            code: &frag_spv.0,
        },
        bind_group_layouts: &[uniform_bind_group_layout, storage_bind_group_layout],
        layout: GraphicsPipelineLayout {
            color_attachment_formats: &[TextureFormat::BGRA8_SRGB],
            depth_attachment_format: Some(TextureFormat::DEPTH_F32),
            stencil_attachment_format: None,
        },
        topology: Topology::Triangles,
        polygon_mode: PolygonMode::Fill,
        culling_mode: CullingMode::Back,
        front_face: FrontFace::Clockwise,
        depth_bias: None,
        depth_compare_op: CompareOp::GreaterOrEqual,
        depth_test_enable: true,
        depth_write_enable: true,
        stencil_test_enable: false,
        stencil_back: default(),
        stencil_front: default(),
    });

    let blåhaj_vertex_buffer = create_buffer_with_data(
        device.as_ref(),
        BufferUsageFlags::STORAGE,
        blåhaj_vertices.as_slice(),
    );

    let blåhaj_index_buffer = create_buffer_with_data(
        device.as_ref(),
        BufferUsageFlags::INDEX,
        blåhaj_indices.as_slice(),
    );

    let mut uniforms = UniformBufferMap::new(device.as_ref(), std::mem::size_of::<Uniform>());

    let mut depth_width = 0;
    let mut depth_height = 0;
    let mut depth_image = default();

    let start_time = Instant::now();
    'main: loop {
        let frame = device.begin_frame();

        while let Some(event) = app.poll_event() {
            use Event::*;
            match event {
                KeyPress {
                    window: _,
                    key,
                    pressed: _,
                    modifiers: _,
                } => {
                    if key == Key::Escape {
                        break 'main;
                    }
                }
                Quit => {
                    break 'main;
                }
                Close { window } => {
                    assert_eq!(window, main_window);
                    device.destroy_window(window);
                    break 'main;
                }
                _ => {}
            }
        }

        let (width, height, swapchain_image) =
            device.acquire_swapchain(&frame, main_window, TextureFormat::BGRA8_SRGB);

        let frame_start = Instant::now() - start_time;
        let frame_start = frame_start.as_secs_f32() * 0.5;

        let (s, c) = sin_cos_pi_f32(frame_start);
        let camera_from_model =
            Mat4::look_at(Point3::new(s * 5.0, 1.0, c * 5.0), Point3::ZERO, -Vec3::Y);
        let clip_from_camera =
            Mat4::perspective_rev_inf_zo(Deg::new(90.0).into(), width as f32 / height as f32, 0.01);
        let clip_from_model = clip_from_camera * camera_from_model;

        uniforms.write(Uniform { clip_from_model });

        if width != depth_width || height != depth_height {
            device.destroy_texture(&frame, depth_image);
            depth_image = device.create_texture(&TextureDesc {
                memory_location: MemoryLocation::PreferDevice,
                usage: TextureUsageFlags::DEPTH_STENCIL,
                dimension: TextureDimension::Type2d,
                format: TextureFormat::DEPTH_F32,
                width,
                height,
                depth: 1,
                layer_count: 1,
                mip_levels: 1,
            });
            depth_width = width;
            depth_height = height;
        }

        let mut cmd_buffer = device.create_cmd_buffer(&frame, &mut thread_token);

        device.cmd_set_pipeline(&mut cmd_buffer, pipeline);

        device.cmd_set_bind_group(
            &frame,
            &mut thread_token,
            &mut cmd_buffer,
            uniform_bind_group_layout,
            0,
            &[Bind {
                binding: 0,
                array_element: 0,
                typed: TypedBind::UniformBuffer(&[uniforms.buffer()]),
            }],
        );

        device.cmd_set_bind_group(
            &frame,
            &mut thread_token,
            &mut cmd_buffer,
            storage_bind_group_layout,
            1,
            &[Bind {
                binding: 0,
                array_element: 0,
                typed: TypedBind::StorageBuffer(&[blåhaj_vertex_buffer]),
            }],
        );

        device.cmd_set_index_buffer(&mut cmd_buffer, blåhaj_index_buffer, 0, IndexType::U16);

        device.cmd_begin_rendering(
            &mut cmd_buffer,
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
                depth_attachment: Some(RenderingAttachment {
                    texture: depth_image,
                    load_op: LoadOp::Clear(ClearValue::DepthStencil {
                        depth: 0.0,
                        stencil: 0,
                    }),
                    store_op: StoreOp::DontCare,
                }),
                stencil_attachment: None,
            },
        );

        device.cmd_set_scissors(
            &mut cmd_buffer,
            &[Scissor {
                x: 0,
                y: 0,
                width,
                height,
            }],
        );

        device.cmd_set_viewports(
            &mut cmd_buffer,
            &[Viewport {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }],
        );

        device.cmd_draw_indexed(&mut cmd_buffer, blåhaj_indices.len() as u32, 1, 0, 0, 0);

        device.cmd_end_rendering(&mut cmd_buffer);

        device.submit(&frame, cmd_buffer);

        device.end_frame(frame);
    }
}
