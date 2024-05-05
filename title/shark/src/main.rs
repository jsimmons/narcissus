use std::fmt::Write;

use crate::{
    fonts::{FontFamily, Fonts},
    pipelines::{BasicPipeline, TextPipeline},
};
use helpers::{load_image, load_obj};
use narcissus_app::{create_app, Event, Key, PressedState, WindowDesc};
use narcissus_core::{default, rand::Pcg64, slice::array_windows, BitIter};
use narcissus_font::{FontCollection, GlyphCache, HorizontalMetrics};
use narcissus_gpu::{
    create_device, Access, Bind, BufferImageCopy, BufferUsageFlags, ClearValue, DeviceExt,
    Extent2d, Extent3d, ImageAspectFlags, ImageBarrier, ImageDesc, ImageDimension, ImageFormat,
    ImageLayout, ImageTiling, ImageUsageFlags, IndexType, LoadOp, MemoryLocation, Offset2d,
    Offset3d, RenderingAttachment, RenderingDesc, Scissor, StoreOp, ThreadToken, TypedBind,
    Viewport,
};
use narcissus_maths::{
    clamp, exp_f32, perlin_noise3, sin_pi_f32, vec3, Affine3, Deg, HalfTurn, Mat3, Mat4, Point3,
    Vec3,
};
use pipelines::{BasicUniforms, PrimitiveInstance, PrimitiveVertex, TextUniforms};

mod fonts;
mod helpers;
mod pipelines;

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
    weapon_projectile_speed: 25.0,
    weapon_projectile_lifetime: 10.0,
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
struct ArchetypeProjectile8 {
    position_x: [f32; 8],
    position_z: [f32; 8],
    velocity_x: [f32; 8],
    velocity_z: [f32; 8],
    lifetime: [f32; 8],
}

struct GameState {
    rng: Pcg64,

    time: f32,
    actions: Actions,
    camera: CameraState,
    player: PlayerState,

    archetype_projectile_bitmap_0: [u64; ARCHTYPE_PROJECTILE_MAX / 64 / 64],
    archetype_projectile_bitmap_1: [u64; ARCHTYPE_PROJECTILE_MAX / 64],
    archetype_projectile: Box<[ArchetypeProjectile8]>,
}

impl GameState {
    fn new() -> Self {
        Self {
            rng: Pcg64::new(),
            time: 0.0,
            actions: Actions::new(),
            camera: CameraState::new(),
            player: PlayerState::new(),
            archetype_projectile_bitmap_0: [0; ARCHTYPE_PROJECTILE_MAX / 64 / 64],
            archetype_projectile_bitmap_1: [0; ARCHTYPE_PROJECTILE_MAX / 64],
            archetype_projectile: vec![
                ArchetypeProjectile8::default();
                ARCHTYPE_PROJECTILE_MAX / 8
            ]
            .into_boxed_slice(),
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
            let [x, y] = self.rng.next_uniform_unit_circle_f32();
            let direction = vec3(x, 0.0, y);
            let velocity = player_velocity + direction * GAME_VARIABLES.weapon_projectile_speed;
            self.spawn_projectile(
                self.player.position,
                velocity,
                GAME_VARIABLES.weapon_projectile_lifetime,
            );

            self.player.weapon_cooldown = GAME_VARIABLES.weapon_cooldown;
        }

        // Expire projectiles
        for (base, base_word) in self.archetype_projectile_bitmap_0.iter_mut().enumerate() {
            for i in BitIter::new(std::iter::once(*base_word)) {
                let index = base * 64 + i;
                let word = &mut self.archetype_projectile_bitmap_1[index];
                for j in BitIter::new(std::iter::once(*word)) {
                    let index = index * 64 + j;
                    self.archetype_projectile[index / 8].lifetime[index % 8] -= delta_time;
                    let dead = self.archetype_projectile[index / 8].lifetime[index % 8] <= 0.0;
                    *word &= !((dead as u64) << j);
                }
                *base_word &= !(((*word == 0) as u64) << i);
            }
        }

        // Move projectiles
        for base in BitIter::new(self.archetype_projectile_bitmap_0.iter().copied()) {
            for i in BitIter::new(std::iter::once(self.archetype_projectile_bitmap_1[base])) {
                let i = base * 64 + i;
                self.archetype_projectile[i / 8].position_x[i % 8] +=
                    self.archetype_projectile[i / 8].velocity_x[i % 8] * delta_time;
                self.archetype_projectile[i / 8].position_z[i % 8] +=
                    self.archetype_projectile[i / 8].velocity_z[i % 8] * delta_time;
            }
        }
    }

