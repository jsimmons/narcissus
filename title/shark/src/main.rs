use std::fmt::Write;
use std::ops::Index;
use std::path::Path;
use std::time::{Duration, Instant};

use narcissus_core::{dds, Widen as _};
use pipelines::basic::BasicPipeline;
use pipelines::{GlyphInstance, PrimitiveUniforms, TILE_SIZE, TILE_STRIDE};
use renderdoc_sys as rdoc;

use fonts::{FontFamily, Fonts};
use helpers::load_obj;
use narcissus_app::{create_app, Event, Key, PressedState, WindowDesc};
use narcissus_core::{box_assume_init, default, rand::Pcg64, zeroed_box, BitIter};
use narcissus_font::{FontCollection, GlyphCache, HorizontalMetrics};
use narcissus_gpu::{
    create_device, Access, Bind, BindDesc, BindGroupLayout, BindingType, Buffer, BufferDesc,
    BufferImageCopy, BufferUsageFlags, ClearValue, CmdEncoder, ColorSpace, ComputePipelineDesc,
    Device, DeviceExt, Extent2d, Extent3d, Frame, GlobalBarrier, Image, ImageAspectFlags,
    ImageBarrier, ImageDesc, ImageDimension, ImageFormat, ImageLayout, ImageSubresourceRange,
    ImageTiling, ImageUsageFlags, IndexType, LoadOp, MemoryLocation, Offset2d, PersistentBuffer,
    Pipeline, PipelineLayout, PresentMode, PushConstantRange, RenderingAttachment, RenderingDesc,
    Sampler, SamplerAddressMode, SamplerDesc, SamplerFilter, Scissor, ShaderDesc, ShaderStageFlags,
    StoreOp, SwapchainConfigurator, SwapchainImage, ThreadToken, TypedBind, Viewport,
};
use narcissus_image as image;
use narcissus_maths::{
    clamp, perlin_noise3, sin_cos_pi_f32, sin_pi_f32, vec3, Affine3, Deg, HalfTurn, Mat3, Mat4,
    Point3, Vec3,
};
use spring::simple_spring_damper_exact;

use crate::pipelines::basic::BasicUniforms;

mod fonts;
mod helpers;
pub mod microshades;
mod pipelines;
mod spring;

const SQRT_2: f32 = 0.70710677;
const GLYPH_CACHE_SIZE: usize = 1024;
const ARCHTYPE_PROJECTILE_MAX: usize = 65536;

struct GameVariables {
    game_speed: f32,

    camera_distance: f32,
    camera_angle: Deg,
    camera_damping: f32,
    camera_deadzone: f32,
    camera_shake_decay: f32,
    camera_shake_max_offset: f32,
    camera_shake_frequency: f32,

    player_speed: f32,

    weapon_cooldown: f32,
    weapon_projectile_speed: f32,
    weapon_projectile_lifetime: f32,
}

static GAME_VARIABLES: GameVariables = GameVariables {
    game_speed: 1.0,

    camera_distance: 45.0,
    camera_angle: Deg::new(60.0),
    camera_damping: 35.0,
    camera_deadzone: 0.1,
    camera_shake_decay: 2.0,
    camera_shake_max_offset: 2.0,
    camera_shake_frequency: 11.0,

    player_speed: 10.0,

    weapon_cooldown: 0.0,
    weapon_projectile_speed: 20.0,
    weapon_projectile_lifetime: 6.0,
};

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Left,
    Right,
    Up,
    Down,
    Damage,
}

pub struct ActionEvent {
    action: Action,
    value: f32,
}

pub struct Actions {
    old_values: [f32; Self::MAX_ACTIONS],
    new_values: [f32; Self::MAX_ACTIONS],
}

impl Actions {
    const MAX_ACTIONS: usize = 256;

    fn new() -> Self {
        Self {
            old_values: [0.0; Self::MAX_ACTIONS],
            new_values: [0.0; Self::MAX_ACTIONS],
        }
    }

    fn is_active(&self, action: Action) -> bool {
        self.new_values[action as usize] != 0.0
    }

    pub fn became_active_this_frame(&self, action: Action) -> bool {
        self.new_values[action as usize] != 0.0 && self.old_values[action as usize] == 0.0
    }

    pub fn became_inactive_this_frame(&self, action: Action) -> bool {
        self.new_values[action as usize] == 0.0 && self.old_values[action as usize] != 0.0
    }

    pub fn tick(&mut self, action_queue: &[ActionEvent]) {
        self.old_values = self.new_values;

        for event in action_queue {
            self.new_values[event.action as usize] = event.value;
        }
    }
}

struct PlayerState {
    position: Point3,
    heading: Vec3,

    weapon_cooldown: f32,
}

impl PlayerState {
    fn new() -> Self {
        Self {
            position: Point3::ZERO,
            heading: vec3(SQRT_2, 0.0, -SQRT_2),

            weapon_cooldown: GAME_VARIABLES.weapon_cooldown,
        }
    }
}

