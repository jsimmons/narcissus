use std::{fmt::Write, time::Instant};

use crate::{
    fonts::{FontFamily, Fonts},
    pipelines::{BasicPipeline, TextPipeline},
};
use helpers::{load_image, load_obj};
use narcissus_app::{create_app, Event, Key, PressedState, WindowDesc};
use narcissus_core::{default, rand::Pcg64, slice::array_windows};
use narcissus_font::{FontCollection, GlyphCache, HorizontalMetrics};
use narcissus_gpu::{
    create_device, Access, BufferDesc, BufferImageCopy, BufferUsageFlags, ClearValue, DeviceExt,
    Extent2d, Extent3d, ImageAspectFlags, ImageBarrier, ImageDesc, ImageDimension, ImageFormat,
    ImageLayout, ImageTiling, ImageUsageFlags, LoadOp, MemoryLocation, Offset2d, Offset3d,
    RenderingAttachment, RenderingDesc, Scissor, StoreOp, ThreadToken, Viewport,
};
use narcissus_maths::{sin_cos_pi_f32, vec3, Affine3, HalfTurn, Mat3, Mat4, Point3, Vec3};
use pipelines::{BasicUniforms, PrimitiveInstance, PrimitiveVertex, TextUniforms};

mod fonts;
mod helpers;
mod pipelines;

const NUM_SHARKS: usize = 50;
const GLYPH_CACHE_SIZE: usize = 1024;

