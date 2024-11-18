use std::fmt::Write;
use std::ops::Index;
use std::path::Path;
use std::time::{Duration, Instant};

use narcissus_core::{dds, Widen};

use shark_shaders::pipelines::{
    calculate_spine_size, BasicConstants, CompositeConstants, ComputeBinds, Draw2dClearConstants,
    Draw2dCmd, Draw2dRasterizeConstants, Draw2dResolveConstants, Draw2dScatterConstants,
    Draw2dScissor, Draw2dSortConstants, GraphicsBinds, Pipelines, RadixSortDownsweepConstants,
    RadixSortUpsweepConstants, DRAW_2D_TILE_SIZE,
};

use renderdoc_sys as rdoc;

use fonts::{FontFamily, Fonts};
use helpers::load_obj;
use narcissus_app::{create_app, Event, Key, WindowDesc};
use narcissus_core::{box_assume_init, default, rand::Pcg64, zeroed_box, BitIter};
use narcissus_font::{FontCollection, GlyphCache, HorizontalMetrics};
use narcissus_gpu::{
    create_device, Access, Bind, BufferImageCopy, BufferUsageFlags, ClearValue, CmdEncoder,
    ColorSpace, DeviceExt, Extent2d, Extent3d, Frame, GlobalBarrier, Gpu, Image, ImageAspectFlags,
    ImageBarrier, ImageDesc, ImageDimension, ImageFormat, ImageLayout, ImageSubresourceRange,
    ImageTiling, ImageUsageFlags, IndexType, LoadOp, MemoryLocation, Offset2d, PersistentBuffer,
    PresentMode, RenderingAttachment, RenderingDesc, Scissor, ShaderStageFlags, StoreOp,
    SwapchainConfigurator, SwapchainImage, ThreadToken, TypedBind, Viewport,
};
use narcissus_image as image;
use narcissus_maths::{
    clamp, perlin_noise3, sin_cos_pi_f32, sin_pi_f32, vec2, vec3, Affine3, Deg, HalfTurn, Mat3,
    Mat4, Point3, Vec2, Vec3,
};
use spring::simple_spring_damper_exact;

mod fonts;
mod helpers;
pub mod microshades;
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

    weapon_cooldown: 0.2,
    weapon_projectile_speed: 20.0,
    weapon_projectile_lifetime: 3.0,
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
    fonts: &'a Fonts<'a>,
    glyph_cache: GlyphCache<'a, Fonts<'a>>,

    width: f32,
    height: f32,
    scale: f32,

    tmp_string: String,

    scissors: Vec<Draw2dScissor>,
    scissor_stack: Vec<u32>,

    draw_cmds: Vec<Draw2dCmd>,
}

impl<'a> UiState<'a> {
    fn new(fonts: &'a Fonts<'a>) -> Self {
        let glyph_cache = GlyphCache::new(fonts, GLYPH_CACHE_SIZE, GLYPH_CACHE_SIZE, 1);

        Self {
            fonts,
            glyph_cache,
            width: 0.0,
            height: 0.0,
            scale: 1.0,
            tmp_string: default(),
            scissors: vec![],
            scissor_stack: vec![],

            draw_cmds: vec![],
        }
    }

    fn begin_frame(&mut self, width: f32, height: f32, scale: f32) {
        self.width = width;
        self.height = height;
        self.scale = scale;

        self.draw_cmds.clear();

        self.scissor_stack.clear();
        self.scissors.clear();

        // Scissor 0 is always the screen bounds.
        self.scissors.push(Draw2dScissor {
            offset_min: vec2(0.0, 0.0),
            offset_max: vec2(width, height),
        });
    }

    fn push_scissor(
        &mut self,
        mut offset_min: Vec2,
        mut offset_max: Vec2,
        intersect_with_current: bool,
    ) {
        if intersect_with_current {
            let current_scissor_index = self.scissor_stack.last().copied().unwrap_or(0);
            let current_scissor = &self.scissors[current_scissor_index.widen()];
            offset_min = Vec2::max(offset_min, current_scissor.offset_min);
            offset_max = Vec2::min(offset_max, current_scissor.offset_max);
        }

        let scissor_index = self.scissors.len() as u32;
        self.scissors.push(Draw2dScissor {
            offset_min,
            offset_max,
        });
        self.scissor_stack.push(scissor_index);
    }

    fn push_fullscreen_scissor(&mut self) {
        // The fullscreen scissor is always at index 0
        self.scissor_stack.push(0);
    }

    fn pop_scissor(&mut self) {
        // It's invalid to pop more than we've pushed.
        self.scissor_stack.pop().expect("unbalanced push / pop");
    }

    fn rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        border_width: f32,
        border_radii: [f32; 4],
        border_color: u32,
        background_color: u32,
    ) {
        let scissor_index = self.scissor_stack.last().copied().unwrap_or(0);

        let border_width = border_width.clamp(0.0, 255.0).floor() as u8;
        let border_radii = border_radii.map(|radius| radius.clamp(0.0, 255.0).floor() as u8);

        self.draw_cmds.push(Draw2dCmd::rect(
            scissor_index,
            vec2(x, y),
            vec2(width, height),
            border_width,
            border_radii,
            border_color,
            background_color,
        ))
    }

    fn text_fmt(
        &mut self,
        mut x: f32,
        y: f32,
        font_family: FontFamily,
        font_size_px: f32,
        args: std::fmt::Arguments,
    ) -> f32 {
        let scissor_index = self.scissor_stack.last().copied().unwrap_or(0);

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

            self.draw_cmds.push(Draw2dCmd::glyph(
                scissor_index,
                touched_glyph_index,
                microshades::GRAY_RGBA8[4].rotate_right(8),
                vec2(x, y),
            ));

            x += advance_width * scale;
        }

        (font.ascent() - font.descent() + font.line_gap()) * scale
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

