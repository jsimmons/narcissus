use std::fmt::Write;
use std::time::{Duration, Instant};

use draw::DrawState;
use game::{Action, ActionEvent, GameState};
use narcissus_core::Widen;

use shark_shaders::pipelines::{Draw2dCmd, Draw2dScissor};

use renderdoc_sys as rdoc;

use fonts::{FontFamily, Fonts};
use narcissus_app::{create_app, Event, Key, WindowDesc};
use narcissus_core::default;
use narcissus_font::{FontCollection, GlyphCache, HorizontalMetrics};
use narcissus_gpu::{
    create_device, ColorSpace, ImageFormat, ImageUsageFlags, PresentMode, SwapchainConfigurator,
    SwapchainImage, ThreadToken,
};
use narcissus_maths::{sin_cos_pi_f32, vec2, Vec2};

mod draw;
mod fonts;
mod game;
mod helpers;
pub mod microshades;
mod spring;

const GLYPH_CACHE_SIZE: usize = 1024;

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