struct CameraState {
    eye_offset: Vec3,

    shake: f32,
    shake_offset: Vec3,

    position: Point3,
    velocity: Vec3,
}

impl CameraState {
    fn new() -> Self {
        let theta = HalfTurn::from(GAME_VARIABLES.camera_angle).as_f32();
        let hypotenuse = GAME_VARIABLES.camera_distance;
        let height = sin_pi_f32(theta) * hypotenuse;
        let base = (hypotenuse * hypotenuse - height * height).sqrt();

        // Rotate camera
        let one_on_sqrt2 = 1.0 / 2.0_f32.sqrt();
        let eye_offset = vec3(-base * one_on_sqrt2, height, -base * one_on_sqrt2);

        Self {
            eye_offset,

            shake: 0.0,
            shake_offset: Vec3::ZERO,

            position: Point3::ZERO,
            velocity: Vec3::ZERO,
        }
    }

    fn tick(&mut self, target: Point3, time: f32, delta_time: f32) {
        if Point3::distance_sq(self.position, target)
            > (GAME_VARIABLES.camera_deadzone * GAME_VARIABLES.camera_deadzone)
        {
            let (pos_x, vel_x) = simple_spring_damper_exact(
                self.position.x,
                self.velocity.x,
                target.x,
                GAME_VARIABLES.camera_damping,
                delta_time,
            );
            let (pos_z, vel_z) = simple_spring_damper_exact(
                self.position.z,
                self.velocity.z,
                target.z,
                GAME_VARIABLES.camera_damping,
                delta_time,
            );

            self.position.z = pos_z;
            self.position.x = pos_x;
            self.velocity.x = vel_x;
            self.velocity.z = vel_z;
        }

        self.shake -= GAME_VARIABLES.camera_shake_decay * delta_time;
        self.shake = clamp(self.shake, 0.0, 1.0);

        let t = time * GAME_VARIABLES.camera_shake_frequency;
        let shake = GAME_VARIABLES.camera_shake_max_offset * self.shake * self.shake * self.shake;

        self.shake_offset.x = shake * perlin_noise3(0.0, t, 0.0);
        self.shake_offset.z = shake * perlin_noise3(1.0, t, 0.0);
    }

    fn camera_from_model(&self) -> Mat4 {
        let position = self.position + self.shake_offset;
        let eye = position + self.eye_offset;
        Mat4::look_at(eye, position, Vec3::Y)
    }
}

#[derive(Clone, Copy, Default)]
#[repr(align(16))]
struct ArchetypeProjectileBlock {
    position_x: [f32; Self::WIDTH],
    position_z: [f32; Self::WIDTH],
    velocity_x: [f32; Self::WIDTH],
    velocity_z: [f32; Self::WIDTH],
    lifetime: [f32; Self::WIDTH],
}

impl ArchetypeProjectileBlock {
    const WIDTH: usize = 8;
}

#[derive(Default)]
struct ArchetypeProjectileChunk {
    bitmap: [u8; Self::WIDTH],
    blocks: [ArchetypeProjectileBlock; Self::WIDTH],
}

impl ArchetypeProjectileChunk {
    const WIDTH: usize = 8;
    const LEN: usize = Self::WIDTH * ArchetypeProjectileBlock::WIDTH;
}

struct ArchetypeProjectile {
    bitmap_non_empty: [u64; ARCHTYPE_PROJECTILE_MAX / ArchetypeProjectileChunk::LEN / 64],
    bitmap_non_full: [u64; ARCHTYPE_PROJECTILE_MAX / ArchetypeProjectileChunk::LEN / 64],
    chunks: [ArchetypeProjectileChunk; ARCHTYPE_PROJECTILE_MAX / ArchetypeProjectileChunk::LEN],
}

struct GameState {
    rng: Pcg64,

    time: f32,
    actions: Actions,
    camera: CameraState,
    player: PlayerState,

    archetype_projectile: Box<ArchetypeProjectile>,
}

impl GameState {
    fn new() -> Self {
        let mut archetype_projectile: Box<ArchetypeProjectile> =
            unsafe { box_assume_init(zeroed_box()) };
        archetype_projectile.bitmap_non_full.fill(u64::MAX);
        Self {
            rng: Pcg64::new(),
            time: 0.0,
            actions: Actions::new(),
            camera: CameraState::new(),
            player: PlayerState::new(),
            archetype_projectile,
        }
    }