struct DrawState<'gpu> {
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
    fn new(gpu: &'gpu Gpu, thread_token: &ThreadToken) -> Self {
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

                self.tile_resolution_x = (width + (DRAW_2D_TILE_SIZE - 1)) / DRAW_2D_TILE_SIZE;
                self.tile_resolution_y = (height + (DRAW_2D_TILE_SIZE - 1)) / DRAW_2D_TILE_SIZE;

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
                gpu.cmd_push_constants(
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
                gpu.cmd_push_constants(
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
                gpu.cmd_push_constants(
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
                    (draw_buffer_len
                        + (self.pipelines.draw_2d_bin_1_scatter_pipeline_workgroup_size - 1))
                        / self.pipelines.draw_2d_bin_1_scatter_pipeline_workgroup_size,
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
                gpu.cmd_push_constants(
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
                    gpu.cmd_push_constants(
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
                    gpu.cmd_push_constants(
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
                gpu.cmd_push_constants(
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
                gpu.cmd_push_constants(
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
                gpu.cmd_push_constants(
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

    let ui_scale_override =
        std::env::var("NARCISSUS_UI_SCALE").map_or(None, |scale| scale.parse::<f32>().ok());

    let app = create_app();
    let gpu = create_device(narcissus_gpu::DeviceBackend::Vulkan);

    let window = app.create_window(&WindowDesc {
        title: "shark",
        width: 800,
        height: 600,
    });

    let thread_token = ThreadToken::new();
    let thread_token = &thread_token;

    let mut action_queue = Vec::new();

    let fonts = Fonts::new();
    let mut ui_state = UiState::new(&fonts);
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

    let mut draw_duration = Duration::ZERO;

    'main: loop {
        let frame = gpu.begin_frame();
        {
            let frame = &frame;

            let SwapchainImage {
                width,
                height,
                image: swapchain_image,
            } = loop {
                let (width, height) = window.size_in_pixels();
                if let Ok(result) = gpu.acquire_swapchain(
                    frame,
                    window.upcast(),
                    width,
                    height,
                    &mut swapchain_configurator,
                ) {
                    break result;
                }
            };

            let mut window_display_scale = window.display_scale();

            let tick_start = Instant::now();
            'tick: loop {
                'poll_events: while let Some(event) = app.poll_event() {
                    use Event::*;
                    match event {
                        KeyPress {
                            window_id: _,
                            key,
                            repeat,
                            down,
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

                            let value = if down { 1.0 } else { 0.0 };

                            if let Some(action) = action {
                                action_queue.push(ActionEvent { action, value })
                            }
                        }
                        ScaleChanged { window_id: _ } => {
                            window_display_scale = window.display_scale()
                        }
                        Quit => {
                            break 'main;
                        }
                        CloseRequested { window_id } => {
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

            let draw_start = Instant::now();
            let tick_duration = draw_start - tick_start;

            ui_state.begin_frame(
                width as f32,
                height as f32,
                ui_scale_override.unwrap_or(window_display_scale),
            );

            {
                let width = width as f32;
                let height = height as f32;

                let (s, c) = sin_cos_pi_f32(game_state.time * 0.1);

                let w = width / 5.0;
                let h = height / 5.0;
                let x = width / 2.0 + w * s;
                let y = height / 2.0 + w * c;

                ui_state.push_scissor(vec2(x - w, y - h), vec2(x + w, y + h), true);
                ui_state.rect(
                    0.0, 0.0, width, height, 0.0, [0.0; 4], 0xffffffff, 0xffffffff,
                );
                ui_state.pop_scissor();

                ui_state.push_scissor(vec2(x - w, y - h), vec2(x + w, y + h), true);

                let mut y = 8.0 * ui_state.scale;
                for i in 0..224 {
                    if i & 1 == 0 {
                        ui_state.push_fullscreen_scissor();
                    }
                    let vertical_advance = ui_state.text_fmt(
                            5.0,
                            y,
                            FontFamily::NotoSansJapanese,
                            16.0 + s * 8.0,
                            format_args!(
                                "お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog.  ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog.  ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog. ████████お握り The Quick Brown Fox Jumped Over The Lazy Dog.  ████████"
                            ),
                        );
                    y += vertical_advance;
                    if i & 1 == 0 {
                        ui_state.pop_scissor();
                    }
                }

                ui_state.pop_scissor();

                for i in 0..500 {
                    let (s, c) = sin_cos_pi_f32(game_state.time * 0.1 + i as f32 * 0.01);

                    let x = width / 2.0 + w * s;
                    let y = height / 2.0 + w * c;
                    ui_state.rect(
                        x - 200.0,
                        y - 200.0,
                        400.0,
                        400.0,
                        100.0,
                        [100.0, 50.0, 25.0, 0.0],
                        0x33333333,
                        microshades::BLUE_RGBA8[4].rotate_right(8),
                    );
                }

                let x = 10.0 * ui_state.scale;
                let mut y = 20.0 * ui_state.scale;
                for i in 0..10 {
                    if i & 1 != 0 {
                        y += ui_state.text_fmt(
                            x,
                            y,
                            FontFamily::RobotoRegular,
                            20.0,
                            format_args!("this tick: {:?}", tick_duration),
                        );
                    } else {
                        y += ui_state.text_fmt(
                            x,
                            y,
                            FontFamily::NotoSansJapanese,
                            20.0,
                            format_args!("last draw: {:?}", draw_duration),
                        );
                    }
                }
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

            draw_duration = Instant::now() - draw_start;
        }

        gpu.end_frame(frame);

        let now = Instant::now();
        tick_accumulator += now - last_frame;
        last_frame = now;
    }
}
