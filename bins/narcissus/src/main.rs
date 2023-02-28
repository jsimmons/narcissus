use std::time::Instant;

use crate::{
    fonts::{FontFamily, Fonts},
    pipelines::{BasicPipeline, TextPipeline},
};
use helpers::{create_buffer_with_data, create_image_with_data, load_image, load_obj};
use mapped_buffer::MappedBuffer;
use narcissus_app::{create_app, Event, Key, WindowDesc};
use narcissus_core::{default, rand::Pcg64};
use narcissus_font::{FontCollection, GlyphCache, TouchedGlyph, TouchedGlyphInfo};
use narcissus_gpu::{
    create_device, Access, BufferImageCopy, BufferUsageFlags, ClearValue, Extent2d, Extent3d,
    ImageAspectFlags, ImageBarrier, ImageDesc, ImageDimension, ImageFormat, ImageLayout,
    ImageUsageFlags, LoadOp, MemoryLocation, Offset2d, Offset3d, RenderingAttachment,
    RenderingDesc, Scissor, StoreOp, ThreadToken, Viewport,
};
use narcissus_maths::{sin_cos_pi_f32, vec3, Affine3, Deg, HalfTurn, Mat3, Mat4, Point3, Vec3};
use pipelines::{BasicUniforms, GlyphInstance, TextUniforms};

mod fonts;
mod helpers;
mod mapped_buffer;
mod pipelines;

const MAX_SHARKS: usize = 262_144;
const NUM_SHARKS: usize = 50;

const GLYPH_CACHE_WIDTH: usize = 1024;
const GLYPH_CACHE_HEIGHT: usize = 512;
const MAX_GLYPH_INSTANCES: usize = 262_144;
const MAX_GLYPHS: usize = 8192;

/// Marker trait indicates it's safe to convert a given type directly to an array of bytes.
///
/// # Safety
///
/// Must not be applied to any types with padding
pub unsafe trait Blittable: Sized {}

unsafe impl Blittable for u8 {}
unsafe impl Blittable for u16 {}
unsafe impl Blittable for Affine3 {}
unsafe impl Blittable for TouchedGlyph {}

