use std::ops::Index;
use std::path::Path;

use crate::game::GameState;
use crate::{microshades, UiState};
use narcissus_core::dds;

use shark_shaders::pipelines::{
    calculate_spine_size, BasicConstants, CompositeConstants, ComputeBinds, Draw2dClearConstants,
    Draw2dRasterizeConstants, Draw2dResolveConstants, Draw2dScatterConstants, Draw2dSortConstants,
    GraphicsBinds, Pipelines, RadixSortDownsweepConstants, RadixSortUpsweepConstants,
    DRAW_2D_TILE_SIZE,
};

use crate::helpers::load_obj;
use narcissus_core::{default, BitIter};
use narcissus_gpu::{
    Access, Bind, BufferImageCopy, BufferUsageFlags, ClearValue, CmdEncoder, DeviceExt, Extent2d,
    Extent3d, Frame, GlobalBarrier, Gpu, Image, ImageAspectFlags, ImageBarrier, ImageDesc,
    ImageDimension, ImageFormat, ImageLayout, ImageSubresourceRange, ImageTiling, ImageUsageFlags,
    IndexType, LoadOp, MemoryLocation, Offset2d, PersistentBuffer, RenderingAttachment,
    RenderingDesc, Scissor, ShaderStageFlags, StoreOp, ThreadToken, TypedBind, Viewport,
};
use narcissus_image as image;
use narcissus_maths::{vec3, Affine3, HalfTurn, Mat3, Mat4, Vec3};

pub struct Model<'a> {
    indices: u32,
    vertex_buffer: PersistentBuffer<'a>,
    index_buffer: PersistentBuffer<'a>,
}

enum ModelRes {
    Shark,
}

struct Models<'a> {
    shark: Model<'a>,
}

impl<'a> Index<ModelRes> for Models<'a> {
    type Output = Model<'a>;

    fn index(&self, index: ModelRes) -> &Self::Output {
        match index {
            ModelRes::Shark => &self.shark,
        }
    }
}

impl<'a> Models<'a> {
    pub fn load(gpu: &'a Gpu) -> Models<'a> {
        fn load_model<P>(gpu: &Gpu, path: P) -> Model
        where
            P: AsRef<Path>,
        {
            let (vertices, indices) = load_obj(path);
            let vertex_buffer = gpu.create_persistent_buffer_with_data(
                MemoryLocation::Device,
                BufferUsageFlags::STORAGE,
                vertices.as_slice(),
            );
            let index_buffer = gpu.create_persistent_buffer_with_data(
                MemoryLocation::Device,
                BufferUsageFlags::INDEX,
                indices.as_slice(),
            );

            gpu.debug_name_buffer(vertex_buffer.to_arg(), "vertex");
            gpu.debug_name_buffer(index_buffer.to_arg(), "index");

            Model {
                indices: indices.len() as u32,
                vertex_buffer,
                index_buffer,
            }
        }

        Models {
            shark: load_model(gpu, "title/shark/data/blåhaj.obj"),
        }
    }
}

enum ImageRes {
    TonyMcMapfaceLut,
    Shark,
}

struct Images {
    tony_mc_mapface_lut: Image,
    shark: Image,
}

impl Index<ImageRes> for Images {
    type Output = Image;

