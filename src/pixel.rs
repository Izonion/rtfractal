

use crate::{WIDTH, HEIGHT};

pub struct PixelGrid<'a>(pub &'a mut [u8]);

impl<'a> PixelGrid<'a> {
	pub fn set_pixel(&mut self, point: Vec2, pixel: &[u8; 3]) {
		let x = point.x.clamp(0.0, (WIDTH - 1) as f32) as u32;
		let y = point.y.clamp(0.0, (HEIGHT - 1) as f32) as u32;
		let i = (x as usize + (y * WIDTH) as usize) * 4;
		self.0[i..i + 3].copy_from_slice(pixel);
	}

	pub fn set_pixel_transformed(&mut self, point: Vec2, transform: &Transform, pixel: &[u8; 3]) {
		let point = transform.apply(point);
		let x = point.x.clamp(0.0, (WIDTH - 1) as f32) as u32;
		let y = point.y.clamp(0.0, (HEIGHT - 1) as f32) as u32;
		let i = (x as usize + (y * WIDTH) as usize) * 4;
		let alpha = transform.alpha as f32 / 255.0;
		self.0[i + 0] = (((self.0[i + 0] as f32 / 255.0) * (1.0 - alpha) + (pixel[0] as f32 / 255.0) * alpha) * 255.0) as u8;
		self.0[i + 1] = (((self.0[i + 1] as f32 / 255.0) * (1.0 - alpha) + (pixel[1] as f32 / 255.0) * alpha) * 255.0) as u8;
		self.0[i + 2] = (((self.0[i + 2] as f32 / 255.0) * (1.0 - alpha) + (pixel[2] as f32 / 255.0) * alpha) * 255.0) as u8;
	}
}

pub struct Transform {
	pub position: Vec2,
	pub rotation: f32,
	pub scale: f32,
	pub alpha: u8,
}

impl Transform {
	fn apply(&self, point: Vec2) -> Vec2 {
		let point = point * self.scale;
		let point = point.rotate(self.rotation);
		let point = point + self.position;
		point
	}

	pub fn apply_inverse(&self, point: Vec2) -> Vec2 {
		let point = point - self.position;
		let point = point.rotate(-self.rotation);
		let point = point / self.scale;
		point
	}
}

// Vec2
#[derive(Copy, Clone)]
pub struct Vec2 {
	pub x: f32,
	pub y: f32,
}

impl Vec2 {
	pub fn new(x: f32, y: f32) -> Self {
		Self {x, y}
	}

	fn rotate(&self, angle: f32) -> Self {
		let sin = angle.sin();
		let cos = angle.cos();
		Self {
			x: self.x * cos - self.y * sin,
			y: self.y * cos + self.x * sin,
		}
	}

	fn normalized(&self) -> Self {
		let magnitude = self.magnitude();
		Self {
			x: self.x / magnitude,
			y: self.y / magnitude,
		}
	}

	pub fn magnitude(&self) -> f32 {
		(self.x * self.x + self.y * self.y).sqrt()
	}
}

impl std::ops::Mul<Self> for Vec2 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self {
		let x = self.x * rhs.x;
		let y = self.y * rhs.y;
		Self::new(x, y)
	}
}

impl std::ops::Mul<f32> for Vec2 {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self {
		let x = self.x * rhs;
		let y = self.y * rhs;
		Self::new(x, y)
	}
}

impl std::ops::Div<Self> for Vec2 {
	type Output = Self;

	fn div(self, rhs: Self) -> Self {
		let x = self.x / rhs.x;
		let y = self.y / rhs.y;
		Self::new(x, y)
	}
}

impl std::ops::Div<f32> for Vec2 {
	type Output = Self;

	fn div(self, rhs: f32) -> Self {
		let x = self.x / rhs;
		let y = self.y / rhs;
		Self::new(x, y)
	}
}

impl std::ops::Sub for Vec2 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		let x = self.x - rhs.x;
		let y = self.y - rhs.y;
		Self::new(x, y)
	}
}

impl std::ops::Add for Vec2 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		let x = self.x + rhs.x;
		let y = self.y + rhs.y;
		Self::new(x, y)
	}
}