use crate::graphics_primitives::*;
use crate::math_primitives::*;

use std::f32;

struct PixelBoundingBox {
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
}

impl From<&[Point2D; 3]> for PixelBoundingBox {
    fn from(vertices: &[Point2D; 3]) -> Self {
        let vals = vertices
            .iter()
            .fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |a, p| {
                (
                    a.0.min(p.x()),
                    a.1.max(p.x()),
                    a.2.min(p.y()),
                    a.3.max(p.y()),
                )
            });
        // Convert the min/max bounds into pixel coordinates. Always round
        // away from the center of the box.
        let (min_x, max_x, min_y, max_y) = (
            vals.0.floor() as usize,
            vals.1.ceil() as usize,
            vals.2.floor() as usize,
            vals.3.ceil() as usize,
        );
        assert!(min_x < max_x && min_y < max_y);
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ColorBufferFormat {
    RGBA,
    BGRA,
}

// RGBA image
#[derive(Debug)]
pub struct ColorBuffer {
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    format: ColorBufferFormat,
}

impl ColorBuffer {
    pub fn new(width: usize, height: usize, format: ColorBufferFormat) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        // Initialize to black
        for _i in 0..width * height {
            buffer.push(0);
        }
        Self {
            buffer,
            width,
            height,
            format,
        }
    }

    // Clear to black
    fn clear(&mut self) {
        assert_eq!(self.buffer.len(), self.height * self.width);
        for i in 0..self.width * self.height {
            self.buffer[i] = 0;
        }
    }

    fn set_pixel(&mut self, row: usize, col: usize, color: Color) {
        match self.format {
            ColorBufferFormat::BGRA => self.buffer[row * self.width + col] = color.to_bgra(),
            ColorBufferFormat::RGBA => self.buffer[row * self.width + col] = color.to_rgba(),
        }
    }

    pub fn get_raw(&self) -> &Vec<u32> {
        &self.buffer
    }
}

#[derive(Debug)]
pub struct DepthBuffer {
    buffer: Vec<f32>,
    width: usize,
    height: usize,
}

impl DepthBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        // Initialize to max depth => everything will be in front
        for _i in 0..width * height {
            buffer.push(f32::MAX);
        }
        Self {
            buffer,
            width,
            height,
        }
    }

    // Clear to black
    fn clear(&mut self) {
        assert_eq!(self.buffer.len(), self.height * self.width);
        for i in 0..self.width * self.height {
            self.buffer[i] = f32::MAX;
        }
    }

    fn get_depth_at(&self, row: usize, col: usize) -> f32 {
        self.buffer[row * self.width + col]
    }
}

pub struct Rasterizer {
    color_buffer: ColorBuffer,
    depth_buffer: DepthBuffer,
}

impl Rasterizer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            color_buffer: ColorBuffer::new(width, height, ColorBufferFormat::BGRA),
            depth_buffer: DepthBuffer::new(width, height),
        }
    }

    // Returns RGBA image
    pub fn rasterize(&mut self, triangles: &[Triangle]) -> &ColorBuffer {
        self.color_buffer.clear();
        for triangle in triangles {
            let bounding_box = PixelBoundingBox::from(&triangle.vertices);
            for i in bounding_box.min_y..bounding_box.max_y {
                for j in bounding_box.min_x..bounding_box.max_x {
                    // Sample middle of pixel
                    let x = j as f32 + 0.5;
                    let y = i as f32 + 0.5;
                    let pos = Point2D::new(x, y);
                    if triangle.is_point_inside(pos) {
                        let col = triangle.interpolate_color_for(pos);
                        self.color_buffer.set_pixel(i, j, col);
                    }
                }
            }
        }

        &self.color_buffer
    }
}
