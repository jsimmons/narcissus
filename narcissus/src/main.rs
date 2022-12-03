use std::{path::Path, time::Instant};

use narcissus_app::{create_app, Event, Key, WindowDesc};
use narcissus_core::{cstr, default, include_bytes_align, obj, rand::Pcg64};
use narcissus_gpu::{
    create_device, Access, Bind, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingType,
    BlendMode, Buffer, BufferDesc, BufferImageCopy, BufferUsageFlags, ClearValue, CompareOp,
    CullingMode, Device, Extent2d, Extent3d, FrontFace, GraphicsPipelineDesc,
    GraphicsPipelineLayout, Image, ImageBarrier, ImageDesc, ImageDimension, ImageFormat,
    ImageLayout, ImageUsageFlags, IndexType, LoadOp, MemoryLocation, Offset2d, Offset3d,
    PolygonMode, RenderingAttachment, RenderingDesc, SamplerAddressMode, SamplerDesc,
    SamplerFilter, Scissor, ShaderDesc, ShaderStageFlags, StoreOp, ThreadToken, Topology,
    TypedBind, Viewport,
};
use narcissus_image as image;
use narcissus_maths::{
    sin_cos_pi_f32, vec2, vec3, vec4, Affine3, Deg, HalfTurn, Mat3, Mat4, Point3, Vec2, Vec3,
};

const MAX_SHARKS: usize = 262_144;
const NUM_SHARKS: usize = 50;

/// Marker trait indicates it's safe to convert a given type directly to an array of bytes.
///
/// # Safety
///
/// Must not be applied to any types with padding
unsafe trait Blittable: Sized {}

#[allow(unused)]
struct Uniforms {
    clip_from_model: Mat4,
}

unsafe impl Blittable for Uniforms {}

#[allow(unused)]
struct Vertex {
    position: [f32; 4],
    normal: [f32; 4],
    texcoord: [f32; 4],
}

