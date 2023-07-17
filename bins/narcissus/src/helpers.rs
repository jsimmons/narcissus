use std::path::Path;

use narcissus_core::{obj, Widen};
use narcissus_gpu::{Buffer, BufferDesc, BufferUsageFlags, Device, MemoryLocation};
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

pub fn create_host_buffer_with_data<T>(
    device: &dyn Device,
    usage: BufferUsageFlags,
    data: &[T],
) -> Buffer
where
    T: Blittable,
{
    // SAFETY: T: Blittable which implies it's freely convertable to a byte slice.
    unsafe {
        let len = std::mem::size_of_val(data);
        let initial_data = std::slice::from_raw_parts(data.as_ptr() as *const u8, len);
        device.create_buffer_with_data(
            &BufferDesc {
                memory_location: MemoryLocation::Host,
                host_mapped: true,
                usage,
                size: len,
            },
            initial_data,
        )
    }
}