    fn spawn_projectile(&mut self, position: Point3, velocity: Vec3, lifetime: f32) {
        let i = BitIter::new(
            self.archetype_projectile_bitmap_1
                .iter()
                .copied()
                .map(|x| !x),
        )
        .next()
        .unwrap();

        self.archetype_projectile[i / 8].position_x[i % 8] = position.x;
        self.archetype_projectile[i / 8].position_z[i % 8] = position.z;
        self.archetype_projectile[i / 8].velocity_x[i % 8] = velocity.x;
        self.archetype_projectile[i / 8].velocity_z[i % 8] = velocity.z;
        self.archetype_projectile[i / 8].lifetime[i % 8] = lifetime;
        self.archetype_projectile_bitmap_1[i / 64] |= 1 << (i % 64);
        self.archetype_projectile_bitmap_0[i / 64 / 64] |= 1 << ((i / 64) % 64);
    }
}

// https://theorangeduck.com/page/spring-roll-call
fn simple_spring_damper_exact(
    x: f32,
    velocity: f32,
    goal: f32,
    damping: f32,
    delta_time: f32,
) -> (f32, f32) {
    let y = damping / 2.0;
    let j0 = x - goal;
    let j1 = velocity + j0 * y;
    let eydt = exp_f32(-y * delta_time);
    (
        eydt * (j0 + j1 * delta_time) + goal,
        eydt * (velocity - j1 * y * delta_time),
    )
}