    fn tick(&mut self, delta_time: f32, action_queue: &[ActionEvent]) {
        let delta_time = delta_time * GAME_VARIABLES.game_speed;
        self.time += delta_time;

        self.actions.tick(action_queue);

        if self.actions.became_active_this_frame(Action::Damage) {
            self.camera.shake += 0.4;
        }

        let movement_bitmap = self.actions.is_active(Action::Up) as usize
            | (self.actions.is_active(Action::Down) as usize) << 1
            | (self.actions.is_active(Action::Left) as usize) << 2
            | (self.actions.is_active(Action::Right) as usize) << 3;

        // Pre-rotated values
        const UP: Vec3 = vec3(SQRT_2, 0.0, SQRT_2);
        const DOWN: Vec3 = vec3(-SQRT_2, 0.0, -SQRT_2);
        const LEFT: Vec3 = vec3(SQRT_2, 0.0, -SQRT_2);
        const RIGHT: Vec3 = vec3(-SQRT_2, 0.0, SQRT_2);
        const UP_LEFT: Vec3 = vec3(1.0, 0.0, 0.0);
        const UP_RIGHT: Vec3 = vec3(0.0, 0.0, 1.0);
        const DOWN_LEFT: Vec3 = vec3(0.0, 0.0, -1.0);
        const DOWN_RIGHT: Vec3 = vec3(-1.0, 0.0, 0.0);

        let movement = [
            // 0 0 0 0
            Vec3::ZERO,
            // 0 0 0 1
            UP,
            // 0 0 1 0
            DOWN,
            // 0 0 1 1
            Vec3::ZERO,
            // 0 1 0 0
            LEFT,
            // 0 1 0 1
            UP_LEFT,
            // 0 1 1 0
            DOWN_LEFT,
            // 0 1 1 1
            LEFT,
            // 1 0 0 0
            RIGHT,
            // 1 0 0 1
            UP_RIGHT,
            // 1 0 1 0
            DOWN_RIGHT,
            // 1 0 1 1
            RIGHT,
            // 1 1 0 0
            Vec3::ZERO,
            // 1 1 0 1
            UP,
            // 1 1 1 0
            DOWN,
            // 1 1 1 1
            Vec3::ZERO,
        ][movement_bitmap];

        if movement != Vec3::ZERO {
            self.player.heading = movement;
        }

        let player_velocity = movement * GAME_VARIABLES.player_speed;
        self.player.position += player_velocity * delta_time;

        self.camera
            .tick(self.player.position, self.time, delta_time);

        self.player.weapon_cooldown -= delta_time;

        if self.player.weapon_cooldown <= 0.0 {
            // fire!
            for _ in 0..32 {
                let [x, y] = self.rng.next_uniform_unit_circle_f32();
                let direction = vec3(x, 0.0, y);
                let velocity = player_velocity + direction * GAME_VARIABLES.weapon_projectile_speed;
                self.spawn_projectile(
                    self.player.position,
                    velocity,
                    GAME_VARIABLES.weapon_projectile_lifetime,
                );
            }

            self.player.weapon_cooldown = GAME_VARIABLES.weapon_cooldown;
        }

        for (bitmap_base, (bitmap_non_empty_word, bitmap_non_full_word)) in self
            .archetype_projectile
            .bitmap_non_empty
            .iter_mut()
            .zip(self.archetype_projectile.bitmap_non_full.iter_mut())
            .enumerate()
        {
            for i in BitIter::new(std::iter::once(*bitmap_non_empty_word)) {
                let chunk_index = bitmap_base * 64 + i;
                let chunk = &mut self.archetype_projectile.chunks[chunk_index];

                for (bitmap, block) in chunk.bitmap.iter_mut().zip(chunk.blocks.iter_mut()) {
                    if *bitmap == 0 {
                        continue;
                    }

                    let old_bitmap = *bitmap;

                    for j in 0..8 {
                        if old_bitmap & (1 << j) == 0 {
                            continue;
                        }

                        block.position_x[j] += block.velocity_x[j] * delta_time;
                        block.position_z[j] += block.velocity_z[j] * delta_time;
                        block.lifetime[j] -= delta_time;
                        let projectile_dead = block.lifetime[j] <= 0.0;

                        *bitmap &= !((projectile_dead as u8) << j);
                    }
                }

                let non_empty = chunk.bitmap.iter().any(|&x| x != 0);
                let non_full = chunk.bitmap.iter().any(|&x| x != u8::MAX);

                *bitmap_non_empty_word =
                    (*bitmap_non_empty_word & !(1 << i)) | (non_empty as u64) << i;
                *bitmap_non_full_word =
                    (*bitmap_non_full_word & !(1 << i)) | (non_full as u64) << i;
            }
        }
    }