    fn index(&self, index: ImageRes) -> &Self::Output {
        match index {
            ImageRes::TonyMcMapfaceLut => &self.tony_mc_mapface_lut,
            ImageRes::Shark => &self.shark,
        }
    }
}

impl Images {
    fn load(gpu: &Gpu, thread_token: &ThreadToken) -> Images {
        fn load_image<P>(
            gpu: &Gpu,
            frame: &Frame,
            thread_token: &ThreadToken,
            cmd_encoder: &mut CmdEncoder,
            path: P,
        ) -> Image
        where
            P: AsRef<Path>,
        {
            let image_data =
                image::Image::from_buffer(std::fs::read(path.as_ref()).unwrap().as_slice())
                    .unwrap();

            let width = image_data.width() as u32;
            let height = image_data.height() as u32;

            let image = gpu.create_image(&ImageDesc {
                memory_location: MemoryLocation::Device,
                host_mapped: false,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER,
                dimension: ImageDimension::Type2d,
                format: ImageFormat::RGBA8_SRGB,
                tiling: ImageTiling::Optimal,
                width,
                height,
                depth: 1,
                layer_count: 1,
                mip_levels: 1,
            });

            gpu.debug_name_image(image, path.as_ref().to_string_lossy().as_ref());

            gpu.cmd_barrier(
                cmd_encoder,
                None,
                &[ImageBarrier::optimal_discard(
                    &Access::General,
                    &Access::TransferWrite,
                    image,
                )],
            );

            let buffer = gpu.request_transient_buffer_with_data(
                frame,
                thread_token,
                BufferUsageFlags::TRANSFER,
                image_data.as_slice(),
            );

            gpu.cmd_copy_buffer_to_image(
                cmd_encoder,
                buffer.to_arg(),
                image,
                ImageLayout::Optimal,
                &[BufferImageCopy {
                    image_extent: Extent3d {
                        width,
                        height,
                        depth: 1,
                    },
                    ..default()
                }],
            );

            gpu.cmd_barrier(
                cmd_encoder,
                None,
                &[ImageBarrier::optimal(
                    &Access::TransferWrite,
                    &Access::FragmentShaderSampledImageRead,
                    image,
                )],
            );

            image
        }

        fn load_dds<P>(
            gpu: &Gpu,
            frame: &Frame,
            thread_token: &ThreadToken,
            cmd_encoder: &mut CmdEncoder,
            path: P,
        ) -> Image
        where
            P: AsRef<Path>,
        {
            let image_data = std::fs::read(path.as_ref()).unwrap();
            let dds = dds::Dds::from_buffer(&image_data).unwrap();
            let header_dxt10 = dds.header_dxt10.unwrap();

            let width = dds.header.width;
            let height = dds.header.height;
            let depth = dds.header.depth;

            let dimension = match header_dxt10.resource_dimension {
                dds::D3D10ResourceDimension::Texture1d => ImageDimension::Type1d,
                dds::D3D10ResourceDimension::Texture2d => ImageDimension::Type2d,
                dds::D3D10ResourceDimension::Texture3d => ImageDimension::Type3d,
                _ => panic!(),
            };

            let format = match header_dxt10.dxgi_format {
                dds::DxgiFormat::R9G9B9E5_SHAREDEXP => ImageFormat::E5B9G9R9_UFLOAT,
                _ => panic!(),
            };

            let image = gpu.create_image(&ImageDesc {
                memory_location: MemoryLocation::Device,
                host_mapped: false,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER,
                dimension,
                format,
                tiling: ImageTiling::Optimal,
                width,
                height,
                depth,
                layer_count: 1,
                mip_levels: 1,
            });

            gpu.debug_name_image(image, path.as_ref().to_string_lossy().as_ref());

            gpu.cmd_barrier(
                cmd_encoder,
                None,
                &[ImageBarrier::optimal_discard(
                    &Access::General,
                    &Access::TransferWrite,
                    image,
                )],
            );

            let buffer = gpu.request_transient_buffer_with_data(
                frame,
                thread_token,
                BufferUsageFlags::TRANSFER,
                dds.data,
            );

            gpu.cmd_copy_buffer_to_image(
                cmd_encoder,
                buffer.to_arg(),
                image,
                ImageLayout::Optimal,
                &[BufferImageCopy {
                    image_extent: Extent3d {
                        width,
                        height,
                        depth,
                    },
                    ..default()
                }],
            );

            gpu.cmd_barrier(
                cmd_encoder,
                None,
                &[ImageBarrier::optimal(
                    &Access::TransferWrite,
                    &Access::ShaderSampledImageRead,
                    image,
                )],
            );

            image
        }

        let images;
        let frame = gpu.begin_frame();
        {
            let frame = &frame;

            let mut cmd_encoder = gpu.request_cmd_encoder(frame, thread_token);
            {
                let cmd_encoder = &mut cmd_encoder;

                gpu.cmd_begin_debug_marker(
                    cmd_encoder,
                    "image upload",
                    microshades::BROWN_RGBA_F32[3],
                );

                images = Images {
                    tony_mc_mapface_lut: load_dds(
                        gpu,
                        frame,
                        thread_token,
                        cmd_encoder,
                        "title/shark/data/tony_mc_mapface.dds",
                    ),
                    shark: load_image(
                        gpu,
                        frame,
                        thread_token,
                        cmd_encoder,
                        "title/shark/data/blåhaj.png",
                    ),
                };

                gpu.cmd_end_debug_marker(cmd_encoder);
            }

            gpu.submit(frame, cmd_encoder);
        }
        gpu.end_frame(frame);

        images
    }
}

pub struct DrawState<'gpu> {
    gpu: &'gpu Gpu,