pub fn main() {
    #[cfg(debug_assertions)]
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1")
    }

    let app = create_app();
    let main_window = app.create_window(&WindowDesc {
        title: "shark",
        width: 800,
        height: 600,
    });

    let device = create_device(narcissus_gpu::DeviceBackend::Vulkan);

    let thread_token = ThreadToken::new();
    let thread_token = &thread_token;

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

    {
        let frame = device.begin_frame();

        let blåhaj_buffer = device.request_transient_buffer_with_data(
            &frame,
            thread_token,
            BufferUsageFlags::TRANSFER,
            blåhaj_image_data.as_slice(),
        );

        let mut cmd_encoder = device.request_cmd_encoder(&frame, thread_token);
        {
            let cmd_encoder = &mut cmd_encoder;

            device.cmd_barrier(
                cmd_encoder,
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
                cmd_encoder,
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
                cmd_encoder,
                None,
                &[ImageBarrier::layout_optimal(
                    &[Access::TransferWrite],
                    &[Access::FragmentShaderSampledImageRead],
                    blåhaj_image,
                    ImageAspectFlags::COLOR,
                )],
            );
        }

        device.submit(&frame, cmd_encoder);
        device.end_frame(frame);
    }

    let mut depth_width = 0;
    let mut depth_height = 0;
    let mut depth_image = default();

    let mut font_size_str = String::new();
    let mut primitive_instances = Vec::new();
    let mut primitive_vertices = Vec::new();
    let mut line_glyph_indices = Vec::new();
    let mut line_kern_advances = Vec::new();

    let mut action_queue = Vec::new();
    let mut game_state = GameState::new();

    let mut basic_transforms = vec![];

    'main: loop {
        let frame = device.begin_frame();
        {
            let frame = &frame;

            let (width, height, swapchain_image) = loop {
                let (width, height) = main_window.extent();
                if let Ok(result) = device.acquire_swapchain(
                    frame,
                    main_window.upcast(),
                    width,
                    height,
                    ImageFormat::BGRA8_SRGB,
                ) {
                    break result;
                }
            };

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

                        if key == Key::Escape {
                            break 'main;
                        }

                        {
                            let value = match pressed {
                                PressedState::Released => 0.0,
                                PressedState::Pressed => 1.0,
                            };

                            if key == Key::Left || key == Key::A {
                                action_queue.push(ActionEvent {
                                    action: Action::Left,
                                    value,
                                })
                            }
                            if key == Key::Right || key == Key::D {
                                action_queue.push(ActionEvent {
                                    action: Action::Right,
                                    value,
                                })
                            }
                            if key == Key::Up || key == Key::W {
                                action_queue.push(ActionEvent {
                                    action: Action::Up,
                                    value,
                                })
                            }
                            if key == Key::Down || key == Key::S {
                                action_queue.push(ActionEvent {
                                    action: Action::Down,
                                    value,
                                })
                            }
                            if key == Key::Space {
                                action_queue.push(ActionEvent {
                                    action: Action::Damage,
                                    value,
                                })
                            }
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

            game_state.tick(1.0 / 120.0, &action_queue);
            action_queue.clear();

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
            basic_transforms.push(Affine3::new(matrix, translation));

            // Render projectiles
            for base in BitIter::new(game_state.archetype_projectile_bitmap_0.iter().copied()) {
                for i in BitIter::new(std::iter::once(
                    game_state.archetype_projectile_bitmap_1[base],
                )) {
                    let i = base * 64 + i;
                    let translation = vec3(
                        game_state.archetype_projectile[i / 8].position_x[i % 8],
                        0.0,
                        game_state.archetype_projectile[i / 8].position_z[i % 8],
                    );
                    let velocity = vec3(
                        game_state.archetype_projectile[i / 8].velocity_x[i % 8],
                        0.0,
                        game_state.archetype_projectile[i / 8].velocity_z[i % 8],
                    );

                    let matrix = rotate_dir(velocity, Vec3::Y) * half_turn_y * scale;
                    basic_transforms.push(Affine3::new(matrix, translation))
                }
            }

            let camera_from_model = game_state.camera.camera_from_model();
            let clip_from_camera = Mat4::perspective_rev_inf_zo(
                HalfTurn::new(1.0 / 3.0),
                width as f32 / height as f32,
                0.01,
            );
            let clip_from_model = clip_from_camera * camera_from_model;

            let mut cmd_encoder = device.request_cmd_encoder(frame, thread_token);
            {
                let cmd_encoder = &mut cmd_encoder;

                if width != depth_width || height != depth_height {
                    device.destroy_image(frame, depth_image);
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
                        cmd_encoder,
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

                // Do some Font Shit.'
                let line0 = "Snarfe, Blåhaj! And the Quick Brown Fox jumped Over the Lazy doge.";
                let line1 = "加盟国は、国際連合と協力して";

                let mut x;
                let mut y = 0.0;

                let mut rng = Pcg64::new();

                primitive_instances.clear();
                primitive_vertices.clear();

                for line in 0..2 {
                    let (font_family, font_size_px, text) = if line & 1 == 0 {
                        (FontFamily::RobotoRegular, 22.0, line0)
                    } else {
                        (FontFamily::NotoSansJapanese, 22.0, line1)
                    };

                    let font = fonts.font(font_family);
                    let scale = font.scale_for_size_px(font_size_px);

                    x = 0.0;
                    y += (font.ascent() - font.descent() + font.line_gap()) * scale;

                    font_size_str.clear();
                    write!(&mut font_size_str, "{font_size_px}: ").unwrap();

                    line_glyph_indices.clear();
                    line_glyph_indices.extend(font_size_str.chars().chain(text.chars()).map(|c| {
                        font.glyph_index(c)
                            .unwrap_or_else(|| font.glyph_index('□').unwrap())
                    }));

                    line_kern_advances.clear();
                    line_kern_advances.push(0.0);
                    line_kern_advances.extend(array_windows(line_glyph_indices.as_slice()).map(
                        |&[prev_index, next_index]| font.kerning_advance(prev_index, next_index),
                    ));

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

                            x += advance * scale;

                            let color = *rng
                                .array_select(&[0xfffac228, 0xfff57d15, 0xffd44842, 0xff9f2a63]);

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
                        frame,
                        thread_token,
                        BufferUsageFlags::TRANSFER,
                        texture,
                    );

                    device.cmd_barrier(
                        cmd_encoder,
                        None,
                        &[ImageBarrier::layout_optimal(
                            &[Access::ShaderSampledImageRead],
                            &[Access::TransferWrite],
                            image,
                            ImageAspectFlags::COLOR,
                        )],
                    );

                    device.cmd_copy_buffer_to_image(
                        cmd_encoder,
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
                        cmd_encoder,
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
                    cmd_encoder,
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
                    cmd_encoder,
                    &[Scissor {
                        offset: Offset2d { x: 0, y: 0 },
                        extent: Extent2d { width, height },
                    }],
                );

                device.cmd_set_viewports(
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
                    device.cmd_set_pipeline(cmd_encoder, basic_pipeline.pipeline);

                    let basic_uniforms = BasicUniforms { clip_from_model };

                    let uniform_buffer = device.request_transient_buffer_with_data(
                        frame,
                        thread_token,
                        BufferUsageFlags::UNIFORM,
                        &basic_uniforms,
                    );

                    let transform_buffer = device.request_transient_buffer_with_data(
                        frame,
                        thread_token,
                        BufferUsageFlags::STORAGE,
                        basic_transforms.as_slice(),
                    );

                    device.cmd_set_bind_group(
                        frame,
                        cmd_encoder,
                        basic_pipeline.uniforms_bind_group_layout,
                        0,
                        &[Bind {
                            binding: 0,
                            array_element: 0,
                            typed: TypedBind::UniformBuffer(&[uniform_buffer.to_arg()]),
                        }],
                    );

                    device.cmd_set_bind_group(
                        frame,
                        cmd_encoder,
                        basic_pipeline.storage_bind_group_layout,
                        1,
                        &[
                            Bind {
                                binding: 0,
                                array_element: 0,
                                typed: TypedBind::StorageBuffer(&[blåhaj_vertex_buffer.to_arg()]),
                            },
                            Bind {
                                binding: 1,
                                array_element: 0,
                                typed: TypedBind::StorageBuffer(&[transform_buffer.to_arg()]),
                            },
                            Bind {
                                binding: 2,
                                array_element: 0,
                                typed: TypedBind::Sampler(&[basic_pipeline.sampler]),
                            },
                            Bind {
                                binding: 3,
                                array_element: 0,
                                typed: TypedBind::Image(&[(ImageLayout::Optimal, blåhaj_image)]),
                            },
                        ],
                    );

                    device.cmd_set_index_buffer(
                        cmd_encoder,
                        blåhaj_index_buffer.to_arg(),
                        0,
                        IndexType::U16,
                    );

                    device.cmd_draw_indexed(
                        cmd_encoder,
                        blåhaj_indices.len() as u32,
                        basic_transforms.len() as u32,
                        0,
                        0,
                        0,
                    );

                    // We're done with you now!
                    basic_transforms.clear();
                };

                // Render text stuff.
                text_pipeline.bind(
                    device.as_ref(),
                    frame,
                    thread_token,
                    cmd_encoder,
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

                device.cmd_draw(cmd_encoder, primitive_vertices.len() as u32, 1, 0, 0);

                device.cmd_end_rendering(cmd_encoder);
            }
            device.submit(frame, cmd_encoder);
        }
        device.end_frame(frame);
    }
}
