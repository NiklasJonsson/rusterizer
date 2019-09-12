use core::ops::Sub;
use minifb::{Key, WindowOptions, Window};

#[derive(Debug, Copy, Clone)]
struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone)]
struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    fn to_rgba(&self) -> [u8; 4] {
        [(self.r * 255.0) as u8,
        (self.g * 255.0) as u8,
        (self.b * 255.0) as u8,
        (self.a * 255.0) as u8,
        ]
    }
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2{x: self.x - other.x, y: self.y - other.y}
    }
}

#[derive(Debug)]
struct Triangle {
    points: [Vec2; 3],
    normals: [Vec2; 3],
    color: Color,
}


fn dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

impl Triangle {
    fn new(points: [Vec2; 3], color: Color) -> Triangle {
        // Clockwise edge equations
        // To have the normals all pointing towards the inner part of the triangle,
        // they all need to have their positive halfspace to the right of the triangle.
        // If we wanted counter-clockwise, then we switch signs on both x and y of normals
        // (and also switch order for v computations above. Note that coordinate system
        // starts in upper left corner.

        let v0 = points[1] - points[0];
        let v1 = points[2] - points[1];
        let v2 = points[0] - points[2];
        let n0 = Vec2::new(-v0.y, v0.x);
        let n1 = Vec2::new(-v1.y, v1.x);
        let n2 = Vec2::new(-v2.y, v2.x);

        let normals = [n0, n1, n2];
        Triangle {points, normals, color}
    }

    fn is_point_inside(&self, point: Vec2) -> bool {
        self.normals.iter().zip(self.points.iter()).fold(true, |acc, (&n, &p)| (dot(n, point-p) >= 0.0) && acc)
    }

    fn get_color(&self) -> Color {
        self.color
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
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                buffer.push(0);
            }
        }
        Self {buffer, width, height}
    }

    fn set_pixel(&mut self, row: usize, col: usize, color: Color) {
        let rgba = color.to_rgba();
        self.buffer[row * self.width + col] = rgba[0] as u32 | (rgba[1] as u32) << 8 | (rgba[2] as u32) << 16 | (rgba[3] as u32) << 24;
    }

    fn get_raw(&self) -> &Vec<u32> {
        &self.buffer
    }
}

const WIDTH: usize = 800;
const HEIGHT: usize = 800;

// Returns RGBA image
fn rasterize(triangles: Vec<Triangle>) -> ColorBuffer {
    let width = WIDTH;
    let height = HEIGHT;
    let mut color_buffer = ColorBuffer::new(width, height);
    for triangle in triangles {
        for i in 0..height {
            for j in 0..width {
                // Sample middle of pixel
                let x = j as f32 + 0.5;
                let y = i as f32 + 0.5;
                let pos = Vec2::new(x, y);
                if triangle.is_point_inside(pos) {
                    color_buffer.set_pixel(i, j, triangle.color);
                }
            }
        }
    }

    color_buffer
}

fn main() {
    let pos0 = Vec2::new(100.0, 100.0);
    let pos1 = Vec2::new(500.0, 100.0);
    let pos2 = Vec2::new(100.0, 300.0);
    let color = Color{r: 0.5, g: 0.5, b: 0.5, a: 0.5};
    let tri = Triangle::new([pos0, pos1, pos2], color);
    let triangles = vec![tri];
    let color_buffer = rasterize(triangles);

    let mut window = Window::new("Test - ESC to exit",
                                 WIDTH,
                                 HEIGHT,
                                 WindowOptions::default()).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(color_buffer.get_raw()).unwrap();
    }
}
