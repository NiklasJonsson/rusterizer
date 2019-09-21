#![feature(const_generics)]
use minifb::{Key, Window, WindowOptions};

use std::time::{Duration, Instant};

mod graphics_primitives;
mod math_primitives;
mod rasterizer;

use crate::graphics_primitives::*;
use crate::math_primitives::*;
use crate::rasterizer::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;

fn get_triangle() -> Vec<Triangle> {
    let pos0 = Point2D::new(300.0, 100.0);
    let pos1 = Point2D::new(400.0, 300.0);
    let pos2 = Point2D::new(200.0, 300.0);
    let color0 = Color::red();
    let color1 = Color::green();
    let color2 = Color::blue();

    let vertex_attributes = [color0.into(), color1.into(), color2.into()];

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

        if let Err(e) = window.update_with_buffer(color_buffer.get_raw()) {
            println!("{}", e);
            return;
        }
    }
}