unsafe impl Blittable for Vertex {}
unsafe impl Blittable for u8 {}
unsafe impl Blittable for u16 {}
unsafe impl Blittable for Affine3 {}

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
            self.positions.push(vec3(x, y, z))
        }

        fn visit_texcoord(&mut self, u: f32, v: f32, _w: f32) {
            self.texcoords.push(vec2(u, v));
        }

        fn visit_normal(&mut self, x: f32, y: f32, z: f32) {
            self.normals.push(vec3(x, y, z))
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
                    position: vec4(position.x, position.y, position.z, 0.0).into(),
                    normal: vec4(normal.x, normal.y, normal.z, 0.0).into(),
                    texcoord: vec4(texcoord.x, texcoord.y, 0.0, 0.0).into(),
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

fn load_image<P: AsRef<Path>>(path: P) -> image::Image {
    let start = std::time::Instant::now();
    let path = path.as_ref();
    let texture =
        image::Image::from_buffer(std::fs::read(path).expect("failed to read file").as_slice())
            .expect("failed to load image");
    println!(
        "loading image {path:?} took {:?}",
        std::time::Instant::now() - start
    );
    texture
}

fn create_buffer_with_data<T>(device: &dyn Device, usage: BufferUsageFlags, data: &[T]) -> Buffer
where
    T: Blittable,
{
    let len = data.len() * std::mem::size_of::<T>();
    let buffer = device.create_buffer(&BufferDesc {
        location: MemoryLocation::HostMapped,
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

fn create_image_with_data(
    device: &dyn Device,
    thread_token: &mut ThreadToken,
    width: u32,
    height: u32,
    data: &[u8],
) -> Image {
    let frame = device.begin_frame();

    let buffer = create_buffer_with_data(device, BufferUsageFlags::TRANSFER_SRC, data);

    let image = device.create_image(&ImageDesc {
        location: MemoryLocation::Device,
        usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
        dimension: ImageDimension::Type2d,
        format: ImageFormat::RGBA8_SRGB,
        initial_layout: ImageLayout::Optimal,
        width,
        height,
        depth: 1,
        layer_count: 1,
        mip_levels: 1,
    });

    let mut cmd_buffer = device.create_cmd_buffer(&frame, thread_token);

    device.cmd_barrier(
        &mut cmd_buffer,
        None,
        &[ImageBarrier::with_access_optimal(
            &[Access::None],
            &[Access::TransferWrite],
            image,
        )],
    );

    device.cmd_copy_buffer_to_image(
        &mut cmd_buffer,
        buffer,
        image,
        ImageLayout::Optimal,
        &[BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: default(),
            image_offset: Offset3d { x: 0, y: 0, z: 0 },
            image_extent: Extent3d {
                width,
                height,
                depth: 1,
            },
        }],
    );

    device.cmd_barrier(
        &mut cmd_buffer,
        None,
        &[ImageBarrier::with_access_optimal(
            &[Access::TransferWrite],
            &[Access::FragmentShaderSampledImageRead],
            image,
        )],
    );

    device.submit(&frame, cmd_buffer);

    device.destroy_buffer(&frame, buffer);

    device.end_frame(frame);

    image
}

struct MappedBuffer<'a> {
    device: &'a dyn Device,
    buffer: Buffer,
    slice: &'a mut [u8],
}

impl<'a> MappedBuffer<'a> {
    pub fn new(device: &'a dyn Device, usage: BufferUsageFlags, len: usize) -> Self {
        let buffer = device.create_buffer(&BufferDesc {
            location: MemoryLocation::HostMapped,
            usage,
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

    pub fn write_slice<T>(&mut self, values: &[T])
    where
        T: Blittable,
    {
        unsafe {
            let len = values.len() * std::mem::size_of::<T>();
            let src = std::slice::from_raw_parts(values.as_ptr() as *const u8, len);
            self.slice[..len].copy_from_slice(src)
        }
    }
}

impl<'a> Drop for MappedBuffer<'a> {
    fn drop(&mut self) {
        // Safety: Make sure we don't have the slice outlive the mapping.
        unsafe {
            self.device.unmap_buffer(self.buffer);
        }
    }
}

pub fn main() {
    let app = create_app();
    let main_window = app.create_window(&WindowDesc {
        title: "narcissus",
        width: 800,
        height: 600,
    });
    let device = create_device(narcissus_gpu::DeviceBackend::Vulkan);

    let mut thread_token = ThreadToken::new();

    let vert_spv = include_bytes_align!(4, "shaders/basic.vert.spv");
    let frag_spv = include_bytes_align!(4, "shaders/basic.frag.spv");

    let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDesc {
        entries: &[BindGroupLayoutEntryDesc {
            slot: 0,
            stages: ShaderStageFlags::ALL,
            binding_type: BindingType::UniformBuffer,
            count: 1,
        }],
    });

    let storage_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDesc {
        entries: &[
            BindGroupLayoutEntryDesc {
                slot: 0,
                stages: ShaderStageFlags::ALL,
                binding_type: BindingType::StorageBuffer,
                count: 1,
            },
            BindGroupLayoutEntryDesc {
                slot: 1,
                stages: ShaderStageFlags::ALL,
                binding_type: BindingType::StorageBuffer,
                count: 1,
            },
            BindGroupLayoutEntryDesc {
                slot: 2,
                stages: ShaderStageFlags::ALL,
                binding_type: BindingType::Sampler,
                count: 1,
            },
            BindGroupLayoutEntryDesc {
                slot: 3,
                stages: ShaderStageFlags::ALL,
                binding_type: BindingType::Image,
                count: 1,
            },
        ],
    });

    let pipeline = device.create_graphics_pipeline(&GraphicsPipelineDesc {
        vertex_shader: ShaderDesc {
            entry: cstr!("main"),
            code: vert_spv,
        },
        fragment_shader: ShaderDesc {
            entry: cstr!("main"),
            code: frag_spv,
        },
        bind_group_layouts: &[uniform_bind_group_layout, storage_bind_group_layout],
        layout: GraphicsPipelineLayout {
            color_attachment_formats: &[ImageFormat::BGRA8_SRGB],
            depth_attachment_format: Some(ImageFormat::DEPTH_F32),
            stencil_attachment_format: None,
        },
        topology: Topology::Triangles,
        polygon_mode: PolygonMode::Fill,
        culling_mode: CullingMode::Back,
        front_face: FrontFace::CounterClockwise,
        blend_mode: BlendMode::Opaque,
        depth_bias: None,
        depth_compare_op: CompareOp::GreaterOrEqual,
        depth_test_enable: true,
        depth_write_enable: true,
        stencil_test_enable: false,
        stencil_back: default(),
        stencil_front: default(),
    });

    let blåhaj_image = load_image("narcissus/data/blåhaj.png");
    let (blåhaj_vertices, blåhaj_indices) = load_obj("narcissus/data/blåhaj.obj");

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

    let blåhaj_image = create_image_with_data(
        device.as_ref(),
        &mut thread_token,
        blåhaj_image.width() as u32,
        blåhaj_image.height() as u32,
        blåhaj_image.as_slice(),
    );

    let mut uniforms = MappedBuffer::new(
        device.as_ref(),
        BufferUsageFlags::UNIFORM,
        std::mem::size_of::<Uniforms>(),
    );

    let mut transforms = MappedBuffer::new(
        device.as_ref(),
        BufferUsageFlags::STORAGE,
        std::mem::size_of::<Affine3>() * MAX_SHARKS,
    );

    let sampler = device.create_sampler(&SamplerDesc {
        filter: SamplerFilter::Point,
        address_mode: SamplerAddressMode::Clamp,
        compare_op: None,
        mip_lod_bias: 0.0,
        min_lod: 0.0,
        max_lod: 1000.0,
    });

    let mut depth_width = 0;
    let mut depth_height = 0;
    let mut depth_image = default();

    let shark_distance = 4.0;

    let mut rng = Pcg64::new();

    let mut shark_transforms = Vec::new();
    for z in 0..NUM_SHARKS {
        for x in 0..NUM_SHARKS {
            let x = x as f32 * shark_distance - NUM_SHARKS as f32 / 2.0 * shark_distance;
            let z = z as f32 * shark_distance - NUM_SHARKS as f32 / 2.0 * shark_distance;
            shark_transforms.push(Affine3 {
                matrix: Mat3::from_axis_rotation(Vec3::Y, HalfTurn::new(rng.next_f32())),
                translate: vec3(x, 0.0, z),
            })
        }
    }

    let start_time = Instant::now();
    'main: loop {
        let frame = device.begin_frame();

        while let Some(event) = app.poll_event() {
            use Event::*;
            match event {
                KeyPress {
                    window_id: _,
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
                Close { window_id } => {
                    let window = app.window(window_id);
                    device.destroy_swapchain(window.upcast());
                }
                _ => {}
            }
        }

        let (width, height, swapchain_image) = loop {
            let (width, height) = main_window.extent();
            if let Ok(result) = device.acquire_swapchain(
                &frame,
                main_window.upcast(),
                width,
                height,
                ImageFormat::BGRA8_SRGB,
            ) {
                break result;
            }
        };

        let frame_start = Instant::now() - start_time;
        let frame_start = frame_start.as_secs_f32() * 0.125;

        for (i, transform) in shark_transforms.iter_mut().enumerate() {
            let direction = if i & 1 == 0 { 1.0 } else { -1.0 };
            let (s, _) = sin_cos_pi_f32(frame_start + (i as f32) * 0.0125);
            transform.translate.y = s;
            transform.matrix *= Mat3::from_axis_rotation(Vec3::Y, HalfTurn::new(0.002 * direction))
        }

        transforms.write_slice(&shark_transforms);

        let (s, c) = sin_cos_pi_f32(frame_start * 0.2);
        let camera_height = c * 8.0;
        let camera_radius = 20.0;
        let eye = Point3::new(s * camera_radius, camera_height, c * camera_radius);
        let center = Point3::ZERO;
        let camera_from_model = Mat4::look_at(eye, center, Vec3::Y);
        let clip_from_camera =
            Mat4::perspective_rev_inf_zo(Deg::new(45.0).into(), width as f32 / height as f32, 0.01);
        let clip_from_model = clip_from_camera * camera_from_model;

        uniforms.write(Uniforms { clip_from_model });

        if width != depth_width || height != depth_height {
            device.destroy_image(&frame, depth_image);
            depth_image = device.create_image(&ImageDesc {
                location: MemoryLocation::HostMapped,
                usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                dimension: ImageDimension::Type2d,
                format: ImageFormat::DEPTH_F32,
                initial_layout: ImageLayout::Optimal,
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
            &[
                Bind {
                    binding: 0,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[blåhaj_vertex_buffer]),
                },
                Bind {
                    binding: 1,
                    array_element: 0,
                    typed: TypedBind::StorageBuffer(&[transforms.buffer()]),
                },
                Bind {
                    binding: 2,
                    array_element: 0,
                    typed: TypedBind::Sampler(&[sampler]),
                },
                Bind {
                    binding: 3,
                    array_element: 0,
                    typed: TypedBind::Image(&[(ImageLayout::Optimal, blåhaj_image)]),
                },
            ],
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
                    image: swapchain_image,
                    load_op: LoadOp::Clear(ClearValue::ColorF32([
                        0.392157, 0.584314, 0.929412, 1.0,
                    ])),
                    store_op: StoreOp::Store,
                }],
                depth_attachment: Some(RenderingAttachment {
                    image: depth_image,
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
                offset: Offset2d { x: 0, y: 0 },
                extent: Extent2d { width, height },
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

        device.cmd_draw_indexed(
            &mut cmd_buffer,
            blåhaj_indices.len() as u32,
            shark_transforms.len() as u32,
            0,
            0,
            0,
        );

        device.cmd_end_rendering(&mut cmd_buffer);

        device.submit(&frame, cmd_buffer);

        device.end_frame(frame);
    }
}
