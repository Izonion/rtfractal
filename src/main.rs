#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::time::{Instant, Duration};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
// const WIDTH: u32 = 320;
// const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 17;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    transforms: Vec<ScreenTransform>,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    let clear_buffer = {
        let mut clear_buffer = [0u8; (WIDTH * HEIGHT * 4) as usize];
        for i in 0..(WIDTH * HEIGHT) as usize {
            clear_buffer[i * 4..i * 4 + 4].copy_from_slice(&[0x48, 0xb2, 0xe8, 0xff]);
        }
        Box::new(clear_buffer) as Box<[u8]>
    };

    let mut last_frame = Instant::now();
    let mut cumulative_delta = Duration::from_secs_f64(0.0);
    let mut frame_count = 0;
    let max_frame_count = 60;
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(&clear_buffer, pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            let current_frame = Instant::now();
            let delta_frame = current_frame - last_frame;
            cumulative_delta += delta_frame;
            frame_count += 1;
            if frame_count == max_frame_count {
                println!("{:?}", cumulative_delta / max_frame_count);
                frame_count = 0;
                cumulative_delta = Duration::from_secs_f64(0.0);
            }
            last_frame = current_frame;

            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        let mut transforms = Vec::new();
        transforms.push(ScreenTransform {
            position: Vec2 { x: WIDTH as f32 / 2.0, y: HEIGHT as f32 / 2.0 },
            rotation: 1.25,
            scale: 0.6,
            alpha: 0xff
        });
        Self {
            transforms,
        }
    }

    fn update(&mut self) {
        // for mut transform in &mut self.transforms {
        // }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, clear_buffer: &Box<[u8]>, frame: &mut [u8]) {
        // Can extend to not have to make a new vec every time
        frame.copy_from_slice(clear_buffer);
        let mut grid = PixelGrid(frame);
        for transform in &self.transforms {
            transform.draw(&mut grid);
        }
    }
}

struct PixelGrid<'a>(&'a mut [u8]);

impl<'a> PixelGrid<'a> {
    fn set_pixel(&mut self, x: i16, y: i16, pixel: &[u8; 4]) {
        let x = (x as u32).clamp(0, WIDTH - 1);
        let y = (y as u32).clamp(0, HEIGHT - 1);
        let i = (x as usize + (y * WIDTH) as usize) * 4;
        self.0[i..i + 4].copy_from_slice(pixel);
    }
}

struct Vec2 {
    x: f32,
    y: f32,
}

struct ScreenTransform {
    position: Vec2,
    rotation: f32,
    scale: f32,
    alpha: u8,
}

impl ScreenTransform {
    fn draw(&self, grid: &mut PixelGrid) {
        let width = WIDTH as f32 * self.scale;
        let height = HEIGHT as f32 * self.scale;

        fn transform(x: f32, y: f32, tr: &ScreenTransform) -> (i16, i16) {
            let width = WIDTH as f32 * tr.scale;
            let height = HEIGHT as f32 * tr.scale;
            let x = x - width / 2.0;
            let y = y - height / 2.0;
            let sin = tr.rotation.sin();
            let cos = tr.rotation.cos();
            (
                (
                    x * cos - y * sin +
                    tr.position.x
                ) as i16,
                (
                    y * cos + x * sin +
                    tr.position.y
                ) as i16,
            )
        }

        // let precision_scale = 1.0;
        let precision_scale = 2.0 - self.scale;
        for x in 0..(width * precision_scale) as i16 {
            for y in (0..(10.0 * precision_scale) as i16).chain(((height - 10.0) * precision_scale) as i16..(height * precision_scale) as i16) {
                let transformed = transform(x as f32 / precision_scale, y as f32 / precision_scale, &self);
                grid.set_pixel( transformed.0,
                                transformed.1,
                                &[0x00, 0x00, 0x00, self.alpha]);
            }
        }
        for x in (0..(10.0 * precision_scale) as i16).chain(((width - 10.0) * precision_scale) as i16..(width * precision_scale) as i16) {
            for y in (10.0 * precision_scale) as i16..((height - 10.0) * precision_scale) as i16 {
                let transformed = transform(x as f32 / precision_scale, y as f32 / precision_scale, &self);
                grid.set_pixel( transformed.0,
                                transformed.1,
                                &[0x00, 0x00, 0x00, self.alpha]);
            }
        }

        // for x in 0..(width * precision_scale) as i16 {
        //     for y in 0..(height * precision_scale) as i16 {
        //         let tex_x = x as f32 / width / precision_scale * WIDTH as f32;
        //         let tex_y = y as f32 / height / precision_scale * HEIGHT as f32;
        //         let transformed = transform(x as f32 / precision_scale, y as f32 / precision_scale, &self);
        //         grid.set_pixel( transformed.0,
        //                         transformed.1,
        //                         &[if (tex_x) as i16 % 100 > 50 {0xff} else {0x00}, if (tex_y) as i16 % 100 > 50 {0xff} else {0x00}, 0x00, self.alpha]);
        //     }
        // }
    }
}