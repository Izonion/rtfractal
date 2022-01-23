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

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;
// const WIDTH: u32 = 320;
// const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 17;

#[derive(Eq, PartialEq, Copy, Clone)]
enum WorldHoverMode {
	Add,
}

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
	transforms: Vec<ScreenTransform>,
	hovering: Option<WorldHoverMode>,
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum EditMode {
	Dual,
	Edit,
	View,
}

fn main() -> Result<(), Error> {
	env_logger::init();
	let event_loop = EventLoop::new();
	let mut input = WinitInputHelper::new();
	let window = {
		let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
		WindowBuilder::new()
			.with_title("Fractalical")
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

	let mut clear_buffer = Box::new([0u8; (WIDTH * HEIGHT * 4) as usize]);
	for i in 0..(WIDTH * HEIGHT) as usize {
		let x = (i % WIDTH as usize) as u32;
		let y = (i / WIDTH as usize) as u32;
		if x > WIDTH * 4 / 10 && x < WIDTH * 6 / 10 &&
			y > HEIGHT * 4 / 10 && y < HEIGHT * 6 / 10 {
			clear_buffer[i * 4..i * 4 + 4].copy_from_slice(&[0x23, 0xA9, 0x50, 0xff]);
		} else {
			clear_buffer[i * 4..i * 4 + 4].copy_from_slice(&[0xE3, 0xE3, 0xE3, 0xff]);
		}
	}

	let mut last_frame_buffer = Box::new([0u8; (WIDTH * HEIGHT * 4) as usize]);

	let mut edit_mode = EditMode::Dual;

	let mut last_frame = Instant::now();
	let mut cumulative_delta = Duration::from_secs_f64(0.0);
	let mut frame_count = 0;
	let max_frame_count = 60;
	event_loop.run(move |event, _, control_flow| {
		// Draw the current frame
		if let Event::RedrawRequested(_) = event {
			world.draw(&clear_buffer, pixels.get_frame(), &mut last_frame_buffer, edit_mode);
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
			} else if input.key_pressed(VirtualKeyCode::Key1) {
				edit_mode = EditMode::Dual;
			} else if input.key_pressed(VirtualKeyCode::Key2) {
				edit_mode = EditMode::Edit;
			} else if input.key_pressed(VirtualKeyCode::Key3) {
				edit_mode = EditMode::View;
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
			if edit_mode != EditMode::View {
				world.update(mouse_pos, mouse_state);
			}
			window.request_redraw();
		}
	});
}

#[derive(Copy, Clone, Eq, PartialEq)]
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
				alpha: 0xf0,
			}, hovering: None, grabbing: None, scale_start: None, controls_visible: false, dead: false});
		Self {
			transforms,
			hovering: None,
		}
	}

	fn update(&mut self, mouse_pos: Option<(f32, f32)>, mouse_state: MouseClickState) {
		let mut first_one = None;
		let mut dead_ones = Vec::new();
		for (i, transform) in self.transforms.iter_mut().enumerate() {
			if transform.dead { dead_ones.push(i); }
			if let Some((x, y)) = mouse_pos {
				if transform.mouse_input(pixel::Vec2::new(x, y), mouse_state) {
					first_one = Some(i);
					break
				}
			}
		}
		if let Some(i) = first_one {
			let top = self.transforms.remove(i);
			self.transforms.insert(0, top);
		}
		for dead_one in dead_ones { self.transforms.remove(dead_one); }

		if let Some(mouse_pos) = mouse_pos {
			if 		mouse_pos.0 > 20.0 &&
					mouse_pos.0 < 80.0 &&
					mouse_pos.1 > 20.0 &&
					mouse_pos.1 < 80.0 {
				self.hovering = Some(WorldHoverMode::Add);
			} else {
				self.hovering = None;
			}
		}
		if self.hovering.is_some() && mouse_state == MouseClickState::Pressed {
			self.transforms.push(ScreenTransform { transform: pixel::Transform {
					position: pixel::Vec2::new(	WIDTH as f32 / 2.0 - rand::random::<f32>() * 100.0 + 50.0,
												HEIGHT as f32 / 2.0 - rand::random::<f32>() * 100.0 + 50.0),
					rotation: rand::random::<f32>() * 0.1 - 0.05,
					scale: rand::random::<f32>() * 0.1 + 0.495,
					alpha: 0xf0,
				}, hovering: None, grabbing: None, scale_start: None, controls_visible: false, dead: false});
		}
	}

	/// Draw the `World` state to the frame buffer.
	///
	/// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
	fn draw(&self, clear_buffer: &Box<[u8; (WIDTH * HEIGHT * 4) as usize]>, frame: &mut [u8], last_frame_buffer: &mut Box<[u8; (WIDTH * HEIGHT * 4) as usize]>, edit_mode: EditMode) {
		frame.copy_from_slice(&clear_buffer[..]);
		if edit_mode == EditMode::Dual || edit_mode == EditMode::View {
			let mut grid = pixel::PixelGrid(frame);
			for (i, pixel) in last_frame_buffer.chunks_exact_mut(4).enumerate() {
				if pixel[0] == 0xE3 { continue; }
				let x = (i % WIDTH as usize) as f32 - WIDTH as f32 / 2.0;
				let y = (i / WIDTH as usize) as f32 - HEIGHT as f32 / 2.0;
				for transform in &self.transforms {
					grid.set_pixel_transformed(pixel::Vec2::new(x, y), &transform.transform, &[pixel[0], pixel[1], pixel[2]]);
				}
			}
			last_frame_buffer.copy_from_slice(frame);
		}
		if edit_mode == EditMode::Dual || edit_mode == EditMode::Edit {
			let mut grid = pixel::PixelGrid(frame);
			let add_color =
						if self.hovering == Some(WorldHoverMode::Add) { &HOVERING_COLOR }
						else { &HOVERABLE_COLOR };
			for x in 43..57 {
				for y in 20..80 {
					grid.set_pixel( pixel::Vec2::new(x as f32, y as f32),
												&add_color);
					grid.set_pixel( pixel::Vec2::new(y as f32, x as f32),
												&add_color);
				}
			}

			for transform in &self.transforms {
				transform.draw(&mut grid);
			}
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Hoverables {
	Rotate,
	Translate,
	Scale,
	Alpha,
	Delete,
}

struct ScreenTransform {
	transform: pixel::Transform,
	controls_visible: bool,
	hovering: Option<Hoverables>,
	grabbing: Option<Hoverables>,
	scale_start: Option<pixel::Vec2>,
	dead: bool,
}

const UNHOVERABLE_COLOR: [u8; 3] = [0x13, 0x1B, 0x23];
const HOVERABLE_COLOR: [u8; 3] = [0xA6, 0x26, 0x39];
const HOVERING_COLOR: [u8; 3] = [0xDB, 0x32, 0x4D];
const CLICKING_COLOR: [u8; 3] = [0x85, 0x1E, 0x2E];

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
											&UNHOVERABLE_COLOR);
				grid.set_pixel_transformed( pixel::Vec2::new(x, height / 2.0 - (line_width as f32) / self.transform.scale),
											&self.transform,
											&UNHOVERABLE_COLOR);
			}
			x += self.transform.scale;
		}
		let mut y = -height / 2.0;
		while y < height / 2.0 {
			for line_width in 0..10 {
				grid.set_pixel_transformed( pixel::Vec2::new(-width / 2.0 + (line_width as f32) / self.transform.scale, y),
											&self.transform,
											&UNHOVERABLE_COLOR);
				grid.set_pixel_transformed( pixel::Vec2::new(width / 2.0 - (line_width as f32) / self.transform.scale, y),
											&self.transform,
											&UNHOVERABLE_COLOR);
			}
			y += self.transform.scale;
		}

		if !self.controls_visible { return }

		let rotate_color =
			if self.grabbing == Some(Hoverables::Rotate) { &CLICKING_COLOR }
			else if self.hovering == Some(Hoverables::Rotate) { &HOVERING_COLOR }
			else { &HOVERABLE_COLOR };
		for x in -75..75 {
			let outer_arc = (100.0*100.0 - x as f32 * x as f32).sqrt() as i16;
			let inner_arc = (75.0*75.0 - x as f32 * x as f32).sqrt() as i16;
			for y in inner_arc.max(50)..outer_arc {
				grid.set_pixel_transformed( pixel::Vec2::new(x as f32, -y as f32 - height / 2.0 + 100.0 + 15.0 / self.transform.scale),
											&self.transform,
											&rotate_color);
			}
		}
		for x in 40..85 {
			for y in 40..x {
				grid.set_pixel_transformed( pixel::Vec2::new(x as f32, -y as f32 - height / 2.0 + 100.0 + 15.0 / self.transform.scale),
											&self.transform,
											&rotate_color);
				grid.set_pixel_transformed( pixel::Vec2::new(-x as f32, -y as f32 - height / 2.0 + 100.0 + 15.0 / self.transform.scale),
											&self.transform,
											&rotate_color);
			}
		}

		let translate_color =
			if self.grabbing == Some(Hoverables::Translate) { &CLICKING_COLOR }
			else if self.hovering == Some(Hoverables::Translate) { &HOVERING_COLOR }
			else { &HOVERABLE_COLOR };
		for x in -10..10 {
			for y in -40..40 {
				grid.set_pixel_transformed( pixel::Vec2::new(x as f32, y as f32),
											&self.transform,
											&translate_color);
				grid.set_pixel_transformed( pixel::Vec2::new(y as f32, x as f32),
											&self.transform,
											&translate_color);
			}
		}
		for x in -25i32..25 {
			for y in 0..(25 - x.abs()) {
				grid.set_pixel_transformed( pixel::Vec2::new(x as f32, (y + 40) as f32),
											&self.transform,
											&translate_color);
				grid.set_pixel_transformed( pixel::Vec2::new(x as f32, -(y + 40) as f32),
											&self.transform,
											&translate_color);
				grid.set_pixel_transformed( pixel::Vec2::new((y + 40) as f32, x as f32),
											&self.transform,
											&translate_color);
				grid.set_pixel_transformed( pixel::Vec2::new(-(y + 40) as f32, x as f32),
											&self.transform,
											&translate_color);
			}
		}

		let scale_color =
			if self.grabbing == Some(Hoverables::Scale) { &CLICKING_COLOR }
			else if self.hovering == Some(Hoverables::Scale) { &HOVERING_COLOR }
			else { &HOVERABLE_COLOR };
		for x in 0..80 {
			for y in 60..80 {
				grid.set_pixel_transformed( pixel::Vec2::new(x as f32 + width / 2.0 - 80.0 - 15.0 / self.transform.scale, y as f32 + height / 2.0 - 80.0 - 15.0 / self.transform.scale),
											&self.transform,
											&scale_color);
				grid.set_pixel_transformed( pixel::Vec2::new(y as f32 + width / 2.0 - 80.0 - 15.0 / self.transform.scale, x as f32 + height / 2.0 - 80.0 - 15.0 / self.transform.scale),
											&self.transform,
											&scale_color);
			}
		}

		let delete_color =
			if self.grabbing == Some(Hoverables::Delete) { &CLICKING_COLOR }
			else if self.hovering == Some(Hoverables::Delete) { &HOVERING_COLOR }
			else { &HOVERABLE_COLOR };
		for x in 30..50 {
			for y in -40..40 {
				let x = x as f32;
				let y = y as f32;
				grid.set_pixel_transformed( pixel::Vec2::new((x + y) / 2.0 - width / 2.0 + 15.0 / self.transform.scale,
															 (x - y) / 2.0 - height / 2.0 + 15.0 / self.transform.scale),
											&self.transform,
											&delete_color);
				grid.set_pixel_transformed( pixel::Vec2::new((y + x) / 2.0 - width / 2.0 + 15.0 / self.transform.scale,
															 (y - x) / 2.0 + 40.0 - height / 2.0 + 15.0 / self.transform.scale),
											&self.transform,
											&delete_color);
			}
		}

	}

	fn mouse_input(&mut self, pos: pixel::Vec2, mouse_state: MouseClickState) -> bool {
		let width = WIDTH as f32;
		let height = HEIGHT as f32;
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
				Hoverables::Translate => {
					self.transform.position = pos;
				},
				Hoverables::Scale => {
					let current_distance = pos - self.transform.position;
					self.transform.scale = current_distance.magnitude() * WIDTH as f32 / HEIGHT as f32 / 540.0;
					self.transform.scale = self.transform.scale.max(0.1).min(1.0);
				},
				_ => (),
			}
			match mouse_state {
				MouseClickState::Released => {
					self.grabbing = None;
				},
				_ => (),
			}
			return true;
		}

		if local_pos.x < -width / 2.0 || local_pos.x > width / 2.0 ||
			local_pos.y < -height / 2.0 || local_pos.y > height / 2.0 {
			self.controls_visible = false;
			return false;
		} else {
			self.controls_visible = true;
		}

		{
			if 		local_pos.x > -85.0 &&
					local_pos.x < 85.0 &&
					local_pos.y > -height / 2.0 + 15.0 / self.transform.scale &&
					local_pos.y < -height / 2.0 + 70.0 + 15.0 / self.transform.scale {
				self.hovering = Some(Hoverables::Rotate);
				match mouse_state {
					MouseClickState::Pressed => {
						self.grabbing = Some(Hoverables::Rotate);
					},
					_ => (),
				}
			} else if 	local_pos.x > -65.0 &&
						local_pos.x < 65.0 &&
						local_pos.y > -65.0 &&
						local_pos.y < 65.0 {
				self.hovering = Some(Hoverables::Translate);
				match mouse_state {
					MouseClickState::Pressed => {
						self.grabbing = Some(Hoverables::Translate);
					},
					_ => (),
				}
			} else if 	local_pos.x > width / 2.0 - 85.0 - 15.0 / self.transform.scale &&
						local_pos.x < width / 2.0 - 0.0 - 15.0 / self.transform.scale &&
						local_pos.y > height / 2.0 - 85.0 - 15.0 / self.transform.scale &&
						local_pos.y < height / 2.0 - 0.0 - 15.0 / self.transform.scale {
				self.hovering = Some(Hoverables::Scale);
				match mouse_state {
					MouseClickState::Pressed => {
						self.grabbing = Some(Hoverables::Scale);
						self.scale_start = Some(pos);
					},
					_ => (),
				}
			} else if 	local_pos.x > -width / 2.0 + 15.0 / self.transform.scale &&
						local_pos.x < -width / 2.0 + 50.0 + 15.0 / self.transform.scale &&
						local_pos.y > -height / 2.0 + 15.0 / self.transform.scale &&
						local_pos.y < -height / 2.0 + 50.0 + 15.0 / self.transform.scale {
				self.hovering = Some(Hoverables::Delete);
				match mouse_state {
					MouseClickState::Pressed => {
						self.dead = true;
					},
					_ => (),
				}
			} else {
				self.hovering = None;
			}
		}
		true
	}
}