pub fn main() {
    let app = create_app();
    let main_window = app.create_window(&WindowDesc {
        title: "narcissus",
        width: 800,
        height: 600,
    });
    let device = create_device(narcissus_gpu::DeviceBackend::Vulkan);

    let thread_token = ThreadToken::new();

    let basic_pipeline = BasicPipeline::new(device.as_ref());
    let text_pipeline = TextPipeline::new(device.as_ref());

    let fonts = Fonts::new();
    let mut glyph_cache = GlyphCache::new(&fonts, GLYPH_CACHE_WIDTH, GLYPH_CACHE_HEIGHT, 1);

    let blåhaj_image = load_image("bins/narcissus/data/blåhaj.png");
    let (blåhaj_vertices, blåhaj_indices) = load_obj("bins/narcissus/data/blåhaj.obj");

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
        &thread_token,
        blåhaj_image.width() as u32,
        blåhaj_image.height() as u32,
        blåhaj_image.as_slice(),
    );

    let mut basic_uniform_buffer = MappedBuffer::new(
        device.as_ref(),
        BufferUsageFlags::UNIFORM,
        std::mem::size_of::<BasicUniforms>(),
    );

    let mut basic_transform_buffer = MappedBuffer::new(
        device.as_ref(),
        BufferUsageFlags::STORAGE,
        std::mem::size_of::<Affine3>() * MAX_SHARKS,
    );

    let mut text_uniform_buffer = MappedBuffer::new(
        device.as_ref(),
        BufferUsageFlags::UNIFORM,
        std::mem::size_of::<TextUniforms>(),
    );

    let mut glyph_instance_buffer = MappedBuffer::new(
        device.as_ref(),
        BufferUsageFlags::STORAGE,
        std::mem::size_of::<GlyphInstance>() * MAX_GLYPH_INSTANCES,
    );

    let mut glyph_buffer = MappedBuffer::new(
        device.as_ref(),
        BufferUsageFlags::STORAGE,
        std::mem::size_of::<TouchedGlyph>() * MAX_GLYPHS,
    );

    let glyph_atlas = device.create_image(&ImageDesc {
        location: MemoryLocation::Device,
        usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
        dimension: ImageDimension::Type2d,
        format: ImageFormat::R8_UNORM,
        initial_layout: ImageLayout::Optimal,
        width: glyph_cache.width() as u32,
        height: glyph_cache.height() as u32,
        depth: 1,
        layer_count: 1,
        mip_levels: 1,
    });

    {
        let frame = device.begin_frame();
        let mut cmd_buffer = device.create_cmd_buffer(&frame, &thread_token);
        device.cmd_barrier(
            &mut cmd_buffer,
            None,
            &[ImageBarrier::layout_optimal(
                &[Access::None],
                &[Access::ShaderSampledImageRead],
                glyph_atlas,
                ImageAspectFlags::COLOR,
            )],
        );
        device.submit(&frame, cmd_buffer);
        device.end_frame(frame);
    }

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

        let mut cmd_buffer = device.create_cmd_buffer(&frame, &thread_token);

        if width != depth_width || height != depth_height {
            device.destroy_image(&frame, depth_image);
            depth_image = device.create_image(&ImageDesc {
                location: MemoryLocation::Device,
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

            device.cmd_barrier(
                &mut cmd_buffer,
                None,
                &[ImageBarrier::layout_optimal(
                    &[Access::None],
                    &[Access::DepthStencilAttachmentWrite],
                    depth_image,
                    ImageAspectFlags::DEPTH,
                )],
            );

            depth_width = width;
            depth_height = height;
        }

        let frame_start = Instant::now() - start_time;
        let frame_start = frame_start.as_secs_f32() * 0.125;

        for (i, transform) in shark_transforms.iter_mut().enumerate() {
            let direction = if i & 1 == 0 { 1.0 } else { -1.0 };
            let (s, _) = sin_cos_pi_f32(frame_start + (i as f32) * 0.0125);
            transform.translate.y = s;
            transform.matrix *= Mat3::from_axis_rotation(Vec3::Y, HalfTurn::new(0.002 * direction))
        }

        basic_transform_buffer.write_slice(&shark_transforms);

        let (s, c) = sin_cos_pi_f32(frame_start * 0.2);
        let camera_height = c * 8.0;
        let camera_radius = 20.0;
        let eye = Point3::new(s * camera_radius, camera_height, c * camera_radius);
        let center = Point3::ZERO;
        let camera_from_model = Mat4::look_at(eye, center, Vec3::Y);
        let clip_from_camera =
            Mat4::perspective_rev_inf_zo(Deg::new(45.0).into(), width as f32 / height as f32, 0.01);
        let clip_from_model = clip_from_camera * camera_from_model;

        basic_uniform_buffer.write(BasicUniforms { clip_from_model });

        // Do some Font Shit.'
        let line0 = "Snarfe, Blåhaj! And the Quick Brown Fox jumped Over the Lazy doge. ½½½½ Snarfe, Blåhaj! And the Quick Brown Fox jumped Over the Lazy doge. ½½½½ Snarfe, Blåhaj! And the Quick Brown Fox jumped Over the Lazy doge. ½½½½";
        let line1 = "加盟国は、国際連合と協力して 加盟国は、国際連合と協力して 加盟国は、国際連合と協力して 加盟国は、国際連合と協力して 加盟国は、国際連合と協力して 加盟国は、国際連合と協力して";

        let mut glyph_instances = Vec::new();

        let mut x;
        let mut y = 0.0;

        let mut rng = Pcg64::new();

        for line in 0..34 {
            let (font_family, font_size_px, text) = if line & 1 == 0 {
                (FontFamily::RobotoRegular, 10.0, line0)
            } else {
                (FontFamily::NotoSansJapanese, 20.0, line1)
            };

            let font_size_px = font_size_px + (line / 2) as f32 * 2.0;

            let font = fonts.font(font_family);
            let scale = font.scale_for_size_px(font_size_px);

            x = 0.0;
            y += (font.ascent() - font.descent() + font.line_gap()) * scale;
            y = y.trunc();

            let mut prev_glyph_index = None;
            for c in text.chars() {
                let TouchedGlyphInfo {
                    touched_glyph_index,
                    glyph_index,
                    advance_width,
                } = glyph_cache.touch_glyph(font_family, c, font_size_px);

                if let Some(prev_glyph_index) = prev_glyph_index.replace(glyph_index) {
                    x += font.kerning_advance(prev_glyph_index, glyph_index) * scale;
                }

                const COLOR_SERIES: [u32; 4] = [0xfffac228, 0xfff57d15, 0xffd44842, 0xff9f2a63];
                let color = COLOR_SERIES[rng.next_bound_u64(4) as usize];

                glyph_instances.push(GlyphInstance {
                    x,
                    y,
                    touched_glyph_index,
                    color,
                });

                x += advance_width * scale;
            }
        }

        let atlas_width = glyph_cache.width() as u32;
        let atlas_height = glyph_cache.height() as u32;

        text_uniform_buffer.write(TextUniforms {
            screen_width: width,
            screen_height: height,
            atlas_width,
            atlas_height,
        });

        glyph_instance_buffer.write_slice(&glyph_instances);

        let (touched_glyphs, texture) = glyph_cache.update_atlas();

        // Update information for the glyphs we need this frame.
        glyph_buffer.write_slice(touched_glyphs);

        // If the atlas has been updated, we need to upload it to the GPU.
        if let Some(texture) = texture {
            let width = atlas_width;
            let height = atlas_height;
            let image = glyph_atlas;
            let data = texture;

            let buffer =
                create_buffer_with_data(device.as_ref(), BufferUsageFlags::TRANSFER_SRC, data);

            device.cmd_barrier(
                &mut cmd_buffer,
                None,
                &[ImageBarrier::layout_optimal(
                    &[Access::ShaderSampledImageRead],
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

            device.destroy_buffer(&frame, buffer);
        }

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

        // Render basic stuff.
        basic_pipeline.bind(
            device.as_ref(),
            &frame,
            &mut cmd_buffer,
            basic_uniform_buffer.buffer(),
            blåhaj_vertex_buffer,
            blåhaj_index_buffer,
            basic_transform_buffer.buffer(),
            blåhaj_image,
        );

        device.cmd_draw_indexed(
            &mut cmd_buffer,
            blåhaj_indices.len() as u32,
            shark_transforms.len() as u32,
            0,
            0,
            0,
        );

        // Render text stuff.
        text_pipeline.bind(
            device.as_ref(),
            &frame,
            &mut cmd_buffer,
            text_uniform_buffer.buffer(),
            glyph_buffer.buffer(),
            glyph_instance_buffer.buffer(),
            glyph_atlas,
        );

        device.cmd_draw(&mut cmd_buffer, 4, glyph_instances.len() as u32, 0, 0);

        device.cmd_end_rendering(&mut cmd_buffer);

        device.submit(&frame, cmd_buffer);

        device.end_frame(frame);
    }
}
