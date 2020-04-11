use core::ops::Add;
use core::ops::Mul;

#[derive(Debug, Copy, Clone, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn to_rgba(&self) -> u32 {
        (self.r * 255.0) as u32
            | ((self.g * 255.0) as u32) << 8
            | ((self.b * 255.0) as u32) << 16
            | ((self.a * 255.0) as u32) << 24
    }

    pub fn to_bgra(&self) -> u32 {
        (self.b * 255.0) as u32
            | ((self.g * 255.0) as u32) << 8
            | ((self.r * 255.0) as u32) << 16
            | ((self.a * 255.0) as u32) << 24
    }

    pub fn to_argb(&self) -> u32 {
        (self.a * 255.0) as u32
            | ((self.r * 255.0) as u32) << 8
            | ((self.g * 255.0) as u32) << 16
            | ((self.b * 255.0) as u32) << 24
    }

    /* Untested
    pub fn from_rgba(rgba: u32) -> Self {
        Color {
            r: ((rgba & 0xF) as f32) / 255.0,
            g: (((rgba & 0xF0) << 8) as f32) / 255.0,
            b: (((rgba & 0xF00) << 16) as f32) / 255.0,
            a: (((rgba & 0xF000) << 24) as f32) / 255.0,
        }
    }
    */

    pub fn from_rgba(rgba: [u8; 4]) -> Self {
        Color {
            r: (rgba[0] as f32) / 255.0,
            g: (rgba[1] as f32) / 255.0,
            b: (rgba[2] as f32) / 255.0,
            a: (rgba[3] as f32) / 255.0,
        }
    }


    pub fn red() -> Self {
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub fn green() -> Self {
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub fn blue() -> Self {
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn white() -> Self {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
            a: self.a * scalar,
        }
    }
}

impl Add<Color> for Color {
    type Output = Color;
    fn add(self, other: Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a,
        }
    }
}