    fn spawn_projectile(&mut self, position: Point3, velocity: Vec3, lifetime: f32) {
        let projectile = &mut self.archetype_projectile;
        let bitmap_non_full = &mut projectile.bitmap_non_full;
        let bitmap_non_empty = &mut projectile.bitmap_non_empty;

        let chunk_index = BitIter::new(bitmap_non_full.iter().copied())
            .next()
            .unwrap();
        let chunk = &mut projectile.chunks[chunk_index];

        let block_index = chunk
            .bitmap
            .iter()
            .copied()
            .position(|x| x != u8::MAX)
            .unwrap();
        let block = &mut chunk.blocks[block_index];

        let j = BitIter::new(std::iter::once(!chunk.bitmap[block_index]))
            .next()
            .unwrap();

        block.position_x[j] = position.x;
        block.position_z[j] = position.z;
        block.velocity_x[j] = velocity.x;
        block.velocity_z[j] = velocity.z;
        block.lifetime[j] = lifetime;

        chunk.bitmap[block_index] |= 1 << j;
        let block_non_full = chunk.bitmap[block_index] != !0;
        bitmap_non_empty[chunk_index / 64] |= 1 << (chunk_index % 64);
        bitmap_non_full[chunk_index / 64] = (bitmap_non_full[chunk_index / 64]
            & !(1 << (chunk_index % 64)))
            | (block_non_full as u64) << (chunk_index % 64);
    }
}

struct UiState<'a> {
    scale: f32,
    fonts: &'a Fonts<'a>,
    glyph_cache: GlyphCache<'a, Fonts<'a>>,

    tmp_string: String,

    primitive_instances: Vec<GlyphInstance>,
}

impl<'a> UiState<'a> {
    fn new(fonts: &'a Fonts<'a>, scale: f32) -> Self {
        let glyph_cache = GlyphCache::new(fonts, GLYPH_CACHE_SIZE, GLYPH_CACHE_SIZE, 1);

        Self {
            scale,
            fonts,
            glyph_cache,
            tmp_string: default(),
            primitive_instances: vec![],
        }
    }

    fn text_fmt(
        &mut self,
        mut x: f32,
        y: f32,
        font_family: FontFamily,
        font_size_px: f32,
        args: std::fmt::Arguments,
    ) {
        let font = self.fonts.font(font_family);
        let font_size_px = font_size_px * self.scale;
        let scale = font.scale_for_size_px(font_size_px);

        let mut prev_index = None;

        self.tmp_string.clear();
        self.tmp_string.write_fmt(args).unwrap();

        for c in self.tmp_string.chars() {
            let glyph_index = font
                .glyph_index(c)
                .unwrap_or_else(|| font.glyph_index('□').unwrap());

            let touched_glyph_index =
                self.glyph_cache
                    .touch_glyph(font_family, glyph_index, font_size_px);

            let HorizontalMetrics {
                advance_width,
                left_side_bearing: _,
            } = font.horizontal_metrics(glyph_index);

            let advance = if let Some(prev_index) = prev_index {
                font.kerning_advance(prev_index, glyph_index)
            } else {
                0.0
            };
            prev_index = Some(glyph_index);

            x += advance * scale;

            self.primitive_instances.push(GlyphInstance {
                x,
                y,
                touched_glyph_index,
                color: 0x880000ff,
            });

            x += advance_width * scale;
        }
    }
}

enum SamplerRes {
    Bilinear,
}

pub struct Samplers {
    bilinear: Sampler,
}

impl Index<SamplerRes> for Samplers {
    type Output = Sampler;

    fn index(&self, index: SamplerRes) -> &Self::Output {
        match index {
            SamplerRes::Bilinear => &self.bilinear,
        }
    }
}

impl Samplers {
    fn load(gpu: &Gpu) -> Samplers {
        let bilinear = gpu.create_sampler(&SamplerDesc {
            filter: SamplerFilter::Bilinear,
            address_mode: SamplerAddressMode::Clamp,
            compare_op: None,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
        });
        Samplers { bilinear }
    }
}