pub fn main() {
    let app = create_app();
    let main_window = app.create_window(&WindowDesc {
        title: "shark",
        width: 800,
        height: 600,
    });
    let device = create_device(narcissus_gpu::DeviceBackend::Vulkan);

    let thread_token = ThreadToken::new();

    let basic_pipeline = BasicPipeline::new(device.as_ref());
    let text_pipeline = TextPipeline::new(device.as_ref());

    let fonts = Fonts::new();
    let mut glyph_cache = GlyphCache::new(&fonts, GLYPH_CACHE_SIZE, GLYPH_CACHE_SIZE, 1);

    let blåhaj_image_data = load_image("title/shark/data/blåhaj.png");
    let (blåhaj_vertices, blåhaj_indices) = load_obj("title/shark/data/blåhaj.obj");

    let blåhaj_vertex_buffer = device.create_persistent_buffer_with_data(
        MemoryLocation::Device,
        BufferUsageFlags::STORAGE,
        blåhaj_vertices.as_slice(),
    );

    let blåhaj_index_buffer = device.create_persistent_buffer_with_data(
        MemoryLocation::Device,
        BufferUsageFlags::INDEX,
        blåhaj_indices.as_slice(),
    );

    let blåhaj_image = device.create_image(&ImageDesc {
        memory_location: MemoryLocation::Device,
        host_mapped: false,
        usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER,
        dimension: ImageDimension::Type2d,
        format: ImageFormat::RGBA8_SRGB,
        tiling: ImageTiling::Optimal,
        width: blåhaj_image_data.width() as u32,
        height: blåhaj_image_data.height() as u32,
        depth: 1,
        layer_count: 1,
        mip_levels: 1,
    });

    let glyph_atlas = device.create_image(&ImageDesc {
        memory_location: MemoryLocation::Device,
        usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER,
        host_mapped: false,
        dimension: ImageDimension::Type2d,
        format: ImageFormat::R8_UNORM,
        tiling: ImageTiling::Optimal,
        width: glyph_cache.width() as u32,
        height: glyph_cache.height() as u32,
        depth: 1,
        layer_count: 1,
        mip_levels: 1,
    });

    let mut rng = Pcg64::new();
    let mut buffers = (0..4096)
        .map(|_| {
            device.create_buffer(&BufferDesc {
                memory_location: MemoryLocation::Host,
                host_mapped: true,
                usage: BufferUsageFlags::STORAGE,
                size: 16 + rng.next_bound_usize(1024 - 16),
            })
        })
        .collect::<Vec<_>>();

    buffers.extend((0..512).map(|_| {
        device.create_buffer(&BufferDesc {
            memory_location: MemoryLocation::Host,
            host_mapped: true,
            usage: BufferUsageFlags::STORAGE,
            size: 16 + rng.next_bound_usize(10 * 1024 * 1024 - 16),
        })
    }));

    {
        let frame = device.begin_frame();

        for buffer in buffers.drain(..) {
            device.destroy_buffer(&frame, buffer);
        }

        let blåhaj_buffer = device.request_transient_buffer_with_data(
            &frame,
            &thread_token,
            BufferUsageFlags::TRANSFER,
            blåhaj_image_data.as_slice(),
        );

        let mut cmd_encoder = device.request_cmd_encoder(&frame, &thread_token);

        device.cmd_barrier(
            &mut cmd_encoder,
            None,
            &[
                ImageBarrier::layout_optimal(
                    &[Access::None],
                    &[Access::ShaderSampledImageRead],
                    glyph_atlas,
                    ImageAspectFlags::COLOR,
                ),
                ImageBarrier::layout_optimal(
                    &[Access::None],
                    &[Access::TransferWrite],
                    blåhaj_image,
                    ImageAspectFlags::COLOR,
                ),
            ],
        );

        device.cmd_copy_buffer_to_image(
            &mut cmd_encoder,
            blåhaj_buffer.to_arg(),
            blåhaj_image,
            ImageLayout::Optimal,
            &[BufferImageCopy {
                buffer_offset: 0,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: default(),
                image_offset: Offset3d { x: 0, y: 0, z: 0 },
                image_extent: Extent3d {
                    width: blåhaj_image_data.width() as u32,
                    height: blåhaj_image_data.width() as u32,
                    depth: 1,
                },
            }],
        );

        device.cmd_barrier(
            &mut cmd_encoder,
            None,
            &[ImageBarrier::layout_optimal(
                &[Access::TransferWrite],
                &[Access::FragmentShaderSampledImageRead],
                blåhaj_image,
                ImageAspectFlags::COLOR,
            )],
        );

        device.submit(&frame, cmd_encoder);
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
                matrix: Mat3::from_axis_rotation(Vec3::Y, HalfTurn::new(rng.next_f32() * 2.0)),
                translate: vec3(x, 0.0, z),
            })
        }
    }

    let mut font_size_str = String::new();
    let mut primitive_instances = Vec::new();
    let mut primitive_vertices = Vec::new();
    let mut line_glyph_indices = Vec::new();
    let mut line_kern_advances = Vec::new();

    let mut align_v = false;
    let mut kerning = true;

    let start_time = Instant::now();
    'main: loop {
        let frame = device.begin_frame();

        while let Some(event) = app.poll_event() {
            use Event::*;
            match event {
                KeyPress {
                    window_id: _,
                    key,
                    pressed,
                    modifiers: _,
                } => {
                    if key == Key::Escape {
                        break 'main;
                    }
                    if key == Key::Space && pressed == PressedState::Pressed {
                        align_v = !align_v;
                        println!("align: {align_v}");
                    }
                    if key == Key::K && pressed == PressedState::Pressed {
                        kerning = !kerning;
                        println!("kerning: {kerning}");
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

        let mut cmd_buffer = device.request_cmd_encoder(&frame, &thread_token);

        if width != depth_width || height != depth_height {
            device.destroy_image(&frame, depth_image);
            depth_image = device.create_image(&ImageDesc {
                memory_location: MemoryLocation::Device,
                host_mapped: false,
                usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                dimension: ImageDimension::Type2d,
                format: ImageFormat::DEPTH_F32,
                tiling: ImageTiling::Optimal,
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

        let (s, c) = sin_cos_pi_f32(frame_start * 0.2);
        let camera_height = c * 8.0;
        let camera_radius = 20.0;
        let eye = Point3::new(s * camera_radius, camera_height, c * camera_radius);
        let center = Point3::ZERO;
        let camera_from_model = Mat4::look_at(eye, center, Vec3::Y);
        let clip_from_camera = Mat4::perspective_rev_inf_zo(
            HalfTurn::new(1.0 / 3.0),
            width as f32 / height as f32,
            0.01,
        );
        let clip_from_model = clip_from_camera * camera_from_model;

        // Do some Font Shit.'
        let line0 = "Snarfe, Blåhaj! And the Quick Brown Fox jumped Over the Lazy doge.";
        let line1 = "加盟国は、国際連合と協力して";

        let mut x;
        let mut y = 0.0;

        let mut rng = Pcg64::new();

        primitive_instances.clear();
        primitive_vertices.clear();

        for line in 0.. {
            let (font_family, font_size_px, text) = if line & 1 == 0 {
                (FontFamily::RobotoRegular, 14.0, line0)
            } else {
                (FontFamily::NotoSansJapanese, 14.0, line1)
            };

            let font = fonts.font(font_family);
            let scale = font.scale_for_size_px(font_size_px);

            x = 0.0;
            y += (font.ascent() - font.descent() + font.line_gap()) * scale;
            if align_v {
                y = y.trunc();
            }

            if y > height as f32 {
                break;
            }

            font_size_str.clear();
            write!(&mut font_size_str, "{font_size_px}: ").unwrap();

            line_glyph_indices.clear();
            line_glyph_indices.extend(font_size_str.chars().chain(text.chars()).map(|c| {
                font.glyph_index(c)
                    .unwrap_or_else(|| font.glyph_index('□').unwrap())
            }));

            line_kern_advances.clear();
            line_kern_advances.push(0.0);
            line_kern_advances.extend(
                array_windows(line_glyph_indices.as_slice())
                    .map(|&[prev_index, next_index]| font.kerning_advance(prev_index, next_index)),
            );

            'repeat_str: for _ in 0.. {
                for (glyph_index, advance) in line_glyph_indices
                    .iter()
                    .copied()
                    .zip(line_kern_advances.iter().copied())
                {
                    if x >= width as f32 {
                        break 'repeat_str;
                    }

                    let touched_glyph_index =
                        glyph_cache.touch_glyph(font_family, glyph_index, font_size_px);

                    let HorizontalMetrics {
                        advance_width,
                        left_side_bearing: _,
                    } = font.horizontal_metrics(glyph_index);

                    if kerning {
                        x += advance * scale;
                    }

                    let color =
                        *rng.array_select(&[0xfffac228, 0xfff57d15, 0xffd44842, 0xff9f2a63]);

                    let instance_index = primitive_instances.len() as u32;
                    primitive_instances.push(PrimitiveInstance {
                        x,
                        y,
                        touched_glyph_index,
                        color,
                    });
                    let glyph_vertices = &[
                        PrimitiveVertex::glyph(0, instance_index),
                        PrimitiveVertex::glyph(1, instance_index),
                        PrimitiveVertex::glyph(2, instance_index),
                        PrimitiveVertex::glyph(2, instance_index),
                        PrimitiveVertex::glyph(1, instance_index),
                        PrimitiveVertex::glyph(3, instance_index),
                    ];
                    primitive_vertices.extend_from_slice(glyph_vertices);

                    x += advance_width * scale;
                }
            }
        }

        let atlas_width = glyph_cache.width() as u32;
        let atlas_height = glyph_cache.height() as u32;

        let (touched_glyphs, texture) = glyph_cache.update_atlas();

        // If the atlas has been updated, we need to upload it to the GPU.
        if let Some(texture) = texture {
            let width = atlas_width;
            let height = atlas_height;
            let image = glyph_atlas;

            let buffer = device.request_transient_buffer_with_data(
                &frame,
                &thread_token,
                BufferUsageFlags::TRANSFER,
                texture,
            );

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
                buffer.to_arg(),
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

            device.cmd_copy_buffer_to_image(
                &mut cmd_buffer,
                buffer.to_arg(),
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
                    load_op: LoadOp::Clear(ClearValue::ColorF32([1.0, 1.0, 1.0, 1.0])),
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
            &thread_token,
            &mut cmd_buffer,
            &BasicUniforms { clip_from_model },
            &blåhaj_vertex_buffer,
            &blåhaj_index_buffer,
            shark_transforms.as_slice(),
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
            &thread_token,
            &mut cmd_buffer,
            &TextUniforms {
                screen_width: width,
                screen_height: height,
                atlas_width,
                atlas_height,
            },
            primitive_vertices.as_slice(),
            touched_glyphs,
            primitive_instances.as_slice(),
            glyph_atlas,
        );

        device.cmd_draw(&mut cmd_buffer, primitive_vertices.len() as u32, 1, 0, 0);

        device.cmd_end_rendering(&mut cmd_buffer);

        device.submit(&frame, cmd_buffer);

        device.end_frame(frame);
    }
}
