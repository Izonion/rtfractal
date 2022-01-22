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

	// let mut pixels = {
		let window_size = window.inner_size();
		let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
		Pixels::new(WIDTH, HEIGHT, surface_texture)?
	};
	let mut world = World::new();

	let clear_buffer = {
		let mut clear_buffer = [0u8; (WIDTH * HEIGHT * 4) as usize];
		for i in 0..(WIDTH * HEIGHT) as usize {
			clear_buffer[i * 4..i * 4 + 4].copy_from_slice(&[0xE3, 0xE3, 0xE3, 0xff]);
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

			let mouse_state = if input.mouse_pressed(0) {
				MouseClickState::Pressed
			} else if input.mouse_released(0) {
				MouseClickState::Released
			} else if input.mouse_held(0) {
				MouseClickState::Held
			} else {
				MouseClickState::Idle
			};

			let mouse_pos = if let Some(mouse_pos) = input.mouse() {
				if let Ok((x, y)) = pixels.window_pos_to_pixel(mouse_pos) {
					Some((x as f32, y as f32))
				} else { None }
			} else { None };

			// Update internal state and request a redraw
			world.update(mouse_pos, mouse_state);
			window.request_redraw();
		}
	});
}

#[derive(Copy, Clone)]
enum MouseClickState {
	Pressed,
	Held,
	Released,
	Idle,
}

impl World {
	/// Create a new `World` instance that can draw a moving box.
	fn new() -> Self {
		let mut transforms = Vec::new();
		transforms.push(ScreenTransform { transform: pixel::Transform {
			position: pixel::Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
			rotation: 0.0,
			scale: 0.6,
			alpha: 0xff,
		}, hovering: None, grabbing: None});
		Self {
			transforms,
		}
	}

	fn update(&mut self, mouse_pos: Option<(f32, f32)>, mouse_state: MouseClickState) {
		for transform in &mut self.transforms {
			if let Some((x, y)) = mouse_pos {
				transform.mouse_input(pixel::Vec2::new(x, y), mouse_state);
			}
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

#[derive(Eq, PartialEq, Copy, Clone)]
enum Hoverables {
	Rotate,
	Translate,
	Scale,
	Alpha,
}

struct ScreenTransform {
	transform: pixel::Transform,
	hovering: Option<Hoverables>,
	grabbing: Option<Hoverables>,
}

const ROTATE_HOVERABLE: [f32; 4] = [-85.0, -200.0, 85.0, -140.0]; // Two bounding points
const TRANSLATE_HOVERABLE: [f32; 4] = [-85.0, -200.0, 85.0, -140.0]; // Two bounding points

impl ScreenTransform {
	fn draw(&self, grid: &mut pixel::PixelGrid) {
		let width = WIDTH as f32;
		let height = HEIGHT as f32;

		let unhoverable_color = [0x13, 0x1B, 0x23];
		let hoverable_color = [0xA6, 0x26, 0x39];
		let hovering_color = [0xDB, 0x32, 0x4D];
		let clicking_color = [0x85, 0x1E, 0x2E];

		// let precision_scale = 1.0;
		let mut x = -width / 2.0;
		while x < width / 2.0 {
			for line_width in 0..10 {
				grid.set_pixel_transformed( pixel::Vec2::new(x, -height / 2.0 + (line_width as f32) / self.transform.scale),
											&self.transform,
											&unhoverable_color);
				grid.set_pixel_transformed( pixel::Vec2::new(x, height / 2.0 - (line_width as f32) / self.transform.scale),
											&self.transform,
											&unhoverable_color);
			}
			x += self.transform.scale;
		}
		let mut y = -height / 2.0;
		while y < height / 2.0 {
			for line_width in 0..10 {
				grid.set_pixel_transformed( pixel::Vec2::new(-width / 2.0 + (line_width as f32) / self.transform.scale, y),
											&self.transform,
											&unhoverable_color);
				grid.set_pixel_transformed( pixel::Vec2::new(width / 2.0 - (line_width as f32) / self.transform.scale, y),
											&self.transform,
											&unhoverable_color);
			}
			y += self.transform.scale;
		}

		let rotate_color =
			if self.grabbing == Some(Hoverables::Rotate) { &clicking_color }
			else if self.hovering == Some(Hoverables::Rotate) { &hovering_color }
			else { &hoverable_color };
		for x in -75..75 {
			let outer_arc = (100.0*100.0 - x as f32 * x as f32).sqrt() as i16;
			let inner_arc = (75.0*75.0 - x as f32 * x as f32).sqrt() as i16;
			for y in inner_arc.max(50)..outer_arc {
				grid.set_pixel_transformed( pixel::Vec2::new(x as f32, -y as f32 - 100.0),
											&self.transform,
											&rotate_color);
			}
		}
		for x in 40..85 {
			for y in 40..x {
				grid.set_pixel_transformed( pixel::Vec2::new(x as f32, -y as f32 - 100.0),
											&self.transform,
											&rotate_color);
				grid.set_pixel_transformed( pixel::Vec2::new(-x as f32, -y as f32 - 100.0),
											&self.transform,
											&rotate_color);
			}
		}
	}

	fn mouse_input(&mut self, pos: pixel::Vec2, mouse_state: MouseClickState) {
		let local_pos = self.transform.apply_inverse(pos);
		if let Some(grabbing) = self.grabbing {
			self.hovering = None;
			match grabbing {
				Hoverables::Rotate => {
					let pos = pos - self.transform.position;
					let angle = if pos.y >= 0.0 {
						-(pos.x / pos.y).atan() + std::f32::consts::PI
					} else {
						-(pos.x / pos.y).atan()
					};
					self.transform.rotation = angle;
				},
				_ => (),
			}
			match mouse_state {
				MouseClickState::Released => {
					self.grabbing = None;
				},
				_ => (),
			}
		} else {
			if 		local_pos.x > ROTATE_HOVERABLE[0] &&
					local_pos.x < ROTATE_HOVERABLE[2] &&
					local_pos.y > ROTATE_HOVERABLE[1] &&
					local_pos.y < ROTATE_HOVERABLE[3] {
				self.hovering = Some(Hoverables::Rotate);
				match mouse_state {
					MouseClickState::Pressed => {
						self.grabbing = Some(Hoverables::Rotate);
					},
					_ => (),
				}
			} else if 	local_pos.x > ROTATE_HOVERABLE[0] &&
						local_pos.x < ROTATE_HOVERABLE[2] &&
						local_pos.y > ROTATE_HOVERABLE[1] &&
						local_pos.y < ROTATE_HOVERABLE[3] else {
				self.hovering = None;
			}
		}
	}
}