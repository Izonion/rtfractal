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
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
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
        Self {
            box_x: 23,
            box_y: 16,
            velocity_x: 10,
            velocity_y: 10,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, clear_buffer: &Box<[u8]>, frame: &mut [u8]) {
        // Can extend to not have to make a new vec every time
        frame.copy_from_slice(clear_buffer);
        let mut grid = PixelGrid(frame);
        // pixels.iter_mut().for_each(|pixel| pixel.copy_from_slice(&[0x48, 0xb2, 0xe8, 0xff]));
        for inbox_x in 0..BOX_SIZE {
            for inbox_y in 0..BOX_SIZE {
                let x = self.box_x + inbox_x;
                let y = self.box_y + inbox_y;
                grid.set_pixel(x, y, &[0x5e, 0x48, 0xe8, 0xff]);
            }
        }
        // for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        //     let x = (i % WIDTH as usize) as i16;
        //     let y = (i / WIDTH as usize) as i16;

        //     let inside_the_box = x >= self.box_x
        //         && x < self.box_x + BOX_SIZE
        //         && y >= self.box_y
        //         && y < self.box_y + BOX_SIZE;

        //     let rgba = if inside_the_box {
        //         [0x5e, 0x48, 0xe8, 0xff]
        //     } else {
        //         [0x48, 0xb2, 0xe8, 0xff]
        //     };

        //     pixel.copy_from_slice(&rgba);
        // }
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
