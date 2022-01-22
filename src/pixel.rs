

use crate::{WIDTH, HEIGHT};

pub struct PixelGrid<'a>(pub &'a mut [u8]);

impl<'a> PixelGrid<'a> {
    pub fn set_pixel(&mut self, point: Vec2, pixel: &[u8; 4]) {
        let x = point.x.clamp(0.0, (WIDTH - 1) as f32) as u32;
        let y = point.y.clamp(0.0, (HEIGHT - 1) as f32) as u32;
        let i = (x as usize + (y * WIDTH) as usize) * 4;
        self.0[i..i + 4].copy_from_slice(pixel);
    }

    pub fn set_pixel_transformed(&mut self, point: Vec2, transform: &Transform, pixel: &[u8; 4]) {
    	let point = transform.apply(point);
        let x = point.x.clamp(0.0, (WIDTH - 1) as f32) as u32;
        let y = point.y.clamp(0.0, (HEIGHT - 1) as f32) as u32;
        let i = (x as usize + (y * WIDTH) as usize) * 4;
        self.0[i..i + 4].copy_from_slice(&[pixel[0], pixel[1], pixel[2],
        	(((pixel[3] as f32 / 256.0) * (transform.alpha as f32 / 256.0)) * 256.0) as u8]);
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
}

// Vec2
#[derive(Copy, Clone)]
pub struct Vec2 {
    x: f32,
    y: f32,
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