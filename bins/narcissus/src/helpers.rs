use std::path::Path;

use narcissus_core::{default, obj, Widen};
use narcissus_gpu::{
    Access, Buffer, BufferDesc, BufferImageCopy, BufferUsageFlags, Device, Extent3d, Image,
    ImageAspectFlags, ImageBarrier, ImageDesc, ImageDimension, ImageFormat, ImageLayout,
    ImageUsageFlags, MemoryLocation, Offset3d, ThreadToken,
};
use narcissus_image as image;
use narcissus_maths::{vec2, vec3, vec4, Vec2, Vec3};

use crate::{pipelines::Vertex, Blittable};

pub fn load_obj<P: AsRef<Path>>(path: P) -> (Vec<Vertex>, Vec<u16>) {
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
            let position = visitor.positions[position_index.widen() - 1];
            let normal = visitor.normals[normal_index.widen() - 1];
            let texcoord = visitor.texcoords[texcoord_index.widen() - 1];
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

pub fn load_image<P: AsRef<Path>>(path: P) -> image::Image {
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

pub fn create_buffer_with_data<T>(
    device: &dyn Device,
    usage: BufferUsageFlags,
    data: &[T],
) -> Buffer
where
    T: Blittable,
{
    // SAFETY: T: Blittable which implies it's freely convertable to a byte slice.
    unsafe {
        let len = data.len() * std::mem::size_of::<T>();
        let initial_data = std::slice::from_raw_parts(data.as_ptr() as *const u8, len);
        device.create_buffer_with_data(
            &BufferDesc {
                location: MemoryLocation::HostMapped,
                usage,
                size: len,
            },
            initial_data,
        )
    }
}

pub fn create_image_with_data(
    device: &dyn Device,
    thread_token: &ThreadToken,
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
        &[ImageBarrier::layout_optimal(
            &[Access::None],
            &[Access::TransferWrite],
            image,
            ImageAspectFlags::COLOR,
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
        &[ImageBarrier::layout_optimal(
            &[Access::TransferWrite],
            &[Access::FragmentShaderSampledImageRead],
            image,
            ImageAspectFlags::COLOR,
        )],
    );

    device.submit(&frame, cmd_buffer);

    device.destroy_buffer(&frame, buffer);

    device.end_frame(frame);

    image
}
