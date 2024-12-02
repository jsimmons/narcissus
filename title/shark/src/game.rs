use std::f32::consts::SQRT_2;

use narcissus_core::{box_assume_init, default, random::Pcg64, zeroed_box, BitIter};
use narcissus_maths::{clamp, perlin_noise3, sin_pi_f32, vec3, Deg, HalfTurn, Mat4, Point3, Vec3};

use crate::spring::simple_spring_damper_exact;

const ARCHTYPE_PROJECTILE_MAX: usize = 65536;

pub struct GameVariables {
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

pub static GAME_VARIABLES: GameVariables = GameVariables {
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
    pub action: Action,
    pub value: f32,
}

pub struct Actions {
    pub old_values: [f32; Self::MAX_ACTIONS],
    pub new_values: [f32; Self::MAX_ACTIONS],
}

impl Default for Actions {
    fn default() -> Self {
        Self {
            old_values: [0.0; Self::MAX_ACTIONS],
            new_values: [0.0; Self::MAX_ACTIONS],
        }
    }
}

impl Actions {
    const MAX_ACTIONS: usize = 256;

    fn is_active(&self, action: Action) -> bool {
        self.new_values[action as usize] != 0.0
    }

    pub fn became_active_this_frame(&self, action: Action) -> bool {
        self.new_values[action as usize] != 0.0 && self.old_values[action as usize] == 0.0
    }

    pub fn tick(&mut self, action_queue: &[ActionEvent]) {
        self.old_values = self.new_values;

        for event in action_queue {
            self.new_values[event.action as usize] = event.value;
        }
    }
}

pub struct PlayerState {
    pub position: Point3,
    pub heading: Vec3,
    pub weapon_cooldown: f32,
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

pub struct CameraState {
    pub eye_offset: Vec3,

    pub shake: f32,
    pub shake_offset: Vec3,

    pub position: Point3,
    pub velocity: Vec3,
}

impl Default for CameraState {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraState {
    pub fn new() -> Self {
        let theta = HalfTurn::from(GAME_VARIABLES.camera_angle).as_f32();
        let hypotenuse = GAME_VARIABLES.camera_distance;
        let height = sin_pi_f32(theta) * hypotenuse;
        let base = (hypotenuse * hypotenuse - height * height).sqrt();

        // Rotate camera
        let one_on_sqrt2 = 1.0 / SQRT_2;
        let eye_offset = vec3(-base * one_on_sqrt2, height, -base * one_on_sqrt2);

        Self {
            eye_offset,

            shake: 0.0,
            shake_offset: Vec3::ZERO,

            position: Point3::ZERO,
            velocity: Vec3::ZERO,
        }
    }

    pub fn tick(&mut self, target: Point3, time: f32, delta_time: f32) {
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

    pub fn camera_from_model(&self) -> Mat4 {
        let position = self.position + self.shake_offset;
        let eye = position + self.eye_offset;
        Mat4::look_at(eye, position, Vec3::Y)
    }
}

#[derive(Clone, Copy, Default)]
#[repr(align(16))]
pub struct ArchetypeProjectileBlock {
    pub position_x: [f32; Self::WIDTH],
    pub position_z: [f32; Self::WIDTH],
    pub velocity_x: [f32; Self::WIDTH],
    pub velocity_z: [f32; Self::WIDTH],
    pub lifetime: [f32; Self::WIDTH],
}

impl ArchetypeProjectileBlock {
    const WIDTH: usize = 8;
}

#[derive(Default)]
pub struct ArchetypeProjectileChunk {
    pub bitmap: [u8; Self::WIDTH],
    pub blocks: [ArchetypeProjectileBlock; Self::WIDTH],
}

impl ArchetypeProjectileChunk {
    const WIDTH: usize = 8;
    const LEN: usize = Self::WIDTH * ArchetypeProjectileBlock::WIDTH;
}

pub struct ArchetypeProjectile {
    pub bitmap_non_empty: [u64; ARCHTYPE_PROJECTILE_MAX / ArchetypeProjectileChunk::LEN / 64],
    pub bitmap_non_full: [u64; ARCHTYPE_PROJECTILE_MAX / ArchetypeProjectileChunk::LEN / 64],
    pub chunks: [ArchetypeProjectileChunk; ARCHTYPE_PROJECTILE_MAX / ArchetypeProjectileChunk::LEN],
}

pub struct GameState {
    pub rng: Pcg64,

    pub time: f32,
    pub actions: Actions,
    pub camera: CameraState,
    pub player: PlayerState,

    pub archetype_projectile: Box<ArchetypeProjectile>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let mut archetype_projectile: Box<ArchetypeProjectile> =
            unsafe { box_assume_init(zeroed_box()) };
        archetype_projectile.bitmap_non_full.fill(u64::MAX);
        Self {
            rng: Pcg64::new(),
            time: 0.0,
            actions: default(),
            camera: CameraState::new(),
            player: PlayerState::new(),
            archetype_projectile,
        }
    }

    pub fn tick(&mut self, delta_time: f32, action_queue: &[ActionEvent]) {
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

    pub fn spawn_projectile(&mut self, position: Point3, velocity: Vec3, lifetime: f32) {
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
