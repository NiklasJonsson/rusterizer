#![feature(const_generics)]
#![feature(unsized_locals)]
use minifb::{Key, Window, WindowOptions};

use std::time::{Duration, Instant};

mod graphics_primitives;
mod math;
mod rasterizer;

use crate::graphics_primitives::*;
use crate::math::*;
use crate::rasterizer::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;

/*
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
*/

fn get_triangles() -> Vec<Triangle<WorldSpace>> {
    unimplemented!();
}

fn get_view_matrix() -> Mat4<WorldSpace, CameraSpace> {
    unimplemented!();
}

fn main() {
    let triangles = get_triangles();

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

    let v = vec3::<WorldSpace>(0.0, 0.1, 0.2);
    let v1 = vec3::<WorldSpace>(0.0, 0.1, 0.2);
    println!("{:?}", v * 3.0);

    let mut avg = Duration::new(0, 0);
    let mut iterations = 0;

    let view_matrix = get_view_matrix();
    let proj_matrix = project(2.0, 100.0, 2.0);
    let triangles = triangles
        .into_iter()
        .map(|tri| proj_matrix * view_matrix * tri)
        .collect::<Vec<_>>();

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