    width: u32,
    height: u32,

    tile_resolution_x: u32,
    tile_resolution_y: u32,

    depth_image: Image,
    color_image: Image,
    ui_image: Image,

    glyph_atlas_images: [Image; 2],
    glyph_atlas_image_index: usize,

    pipelines: Pipelines,

    models: Models<'gpu>,
    images: Images,

    transforms: Vec<Affine3>,
}

impl<'gpu> DrawState<'gpu> {
    pub fn new(gpu: &'gpu Gpu, thread_token: &ThreadToken) -> Self {
        let pipelines = Pipelines::load(gpu);
        let models = Models::load(gpu);
        let images = Images::load(gpu, thread_token);

        Self {
            gpu,
            width: 0,
            height: 0,
            tile_resolution_x: 0,
            tile_resolution_y: 0,
            depth_image: default(),
            color_image: default(),
            ui_image: default(),
            glyph_atlas_images: default(),
            glyph_atlas_image_index: 0,
            pipelines,
            models,
            images,
            transforms: vec![],
        }
    }

    pub fn draw(
        &mut self,
        thread_token: &ThreadToken,
        frame: &Frame,
        ui_state: &mut UiState,
        game_state: &GameState,
        width: u32,
        height: u32,
        swapchain_image: Image,
    ) {
        let gpu = self.gpu;

        let half_turn_y = Mat3::from_axis_rotation(Vec3::Y, HalfTurn::new(0.5));
        let scale = Mat3::from_scale(Vec3::splat(0.4));

        fn rotate_dir(dir: Vec3, up: Vec3) -> Mat3 {
            let f = dir.normalized();
            let r = Vec3::cross(f, up).normalized();
            let u = Vec3::cross(r, f);
            Mat3::from_rows([[r.x, u.x, -f.x], [r.y, u.y, -f.y], [r.z, u.z, -f.z]])
        }

        let matrix = rotate_dir(game_state.player.heading, Vec3::Y) * half_turn_y;
        let translation = game_state.player.position.as_vec3();
        self.transforms.push(Affine3::new(matrix, translation));

        let half_turn_y_scale = half_turn_y * scale;

        // Render projectiles
        for i in BitIter::new(
            game_state
                .archetype_projectile
                .bitmap_non_empty
                .iter()
                .copied(),
        ) {
            let chunk = &game_state.archetype_projectile.chunks[i];
            for (&bitmap, block) in chunk.bitmap.iter().zip(chunk.blocks.iter()) {
                if bitmap == 0 {
                    continue;
                }

                for j in 0..8 {
                    if bitmap & (1 << j) == 0 {
                        continue;
                    }

                    let translation = vec3(block.position_x[j], 0.0, block.position_z[j]);
                    let velocity = vec3(block.velocity_x[j], 0.0, block.velocity_z[j]);
                    let matrix = rotate_dir(velocity, Vec3::Y) * half_turn_y_scale;
                    self.transforms.push(Affine3::new(matrix, translation));
                }
            }
        }

        let camera_from_model = game_state.camera.camera_from_model();
        let clip_from_camera = Mat4::perspective_rev_inf_zo(
            HalfTurn::new(1.0 / 3.0),
            width as f32 / height as f32,
            0.01,
        );
        let clip_from_model = clip_from_camera * camera_from_model;

        let atlas_width = ui_state.glyph_cache.width() as u32;
        let atlas_height = ui_state.glyph_cache.height() as u32;

        let mut cmd_encoder = self.gpu.request_cmd_encoder(frame, thread_token);
        {
            let cmd_encoder = &mut cmd_encoder;

            for glyph_atlas_image in &mut self.glyph_atlas_images {
                if glyph_atlas_image.is_null() {
                    *glyph_atlas_image = gpu.create_image(&ImageDesc {
                        memory_location: MemoryLocation::Device,
                        host_mapped: false,
                        usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER,
                        dimension: ImageDimension::Type2d,
                        format: ImageFormat::R8_SRGB,
                        tiling: ImageTiling::Optimal,
                        width: atlas_width,
                        height: atlas_height,
                        depth: 1,
                        layer_count: 1,
                        mip_levels: 1,
                    });

                    gpu.debug_name_image(*glyph_atlas_image, "glyph atlas");

                    gpu.cmd_barrier(
                        cmd_encoder,
                        None,
                        &[ImageBarrier::optimal_discard(
                            &Access::None,
                            &Access::FragmentShaderSampledImageRead,
                            *glyph_atlas_image,
                        )],
                    );
                }
            }

            if width != self.width || height != self.height {
                gpu.destroy_image(frame, self.depth_image);
                gpu.destroy_image(frame, self.color_image);
                gpu.destroy_image(frame, self.ui_image);

                self.tile_resolution_x = width.div_ceil(DRAW_2D_TILE_SIZE);
                self.tile_resolution_y = height.div_ceil(DRAW_2D_TILE_SIZE);

                self.depth_image = gpu.create_image(&ImageDesc {
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

                gpu.debug_name_image(self.depth_image, "depth");

                self.color_image = gpu.create_image(&ImageDesc {
                    memory_location: MemoryLocation::Device,
                    host_mapped: false,
                    usage: ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::STORAGE,
                    dimension: ImageDimension::Type2d,
                    format: ImageFormat::RGBA16_FLOAT,
                    tiling: ImageTiling::Optimal,
                    width,
                    height,
                    depth: 1,
                    layer_count: 1,
                    mip_levels: 1,
                });

                gpu.debug_name_image(self.color_image, "render target");

                self.ui_image = gpu.create_image(&ImageDesc {
                    memory_location: MemoryLocation::Device,
                    host_mapped: false,
                    usage: ImageUsageFlags::STORAGE,
                    dimension: ImageDimension::Type2d,
                    format: ImageFormat::RGBA16_FLOAT,
                    tiling: ImageTiling::Optimal,
                    width,
                    height,
                    depth: 1,
                    layer_count: 1,
                    mip_levels: 1,
                });

                gpu.debug_name_image(self.ui_image, "ui");

                gpu.cmd_barrier(
                    cmd_encoder,
                    None,
                    &[ImageBarrier::optimal_aspect(
                        &Access::None,
                        &Access::DepthStencilAttachmentWrite,
                        self.depth_image,
                        ImageAspectFlags::DEPTH,
                    )],
                );

                self.width = width;
                self.height = height;
            }

            let (touched_glyphs, glyph_texture) = ui_state.glyph_cache.update_atlas();

            // If the atlas has been updated, we need to upload it to the GPU.
            if let Some(texture) = glyph_texture {
                gpu.cmd_begin_debug_marker(
                    cmd_encoder,
                    "upload glyph atlas",
                    microshades::BROWN_RGBA_F32[3],
                );

                let buffer = gpu.request_transient_buffer_with_data(
                    frame,
                    thread_token,
                    BufferUsageFlags::TRANSFER,
                    texture,
                );

                self.glyph_atlas_image_index = self.glyph_atlas_image_index.wrapping_add(1);

                gpu.cmd_barrier(
                    cmd_encoder,
                    None,
                    &[ImageBarrier::optimal_discard(
                        &Access::ShaderSampledImageRead,
                        &Access::TransferWrite,
                        self.glyph_atlas_images[self.glyph_atlas_image_index & 1],
                    )],
                );

                gpu.cmd_copy_buffer_to_image(
                    cmd_encoder,
                    buffer.to_arg(),
                    self.glyph_atlas_images[self.glyph_atlas_image_index & 1],
                    ImageLayout::Optimal,
                    &[BufferImageCopy {
                        image_extent: Extent3d {
                            width: atlas_width,
                            height: atlas_height,
                            depth: 1,
                        },
                        ..default()
                    }],
                );

                gpu.cmd_barrier(
                    cmd_encoder,
                    None,
                    &[ImageBarrier::optimal(
                        &Access::TransferWrite,
                        &Access::ShaderSampledImageRead,
                        self.glyph_atlas_images[self.glyph_atlas_image_index & 1],
                    )],
                );

                gpu.cmd_end_debug_marker(cmd_encoder);
            }

            gpu.cmd_barrier(
                cmd_encoder,
                None,
                &[
                    ImageBarrier::optimal_discard(
                        &Access::ShaderOtherRead,
                        &Access::ColorAttachmentWrite,
                        self.color_image,
                    ),
                    ImageBarrier::general_discard(
                        &Access::ShaderOtherRead,
                        &Access::ComputeWrite,
                        self.ui_image,
                    ),
                ],
            );

            gpu.cmd_begin_debug_marker(cmd_encoder, "sharks", microshades::BLUE_RGBA_F32[3]);

            gpu.cmd_begin_rendering(
                cmd_encoder,
                &RenderingDesc {
                    x: 0,
                    y: 0,
                    width,
                    height,
                    color_attachments: &[RenderingAttachment {
                        image: self.color_image,
                        load_op: LoadOp::Clear(ClearValue::ColorF32([1.0, 1.0, 1.0, 1.0])),
                        store_op: StoreOp::Store,
                    }],
                    depth_attachment: Some(RenderingAttachment {
                        image: self.depth_image,
                        load_op: LoadOp::Clear(ClearValue::DepthStencil {
                            depth: 0.0,
                            stencil: 0,
                        }),
                        store_op: StoreOp::DontCare,
                    }),
                    stencil_attachment: None,
                },
            );

            gpu.cmd_set_scissors(
                cmd_encoder,
                &[Scissor {
                    offset: Offset2d { x: 0, y: 0 },
                    extent: Extent2d { width, height },
                }],
            );

            gpu.cmd_set_viewports(
                cmd_encoder,
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
            {
                let model = &self.models[ModelRes::Shark];
                let image = self.images[ImageRes::Shark];

                let instance_count = self.transforms.len() as u32;

                let transform_buffer = gpu.request_transient_buffer_with_data(
                    frame,
                    thread_token,
                    BufferUsageFlags::STORAGE,
                    self.transforms.as_slice(),
                );

                // We're done with you now!
                self.transforms.clear();

                let graphics_bind_group = gpu.request_transient_bind_group(
                    frame,
                    thread_token,
                    self.pipelines.graphics_bind_group_layout,
                    &[Bind {
                        binding: GraphicsBinds::Albedo as u32,
                        array_element: 0,
                        typed: TypedBind::SampledImage(&[(ImageLayout::Optimal, image)]),
                    }],
                );

                gpu.cmd_set_pipeline(cmd_encoder, self.pipelines.basic_pipeline);
                gpu.cmd_set_bind_group(cmd_encoder, 0, &graphics_bind_group);
                gpu.cmd_push_constants_with_data(
                    cmd_encoder,
                    ShaderStageFlags::VERTEX,
                    0,
                    &BasicConstants {
                        clip_from_model,
                        vertex_buffer_address: gpu.get_buffer_address(model.vertex_buffer.to_arg()),
                        transform_buffer_address: gpu.get_buffer_address(transform_buffer.to_arg()),
                    },
                );

                gpu.cmd_set_index_buffer(
                    cmd_encoder,
                    model.index_buffer.to_arg(),
                    0,
                    IndexType::U16,
                );

                gpu.cmd_draw_indexed(cmd_encoder, model.indices, instance_count, 0, 0, 0);
            }

            gpu.cmd_end_rendering(cmd_encoder);

            gpu.cmd_end_debug_marker(cmd_encoder);

            gpu.cmd_compute_touch_swapchain(cmd_encoder, swapchain_image);

            let compute_bind_group = gpu.request_transient_bind_group(
                frame,
                thread_token,
                self.pipelines.compute_bind_group_layout,
                &[
                    Bind {
                        binding: ComputeBinds::TonyMcMapfaceLut as u32,
                        array_element: 0,
                        typed: TypedBind::SampledImage(&[(
                            ImageLayout::Optimal,
                            self.images[ImageRes::TonyMcMapfaceLut],
                        )]),
                    },
                    Bind {
                        binding: ComputeBinds::GlyphAtlas as u32,
                        array_element: 0,
                        typed: TypedBind::SampledImage(&[(
                            ImageLayout::Optimal,
                            self.glyph_atlas_images[self.glyph_atlas_image_index & 1],
                        )]),
                    },
                    Bind {
                        binding: ComputeBinds::UiRenderTarget as u32,
                        array_element: 0,
                        typed: TypedBind::StorageImage(&[(ImageLayout::General, self.ui_image)]),
                    },
                    Bind {
                        binding: ComputeBinds::ColorRenderTarget as u32,
                        array_element: 0,
                        typed: TypedBind::StorageImage(&[(ImageLayout::General, self.color_image)]),
                    },
                    Bind {
                        binding: ComputeBinds::CompositedRenderTarget as u32,
                        array_element: 0,
                        typed: TypedBind::StorageImage(&[(ImageLayout::General, swapchain_image)]),
                    },
                ],
            );

            let tile_buffer = gpu.request_transient_buffer(
                frame,
                thread_token,
                BufferUsageFlags::STORAGE,
                self.tile_resolution_x as usize
                    * self.tile_resolution_y as usize
                    * std::mem::size_of::<u32>()
                    * 2,
            );
            let tile_buffer_address = gpu.get_buffer_address(tile_buffer.to_arg());

            // Render UI
            {
                gpu.cmd_begin_debug_marker(
                    cmd_encoder,
                    "2d primitives",
                    microshades::PURPLE_RGBA_F32[3],
                );

                let draw_buffer = gpu.request_transient_buffer_with_data(
                    frame,
                    thread_token,
                    BufferUsageFlags::STORAGE,
                    ui_state.draw_cmds.as_slice(),
                );

                let draw_buffer_len = ui_state.draw_cmds.len() as u32;

                let scissor_buffer = gpu.request_transient_buffer_with_data(
                    frame,
                    thread_token,
                    BufferUsageFlags::STORAGE,
                    ui_state.scissors.as_slice(),
                );

                let glyph_buffer = gpu.request_transient_buffer_with_data(
                    frame,
                    thread_token,
                    BufferUsageFlags::STORAGE,
                    touched_glyphs,
                );

                const COARSE_BUFFER_LEN: usize = 1 << 20;
                let coarse_buffer = gpu.request_transient_buffer(
                    frame,
                    thread_token,
                    BufferUsageFlags::STORAGE,
                    COARSE_BUFFER_LEN * std::mem::size_of::<u32>(),
                );

                let indirect_dispatch_buffer = gpu.request_transient_buffer(
                    frame,
                    thread_token,
                    BufferUsageFlags::INDIRECT,
                    3 * std::mem::size_of::<u32>(),
                );

                let finished_buffer = gpu.request_transient_buffer(
                    frame,
                    thread_token,
                    BufferUsageFlags::INDIRECT,
                    std::mem::size_of::<u32>(),
                );

                let tmp_buffer = gpu.request_transient_buffer(
                    frame,
                    thread_token,
                    BufferUsageFlags::STORAGE,
                    COARSE_BUFFER_LEN * std::mem::size_of::<u32>(),
                );

                let spine_buffer = gpu.request_transient_buffer(
                    frame,
                    thread_token,
                    BufferUsageFlags::STORAGE,
                    calculate_spine_size(COARSE_BUFFER_LEN) * std::mem::size_of::<u32>(), // TODO: Fix size
                );

                let draw_buffer_address = gpu.get_buffer_address(draw_buffer.to_arg());
                let scissor_buffer_address = gpu.get_buffer_address(scissor_buffer.to_arg());
                let glyph_buffer_address = gpu.get_buffer_address(glyph_buffer.to_arg());
                let coarse_buffer_address = gpu.get_buffer_address(coarse_buffer.to_arg());
                let indirect_dispatch_buffer_address =
                    gpu.get_buffer_address(indirect_dispatch_buffer.to_arg());
                let finished_buffer_address = gpu.get_buffer_address(finished_buffer.to_arg());
                let tmp_buffer_address = gpu.get_buffer_address(tmp_buffer.to_arg());
                let spine_buffer_address = gpu.get_buffer_address(spine_buffer.to_arg());

                gpu.cmd_set_pipeline(cmd_encoder, self.pipelines.draw_2d_bin_0_clear_pipeline);
                gpu.cmd_set_bind_group(cmd_encoder, 0, &compute_bind_group);
                gpu.cmd_push_constants_with_data(
                    cmd_encoder,
                    ShaderStageFlags::COMPUTE,
                    0,
                    &Draw2dClearConstants {
                        finished_buffer_address,
                        coarse_buffer_address,
                    },
                );
                gpu.cmd_dispatch(cmd_encoder, 1, 1, 1);

                gpu.cmd_barrier(
                    cmd_encoder,
                    Some(&GlobalBarrier {
                        prev_access: &[Access::ComputeWrite],
                        next_access: &[Access::ComputeOtherRead],
                    }),
                    &[],
                );

                gpu.cmd_set_pipeline(cmd_encoder, self.pipelines.draw_2d_bin_1_scatter_pipeline);
                gpu.cmd_set_bind_group(cmd_encoder, 0, &compute_bind_group);
                gpu.cmd_push_constants_with_data(
                    cmd_encoder,
                    ShaderStageFlags::COMPUTE,
                    0,
                    &Draw2dScatterConstants {
                        tile_resolution_x: self.tile_resolution_x,
                        tile_resolution_y: self.tile_resolution_y,
                        draw_buffer_len,
                        coarse_buffer_len: COARSE_BUFFER_LEN as u32,
                        draw_buffer_address,
                        scissor_buffer_address,
                        glyph_buffer_address,
                        coarse_buffer_address,
                    },
                );

                gpu.cmd_dispatch(
                    cmd_encoder,
                    draw_buffer_len
                        .div_ceil(self.pipelines.draw_2d_bin_1_scatter_pipeline_workgroup_size),
                    1,
                    1,
                );

                gpu.cmd_barrier(
                    cmd_encoder,
                    Some(&GlobalBarrier {
                        prev_access: &[Access::ComputeWrite],
                        next_access: &[Access::ComputeOtherRead],
                    }),
                    &[],
                );

                gpu.cmd_set_pipeline(cmd_encoder, self.pipelines.draw_2d_bin_2_sort_pipeline);
                gpu.cmd_set_bind_group(cmd_encoder, 0, &compute_bind_group);
                gpu.cmd_push_constants_with_data(
                    cmd_encoder,
                    ShaderStageFlags::COMPUTE,
                    0,
                    &Draw2dSortConstants {
                        // -1 due to the count taking up a single slot in the buffer.
                        coarse_buffer_len: COARSE_BUFFER_LEN as u32 - 1,
                        _pad: 0,
                        indirect_dispatch_buffer_address,
                        coarse_buffer_address,
                    },
                );
                gpu.cmd_dispatch(cmd_encoder, 1, 1, 1);

                gpu.cmd_barrier(
                    cmd_encoder,
                    Some(&GlobalBarrier {
                        prev_access: &[Access::ComputeWrite],
                        next_access: &[Access::ComputeOtherRead, Access::IndirectBuffer],
                    }),
                    &[],
                );

                gpu.cmd_begin_debug_marker(
                    cmd_encoder,
                    "radix sort",
                    microshades::ORANGE_RGBA_F32[2],
                );

                // First element in the scratch buffer is the count.
                let count_buffer_address = coarse_buffer_address;
                // Then the elements we want to sort follow.
                let mut src_buffer_address = count_buffer_address.byte_add(4);
                let mut dst_buffer_address = tmp_buffer_address;

                for pass in 0..4 {
                    let shift = pass * 8;

                    // Upsweep
                    gpu.cmd_set_pipeline(cmd_encoder, self.pipelines.radix_sort_0_upsweep_pipeline);
                    gpu.cmd_set_bind_group(cmd_encoder, 0, &compute_bind_group);
                    gpu.cmd_push_constants_with_data(
                        cmd_encoder,
                        ShaderStageFlags::COMPUTE,
                        0,
                        &RadixSortUpsweepConstants {
                            shift,
                            _pad: 0,
                            finished_buffer_address,
                            count_buffer_address,
                            src_buffer_address,
                            spine_buffer_address,
                        },
                    );
                    gpu.cmd_dispatch_indirect(cmd_encoder, indirect_dispatch_buffer.to_arg(), 0);

                    gpu.cmd_barrier(
                        cmd_encoder,
                        Some(&GlobalBarrier {
                            prev_access: &[Access::ComputeWrite],
                            next_access: &[Access::ComputeOtherRead],
                        }),
                        &[],
                    );

                    // Downsweep
                    gpu.cmd_set_pipeline(
                        cmd_encoder,
                        self.pipelines.radix_sort_1_downsweep_pipeline,
                    );
                    gpu.cmd_set_bind_group(cmd_encoder, 0, &compute_bind_group);
                    gpu.cmd_push_constants_with_data(
                        cmd_encoder,
                        ShaderStageFlags::COMPUTE,
                        0,
                        &RadixSortDownsweepConstants {
                            shift,
                            _pad: 0,
                            count_buffer_address,
                            src_buffer_address,
                            dst_buffer_address,
                            spine_buffer_address,
                        },
                    );
                    gpu.cmd_dispatch_indirect(cmd_encoder, indirect_dispatch_buffer.to_arg(), 0);

                    gpu.cmd_barrier(
                        cmd_encoder,
                        Some(&GlobalBarrier {
                            prev_access: &[Access::ComputeWrite],
                            next_access: &[Access::ComputeOtherRead],
                        }),
                        &[],
                    );

                    std::mem::swap(&mut src_buffer_address, &mut dst_buffer_address);
                }

                gpu.cmd_end_debug_marker(cmd_encoder);

                gpu.cmd_set_pipeline(cmd_encoder, self.pipelines.draw_2d_bin_3_resolve_pipeline);
                gpu.cmd_set_bind_group(cmd_encoder, 0, &compute_bind_group);
                gpu.cmd_push_constants_with_data(
                    cmd_encoder,
                    ShaderStageFlags::COMPUTE,
                    0,
                    &Draw2dResolveConstants {
                        tile_stride: self.tile_resolution_x,
                        draw_buffer_len,
                        draw_buffer_address,
                        scissor_buffer_address,
                        glyph_buffer_address,
                        coarse_buffer_address,
                        fine_buffer_address: tmp_buffer_address,
                        tile_buffer_address,
                    },
                );
                gpu.cmd_dispatch(
                    cmd_encoder,
                    1,
                    self.tile_resolution_x,
                    self.tile_resolution_y,
                );

                gpu.cmd_barrier(
                    cmd_encoder,
                    Some(&GlobalBarrier {
                        prev_access: &[Access::ComputeWrite],
                        next_access: &[Access::ComputeOtherRead],
                    }),
                    &[],
                );

                gpu.cmd_set_pipeline(cmd_encoder, self.pipelines.draw_2d_rasterize_pipeline);
                gpu.cmd_set_bind_group(cmd_encoder, 0, &compute_bind_group);
                gpu.cmd_push_constants_with_data(
                    cmd_encoder,
                    ShaderStageFlags::COMPUTE,
                    0,
                    &Draw2dRasterizeConstants {
                        tile_stride: self.tile_resolution_x,
                        _pad: 0,
                        draw_buffer_address,
                        scissor_buffer_address,
                        glyph_buffer_address,
                        coarse_buffer_address,
                        fine_buffer_address: tmp_buffer_address,
                        tile_buffer_address,
                    },
                );
                gpu.cmd_dispatch(cmd_encoder, (self.width + 7) / 8, (self.height + 7) / 8, 1);

                gpu.cmd_end_debug_marker(cmd_encoder);
            }

            // Display transform and composite
            {
                gpu.cmd_begin_debug_marker(
                    cmd_encoder,
                    "composite",
                    microshades::GREEN_RGBA_F32[3],
                );

                gpu.cmd_barrier(
                    cmd_encoder,
                    None,
                    &[
                        ImageBarrier {
                            prev_access: &[Access::ColorAttachmentWrite],
                            next_access: &[Access::ShaderOtherRead],
                            prev_layout: ImageLayout::Optimal,
                            next_layout: ImageLayout::General,
                            image: self.color_image,
                            subresource_range: ImageSubresourceRange::default(),
                            discard_contents: false,
                        },
                        ImageBarrier::general(
                            &Access::ComputeWrite,
                            &Access::ComputeOtherRead,
                            self.ui_image,
                        ),
                    ],
                );

                gpu.cmd_set_pipeline(cmd_encoder, self.pipelines.composite_pipeline);
                gpu.cmd_set_bind_group(cmd_encoder, 0, &compute_bind_group);
                gpu.cmd_push_constants_with_data(
                    cmd_encoder,
                    ShaderStageFlags::COMPUTE,
                    0,
                    &CompositeConstants {
                        tile_resolution_x: self.tile_resolution_x,
                        tile_resolution_y: self.tile_resolution_y,
                        tile_buffer_address,
                    },
                );
                gpu.cmd_dispatch(cmd_encoder, (self.width + 7) / 8, (self.height + 7) / 8, 1);

                gpu.cmd_end_debug_marker(cmd_encoder);
            }
        }
        gpu.submit(frame, cmd_encoder);
    }
}
