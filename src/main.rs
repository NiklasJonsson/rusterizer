use minifb::{Key, Window, WindowOptions};

use std::time::{Duration, Instant};

use std::f32;
use core::ops::Mul;
use core::ops::Add;

mod math_primitives;

use crate::math_primitives::*;

#[derive(Debug, Copy, Clone)]
struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    fn to_rgba(&self) -> u32 {
        (self.r * 255.0) as u32
            | ((self.g * 255.0) as u32) << 8
            | ((self.b * 255.0) as u32) << 16
            | ((self.a * 255.0) as u32) << 24
    }

    fn to_bgra(&self) -> u32 {
        (self.b * 255.0) as u32
            | ((self.g * 255.0) as u32) << 8
            | ((self.r * 255.0) as u32) << 16
            | ((self.a * 255.0) as u32) << 24
    }

    fn red() -> Color {
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
    fn green() -> Color {
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }
    fn blue() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color {r: self.r * scalar, g: self.g * scalar, b: self.b * scalar, a: self.a * scalar}
    }
}

impl Add<Color> for Color {
    type Output = Color;
    fn add(self, other: Color) -> Color {
        Color {r: self.r + other.r, g: self.g + other.g, b: self.b + other.b, a: self.a + other.a}
    }
}

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
                (a.0.min(p.x), a.1.max(p.x), a.2.min(p.y), a.3.max(p.y))
            });
        let (min_x, max_x, min_y, max_y) = (
            vals.0.floor() as usize,
            vals.1.ceil() as usize,
            vals.2.floor() as usize,
            vals.3.ceil() as usize,
        );
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
}

#[derive(Debug)]
struct VertexAttribute {
    color: Color,
}

#[derive(Debug)]
struct Triangle {
    pub vertices: [Point2D; 3],
    normals: [Vec2D; 3],
    pub vertex_attributes: [VertexAttribute; 3],
    area: f32,
}

impl Triangle {
    fn area(vertices: &[Point2D; 3]) -> f32 {
        (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]) * 0.5
    }

    fn new(vertices: [Point2D; 3], vertex_attributes: [VertexAttribute; 3]) -> Triangle {
        // Clockwise edge equations
        // To have the normals all pointing towards the inner part of the triangle,
        // they all need to have their positive halfspace to the right of the triangle.
        // If we wanted counter-clockwise, then we switch signs on both x and y of normals
        // (and also switch order for v computations above. Note that coordinate system
        // starts in upper left corner.

        let v0 = vertices[1] - vertices[0];
        let v1 = vertices[2] - vertices[1];
        let v2 = vertices[0] - vertices[2];
        let n0 = Vec2D::new(-v0.y, v0.x);
        let n1 = Vec2D::new(-v1.y, v1.x);
        let n2 = Vec2D::new(-v2.y, v2.x);

        let normals = [n0, n1, n2];
        let area = Triangle::area(&vertices);

        Triangle {
            vertices,
            normals,
            vertex_attributes,
            area,
        }
    }

    fn is_point_inside(&self, point: Point2D) -> bool {
        // Based on edge equations
        self.normals
            .iter()
            .zip(self.vertices.iter())
            .fold(true, |acc, (&n, &p)| (n.dot(point - p) >= 0.0) && acc)
    }

    fn interpolate_color_for(&self, point: Point2D) -> Color {
        assert!(self.is_point_inside(point));
        let barycentric0 = Triangle::area(&[self.vertices[1], self.vertices[2], point]) / self.area;
        let barycentric1 = Triangle::area(&[self.vertices[2], self.vertices[0], point]) / self.area;
        let barycentric2 = 1.0 - barycentric0 - barycentric1;

        self.vertex_attributes[0].color * barycentric0 + self.vertex_attributes[1].color * barycentric1 + self.vertex_attributes[2].color * barycentric2
    }
}

// RGBA image
struct ColorBuffer {
    buffer: Vec<u32>,
    width: usize,
    height: usize,
}

impl ColorBuffer {
    fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        // Initialize to black
        for _i in 0..width * height {
            buffer.push(0);
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
            self.buffer[i] = 0;
        }
    }

    fn set_pixel(&mut self, row: usize, col: usize, color: Color) {
        self.buffer[row * self.width + col] = color.to_bgra();
    }

    fn get_raw(&self) -> &Vec<u32> {
        &self.buffer
    }
}

const WIDTH: usize = 800;
const HEIGHT: usize = 800;

struct Rasterizer {
    color_buffer: ColorBuffer,
}

impl Rasterizer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            color_buffer: ColorBuffer::new(width, height),
        }
    }

    // Returns RGBA image
    fn rasterize(&mut self, triangles: &[Triangle]) -> &ColorBuffer {
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

fn get_triangle() -> Vec<Triangle> {
    let pos0 = Point2D::new(300.0, 100.0);
    let pos1 = Point2D::new(400.0, 300.0);
    let pos2 = Point2D::new(200.0, 300.0);
    let color0 = Color::red();
    let color1 = Color::green();
    let color2 = Color::blue();

    let vertex_attributes = [VertexAttribute{color: color0},
                             VertexAttribute{color: color1},
                             VertexAttribute{color: color2}];

    let tri = Triangle::new([pos0, pos1, pos2], vertex_attributes);
    vec![tri]
}

/*
fn get_triangles() -> Vec<Triangle> {
    let mut triangles = Vec::new();

    for i in (0..600).step_by(60) {
        let pos0 = Point2D::new((100 + i) as f32, 200.0);
        let pos1 = Point2D::new((100 + i + 50) as f32, 200 as f32);
        let pos2 = Point2D::new((100 + i) as f32, 400.0);
        let color = Color {
            r: (i + 100) as f32 / 700.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        triangles.push(Triangle::new([pos0, pos1, pos2], color));
    }

    triangles
}
*/

fn main() {
    let triangles = get_triangle();
    //let triangles = get_triangles();

    let mut rasterizer = Rasterizer::new(WIDTH, HEIGHT);

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut avg = Duration::new(0, 0);
    let mut iterations = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let t0 = Instant::now();
        let color_buffer = rasterizer.rasterize(&triangles);
        avg = (avg * iterations + t0.elapsed()) / (iterations + 1);
        iterations += 1;

        if iterations % 100 == 0 {
            println!("{:?}", avg);
        }

        match window.update_with_buffer(color_buffer.get_raw()) {
            Err(e) => {
                println!("{}", e);
                return;
            }
            Ok(_) => (),
        }
    }
}