struct Model<'a> {
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
                &[ImageBarrier::layout_optimal(
                    &[Access::None],
                    &[Access::TransferWrite],
                    image,
                    ImageAspectFlags::COLOR,
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
                &[ImageBarrier::layout_optimal(
                    &[Access::TransferWrite],
                    &[Access::FragmentShaderSampledImageRead],
                    image,
                    ImageAspectFlags::COLOR,
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
                &[ImageBarrier {
                    prev_access: &[Access::None],
                    next_access: &[Access::TransferWrite],
                    prev_layout: ImageLayout::Optimal,
                    next_layout: ImageLayout::Optimal,
                    subresource_range: ImageSubresourceRange::default(),
                    image,
                }],
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
                &[ImageBarrier {
                    prev_access: &[Access::TransferWrite],
                    next_access: &[Access::ShaderSampledImageRead],
                    prev_layout: ImageLayout::Optimal,
                    next_layout: ImageLayout::Optimal,
                    subresource_range: ImageSubresourceRange::default(),
                    image,
                }],
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

type Gpu = dyn Device + 'static;

struct DrawState<'gpu> {
    gpu: &'gpu Gpu,

    basic_pipeline: BasicPipeline,

    compute_bind_group_layout: BindGroupLayout,
    bin_clear_pipeline: Pipeline,
    bin_pipeline: Pipeline,
    rasterize_pipeline: Pipeline,
    display_transform_pipeline: Pipeline,

    width: u32,
    height: u32,

    tile_resolution_x: u32,
    tile_resolution_y: u32,

    depth_image: Image,
    color_image: Image,
    ui_image: Image,

    tiles_buffer: Buffer,

    glyph_atlas_image: Image,

    _samplers: Samplers,
    models: Models<'gpu>,
    images: Images,

    transforms: Vec<Affine3>,
}

impl<'gpu> DrawState<'gpu> {
    fn new(gpu: &'gpu Gpu, thread_token: &ThreadToken) -> Self {
        let samplers = Samplers::load(gpu);
        let immutable_samplers = &[samplers[SamplerRes::Bilinear]];

        let compute_bind_group_layout = gpu.create_bind_group_layout(&[
            // Samplers
            BindDesc::with_immutable_samplers(ShaderStageFlags::COMPUTE, immutable_samplers),
            // Tony mc mapface LUT
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::SampledImage),
            // Glyph Atlas
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::SampledImage),
            // UI Render Target
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
            // Color Render Target
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
            // Composited output
            BindDesc::new(ShaderStageFlags::COMPUTE, BindingType::StorageImage),
        ]);

        let compute_pipeline_layout = PipelineLayout {
            bind_group_layouts: &[compute_bind_group_layout],
            push_constant_ranges: &[PushConstantRange {
                stage_flags: ShaderStageFlags::COMPUTE,
                offset: 0,
                size: std::mem::size_of::<PrimitiveUniforms>() as u32,
            }],
        };

        let bin_clear_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_BIN_CLEAR_COMP_SPV,
            },
            layout: &compute_pipeline_layout,
        });

        let bin_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_BIN_COMP_SPV,
            },
            layout: &compute_pipeline_layout,
        });

        let rasterize_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::PRIMITIVE_2D_RASTERIZE_COMP_SPV,
            },
            layout: &compute_pipeline_layout,
        });

        let display_transform_pipeline = gpu.create_compute_pipeline(&ComputePipelineDesc {
            shader: ShaderDesc {
                entry: c"main",
                code: shark_shaders::DISPLAY_TRANSFORM_COMP_SPV,
            },
            layout: &compute_pipeline_layout,
        });

        let basic_pipeline = BasicPipeline::new(gpu, immutable_samplers);

        let models = Models::load(gpu);
        let images = Images::load(gpu, thread_token);

        Self {
            gpu,
            basic_pipeline,
            compute_bind_group_layout,
            bin_clear_pipeline,
            bin_pipeline,
            rasterize_pipeline,
            display_transform_pipeline,
            width: 0,
            height: 0,
            tile_resolution_x: 0,
            tile_resolution_y: 0,
            depth_image: default(),
            color_image: default(),
            ui_image: default(),
            tiles_buffer: default(),
            glyph_atlas_image: default(),
            _samplers: samplers,
            models,
            images,
            transforms: vec![],
        }
    }

    fn draw(
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
        let scale = Mat3::from_scale(Vec3::splat(0.125));

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

            if self.glyph_atlas_image.is_null() {
                self.glyph_atlas_image = gpu.create_image(&ImageDesc {
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

                gpu.debug_name_image(self.glyph_atlas_image, "glyph atlas");

                gpu.cmd_barrier(
                    cmd_encoder,
                    None,
                    &[ImageBarrier::layout_optimal(
                        &[Access::None],
                        &[Access::FragmentShaderSampledImageRead],
                        self.glyph_atlas_image,
                        ImageAspectFlags::COLOR,
                    )],
                );
            }

            if width != self.width || height != self.height {
                gpu.destroy_image(frame, self.depth_image);
                gpu.destroy_image(frame, self.color_image);
                gpu.destroy_image(frame, self.ui_image);

                let tile_resolution_x = (width + (TILE_SIZE - 1)) / TILE_SIZE;
                let tile_resolution_y = (height + (TILE_SIZE - 1)) / TILE_SIZE;

                if tile_resolution_x != self.tile_resolution_x
                    || tile_resolution_y != self.tile_resolution_y
                {
                    gpu.destroy_buffer(frame, self.tiles_buffer);

                    let bitmap_buffer_size = tile_resolution_x
                        * tile_resolution_y
                        * TILE_STRIDE
                        * std::mem::size_of::<u32>() as u32;

                    self.tiles_buffer = gpu.create_buffer(&BufferDesc {
                        memory_location: MemoryLocation::Device,
                        host_mapped: false,
                        usage: BufferUsageFlags::STORAGE,
                        size: bitmap_buffer_size.widen(),
                    });

                    gpu.debug_name_buffer(self.tiles_buffer.to_arg(), "tile bitmap");

                    println!("tile_resolution: ({tile_resolution_x},{tile_resolution_y})");

                    self.tile_resolution_x = tile_resolution_x;
                    self.tile_resolution_y = tile_resolution_y;
                }

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
                    &[ImageBarrier::layout_optimal(
                        &[Access::None],
                        &[Access::DepthStencilAttachmentWrite],
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

                gpu.cmd_barrier(
                    cmd_encoder,
                    None,
                    &[ImageBarrier::layout_optimal(
                        &[Access::ShaderSampledImageRead],
                        &[Access::TransferWrite],
                        self.glyph_atlas_image,
                        ImageAspectFlags::COLOR,
                    )],
                );

                gpu.cmd_copy_buffer_to_image(
                    cmd_encoder,
                    buffer.to_arg(),
                    self.glyph_atlas_image,
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
                    &[ImageBarrier::layout_optimal(
                        &[Access::TransferWrite],
                        &[Access::ShaderSampledImageRead],
                        self.glyph_atlas_image,
                        ImageAspectFlags::COLOR,
                    )],
                );

                gpu.cmd_end_debug_marker(cmd_encoder);
            }

            gpu.cmd_barrier(
                cmd_encoder,
                None,
                &[
                    ImageBarrier::layout_optimal(
                        &[Access::None],
                        &[Access::ColorAttachmentWrite],
                        self.color_image,
                        ImageAspectFlags::COLOR,
                    ),
                    ImageBarrier {
                        prev_access: &[Access::None],
                        next_access: &[Access::ComputeWrite],
                        prev_layout: ImageLayout::Optimal,
                        next_layout: ImageLayout::General,
                        image: self.ui_image,
                        subresource_range: default(),
                    },
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
                gpu.cmd_set_pipeline(cmd_encoder, self.basic_pipeline.pipeline);

                let basic_uniforms = BasicUniforms { clip_from_model };

                let uniform_buffer = gpu.request_transient_buffer_with_data(
                    frame,
                    thread_token,
                    BufferUsageFlags::UNIFORM,
                    &basic_uniforms,
                );

                gpu.cmd_set_bind_group(
                    frame,
                    cmd_encoder,
                    self.basic_pipeline.uniforms_bind_group_layout,
                    0,
                    &[Bind {
                        binding: 0,
                        array_element: 0,
                        typed: TypedBind::UniformBuffer(&[uniform_buffer.to_arg()]),
                    }],
                );

                {
                    let model = &self.models[ModelRes::Shark];
                    let image = self.images[ImageRes::Shark];

                    let transform_buffer = gpu.request_transient_buffer_with_data(
                        frame,
                        thread_token,
                        BufferUsageFlags::STORAGE,
                        self.transforms.as_slice(),
                    );

                    gpu.cmd_set_bind_group(
                        frame,
                        cmd_encoder,
                        self.basic_pipeline.storage_bind_group_layout,
                        1,
                        &[
                            Bind {
                                binding: 0,
                                array_element: 0,
                                typed: TypedBind::StorageBuffer(&[model.vertex_buffer.to_arg()]),
                            },
                            Bind {
                                binding: 1,
                                array_element: 0,
                                typed: TypedBind::StorageBuffer(&[transform_buffer.to_arg()]),
                            },
                            Bind {
                                binding: 2,
                                array_element: 0,
                                typed: TypedBind::SampledImage(&[(ImageLayout::Optimal, image)]),
                            },
                        ],
                    );

                    gpu.cmd_set_index_buffer(
                        cmd_encoder,
                        model.index_buffer.to_arg(),
                        0,
                        IndexType::U16,
                    );

                    gpu.cmd_draw_indexed(
                        cmd_encoder,
                        model.indices,
                        self.transforms.len() as u32,
                        0,
                        0,
                        0,
                    );
                }

                // We're done with you now!
                self.transforms.clear();
            }

            gpu.cmd_end_rendering(cmd_encoder);

            gpu.cmd_end_debug_marker(cmd_encoder);

            // Render UI
            {
                gpu.cmd_begin_debug_marker(
                    cmd_encoder,
                    "2d primitives",
                    microshades::PURPLE_RGBA_F32[3],
                );

                let glyph_buffer = gpu.request_transient_buffer_with_data(
                    frame,
                    thread_token,
                    BufferUsageFlags::STORAGE,
                    touched_glyphs,
                );
                let glyph_instance_buffer = gpu.request_transient_buffer_with_data(
                    frame,
                    thread_token,
                    BufferUsageFlags::STORAGE,
                    ui_state.primitive_instances.as_slice(),
                );

                let num_primitives = ui_state.primitive_instances.len() as u32;
                let num_primitives_32 = (num_primitives + 31) / 32;
                let num_primitives_1024 = (num_primitives_32 + 31) / 32;

                ui_state.primitive_instances.clear();

                gpu.cmd_set_pipeline(cmd_encoder, self.bin_clear_pipeline);

                gpu.cmd_set_bind_group(
                    frame,
                    cmd_encoder,
                    self.compute_bind_group_layout,
                    0,
                    &[
                        Bind {
                            binding: 1,
                            array_element: 0,
                            typed: TypedBind::SampledImage(&[(
                                ImageLayout::Optimal,
                                self.images[ImageRes::TonyMcMapfaceLut],
                            )]),
                        },
                        Bind {
                            binding: 2,
                            array_element: 0,
                            typed: TypedBind::SampledImage(&[(
                                ImageLayout::Optimal,
                                self.glyph_atlas_image,
                            )]),
                        },
                        Bind {
                            binding: 3,
                            array_element: 0,
                            typed: TypedBind::StorageImage(&[(
                                ImageLayout::General,
                                self.ui_image,
                            )]),
                        },
                        Bind {
                            binding: 4,
                            array_element: 0,
                            typed: TypedBind::StorageImage(&[(
                                ImageLayout::General,
                                self.color_image,
                            )]),
                        },
                        Bind {
                            binding: 5,
                            array_element: 0,
                            typed: TypedBind::StorageImage(&[(
                                ImageLayout::General,
                                swapchain_image,
                            )]),
                        },
                    ],
                );

                gpu.cmd_push_constants(
                    cmd_encoder,
                    ShaderStageFlags::COMPUTE,
                    0,
                    &PrimitiveUniforms {
                        screen_resolution_x: self.width,
                        screen_resolution_y: self.height,
                        atlas_resolution_x: atlas_width,
                        atlas_resolution_y: atlas_height,
                        num_primitives,
                        num_primitives_32,
                        num_primitives_1024,
                        tile_resolution_x: self.tile_resolution_x,
                        tile_resolution_y: self.tile_resolution_y,
                        tile_stride: self.tile_resolution_x,
                        glyphs_buffer: gpu.get_buffer_address(glyph_buffer.to_arg()),
                        glyph_instances_buffer: gpu
                            .get_buffer_address(glyph_instance_buffer.to_arg()),
                        tiles_buffer: gpu.get_buffer_address(self.tiles_buffer.to_arg()),
                    },
                );

                gpu.cmd_dispatch(
                    cmd_encoder,
                    (self.tile_resolution_y * self.tile_resolution_x + 63) / 64,
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

                gpu.cmd_set_pipeline(cmd_encoder, self.bin_pipeline);

                gpu.cmd_dispatch(
                    cmd_encoder,
                    (num_primitives + 2047) / 2048,
                    (self.tile_resolution_x + 3) / 4,
                    (self.tile_resolution_y + 3) / 4,
                );

                gpu.cmd_barrier(
                    cmd_encoder,
                    Some(&GlobalBarrier {
                        prev_access: &[Access::ComputeWrite],
                        next_access: &[Access::ComputeOtherRead],
                    }),
                    &[],
                );

                gpu.cmd_set_pipeline(cmd_encoder, self.rasterize_pipeline);

                gpu.cmd_dispatch(cmd_encoder, (self.width + 7) / 8, (self.height + 7) / 8, 1);

                gpu.cmd_end_debug_marker(cmd_encoder);
            }

            // Display transform and composite
            {
                gpu.cmd_begin_debug_marker(
                    cmd_encoder,
                    "display transform",
                    microshades::GREEN_RGBA_F32[3],
                );

                gpu.cmd_barrier(
                    cmd_encoder,
                    None,
                    &[
                        ImageBarrier {
                            prev_access: &[Access::ColorAttachmentWrite],
                            prev_layout: ImageLayout::Optimal,
                            next_access: &[Access::ShaderOtherRead],
                            next_layout: ImageLayout::General,
                            image: self.color_image,
                            subresource_range: ImageSubresourceRange::default(),
                        },
                        ImageBarrier {
                            prev_access: &[Access::ComputeWrite],
                            prev_layout: ImageLayout::General,
                            next_access: &[Access::ComputeOtherRead],
                            next_layout: ImageLayout::General,
                            image: self.ui_image,
                            subresource_range: ImageSubresourceRange::default(),
                        },
                    ],
                );

                gpu.cmd_compute_touch_swapchain(cmd_encoder, swapchain_image);

                gpu.cmd_set_pipeline(cmd_encoder, self.display_transform_pipeline);

                gpu.cmd_dispatch(cmd_encoder, (self.width + 7) / 8, (self.height + 7) / 8, 1);

                gpu.cmd_end_debug_marker(cmd_encoder);
            }
        }
        gpu.submit(frame, cmd_encoder);
    }
}

pub fn main() {
    #[cfg(debug_assertions)]
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1")
    }

    let renderdoc = rdoc::RenderdocApi1_5_0::load();

    // Default to wayland because otherwise HiDPI is totally borked.
    // Unless renderdoc is attached, in which case wayland would break capture.
    if renderdoc.is_none() && std::env::var("SDL_VIDEODRIVER").is_err() {
        std::env::set_var("SDL_VIDEODRIVER", "wayland")
    }

    let app = create_app();
    let gpu = create_device(narcissus_gpu::DeviceBackend::Vulkan);

    let window = app.create_window(&WindowDesc {
        title: "shark",
        width: 800,
        height: 600,
    });

    let scale = 1.0;

    let thread_token = ThreadToken::new();
    let thread_token = &thread_token;

    let mut action_queue = Vec::new();

    let fonts = Fonts::new();
    let mut ui_state = UiState::new(&fonts, scale);
    let mut game_state = GameState::new();
    let mut draw_state = DrawState::new(gpu.as_ref(), thread_token);

    let target_hz = 120.0;
    let target_dt = Duration::from_secs_f64(1.0 / target_hz);

    let mut last_frame = Instant::now();
    let mut tick_accumulator = target_dt;

    struct Configurator();
    impl SwapchainConfigurator for Configurator {
        fn choose_present_mode(&mut self, _available_present_modes: &[PresentMode]) -> PresentMode {
            PresentMode::Fifo
        }

        fn choose_surface_format(
            &mut self,
            _available_usage_flags: ImageUsageFlags,
            available_surface_formats: &[(ImageFormat, ColorSpace)],
        ) -> (ImageUsageFlags, (ImageFormat, ColorSpace)) {
            let image_usage_flags = ImageUsageFlags::STORAGE;

            if let Some(&swapchain_format) =
                available_surface_formats
                    .iter()
                    .find(|(image_format, _color_space)| {
                        image_format == &ImageFormat::A2R10G10B10_UNORM
                    })
            {
                return (image_usage_flags, swapchain_format);
            }

            if let Some(&swapchain_format) = available_surface_formats
                .iter()
                .find(|(image_format, _color_space)| image_format == &ImageFormat::BGRA8_UNORM)
            {
                return (image_usage_flags, swapchain_format);
            }

            // Default
            (
                image_usage_flags,
                (ImageFormat::RGBA8_UNORM, ColorSpace::Srgb),
            )
        }
    }
    let mut swapchain_configurator = Configurator();

    'main: loop {
        let frame = gpu.begin_frame();
        {
            let frame = &frame;

            let SwapchainImage {
                width,
                height,
                image: swapchain_image,
            } = loop {
                let (_width, height) = window.extent();
                let (drawable_width, drawable_height) = window.drawable_extent();

                ui_state.scale = drawable_height as f32 / height as f32;

                if let Ok(result) = gpu.acquire_swapchain(
                    frame,
                    window.upcast(),
                    drawable_width,
                    drawable_height,
                    &mut swapchain_configurator,
                ) {
                    break result;
                }
            };

            let tick_start = Instant::now();
            'tick: loop {
                'poll_events: while let Some(event) = app.poll_event() {
                    use Event::*;
                    match event {
                        KeyPress {
                            window_id: _,
                            key,
                            repeat,
                            pressed,
                            modifiers: _,
                        } => {
                            if repeat {
                                continue 'poll_events;
                            }

                            let action = match key {
                                Key::Left | Key::A => Some(Action::Left),
                                Key::Right | Key::D => Some(Action::Right),
                                Key::Up | Key::W => Some(Action::Up),
                                Key::Down | Key::S => Some(Action::Down),
                                Key::Space => Some(Action::Damage),
                                Key::Escape => break 'main,
                                _ => None,
                            };

                            let value = match pressed {
                                PressedState::Released => 0.0,
                                PressedState::Pressed => 1.0,
                            };

                            if let Some(action) = action {
                                action_queue.push(ActionEvent { action, value })
                            }
                        }
                        Quit => {
                            break 'main;
                        }
                        Close { window_id } => {
                            let window = app.window(window_id);
                            gpu.destroy_swapchain(window.upcast());
                        }
                        _ => {}
                    }
                }

                if tick_accumulator < target_dt {
                    break 'tick;
                }

                game_state.tick(target_dt.as_secs_f32(), &action_queue);

                action_queue.clear();

                tick_accumulator -= target_dt;
            }

            let tick_duration = Instant::now() - tick_start;

            let (base_x, base_y) = sin_cos_pi_f32(game_state.time);
            let base_x = (base_x + 1.0) * 0.5;
            let base_y = (base_y + 1.0) * 0.5;

            for i in 0..80 {
                let i = i as f32;
                ui_state.text_fmt(
                    base_x * 100.0 * scale + 5.0,
                    base_y * 100.0 * scale + i * 15.0 * scale,
                    FontFamily::RobotoRegular,
                    20.0,
                    format_args!("tick: {:?}", tick_duration),
                );
            }

            for i in 0..224 {
                let i = i as f32;
                ui_state.text_fmt(
                        5.0,
                        8.0 + i * 8.0,
                        FontFamily::NotoSansJapanese,
                        8.0,
                        format_args!(
                            "お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog.  ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog.  ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog.  ████████"
                        ),
                    );
            }

            draw_state.draw(
                thread_token,
                frame,
                &mut ui_state,
                &game_state,
                width,
                height,
                swapchain_image,
            );
        }

        gpu.end_frame(frame);

        let now = Instant::now();
        tick_accumulator += now - last_frame;
        last_frame = now;
    }
}
