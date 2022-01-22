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

mod pixel;

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

            if input.mouse_pressed(0) {

            }

            // Update internal state and request a redraw
            world.update(input.mouse());
            window.request_redraw();
        }
    });
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        let mut transforms = Vec::new();
        transforms.push(ScreenTransform { transform: pixel::Transform {
            position: pixel::Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
            rotation: 1.25,
            scale: 0.6,
            alpha: 0xff,
        }});
        Self {
            transforms,
        }
    }

    fn update(&mut self, mouse_pos: Option<(f32, f32)>) {
        for mut transform in &mut self.transforms {
            transform.mouse_input(mouse_pos);
            transform.transform.rotation -= 0.001;
            transform.transform.scale = 0.001f32.max(transform.transform.scale - 0.001);
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, clear_buffer: &Box<[u8]>, frame: &mut [u8]) {
        // Can extend to not have to make a new vec every time
        frame.copy_from_slice(clear_buffer);
        let mut grid = pixel::PixelGrid(frame);
        for transform in &self.transforms {
            transform.draw(&mut grid);
        }
    }
}

struct ScreenTransform {
    transform: pixel::Transform,
}

impl ScreenTransform {
    fn draw(&self, grid: &mut pixel::PixelGrid) {
        let width = WIDTH as f32;
        let height = HEIGHT as f32;

        // let precision_scale = 1.0;
        let mut x = -width / 2.0;
        while x < width / 2.0 {
            for line_width in 0..10 {
                grid.set_pixel_transformed( pixel::Vec2::new(x, -height / 2.0 + (line_width as f32) / self.transform.scale),
                                            &self.transform,
                                            &[0x00, 0x00, 0x00, 0xff]);
                grid.set_pixel_transformed( pixel::Vec2::new(x, height / 2.0 - (line_width as f32) / self.transform.scale),
                                            &self.transform,
                                            &[0x00, 0x00, 0x00, 0xff]);
            }
            x += self.transform.scale;
        }
        let mut y = -height / 2.0;
        while y < height / 2.0 {
            for line_width in 0..10 {
                grid.set_pixel_transformed( pixel::Vec2::new(-width / 2.0 + (line_width as f32) / self.transform.scale, y),
                                            &self.transform,
                                            &[0x00, 0x00, 0x00, 0xff]);
                grid.set_pixel_transformed( pixel::Vec2::new(width / 2.0 - (line_width as f32) / self.transform.scale, y),
                                            &self.transform,
                                            &[0x00, 0x00, 0x00, 0xff]);
            }
            y += self.transform.scale;
        }
        // let y = 0;
        // for x in (0..(10.0 * precision_scale) as i16).chain(((width - 10.0) * precision_scale) as i16..(width * precision_scale) as i16) {
        //     for y in (10.0 * precision_scale) as i16..((height - 10.0) * precision_scale) as i16 {
        //         let transformed = transform(x as f32 / precision_scale, y as f32 / precision_scale, &self);
        //         grid.set_pixel( transformed.0,
        //                         transformed.1,
        //                         &[0x00, 0x00, 0x00, self.alpha]);
        //     }
        // }

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

    fn mouse_input(&mut self, pos: Option<(f32, f32)>) {
        // if let Some(pos) = pos {
        //     let (x, y) = inverse_transform(pos.0, pos.1, self);
        //     println!("{:?}, {:?}", x, y);
        // } else {

        // }
    }
}