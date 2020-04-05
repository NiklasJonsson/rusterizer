#![feature(const_generics)]
#![feature(unsized_locals)]
use minifb::{Key, Window, WindowOptions};

use std::time::{Duration, Instant};

mod camera;
mod graphics_primitives;
mod math;
mod rasterizer;

use crate::graphics_primitives::*;
use crate::rasterizer::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;

fn get_centered_quad(
    width: f32,
) -> (
    Vec<Vertex<math::WorldSpace>>,
    Vec<VertexAttribute>,
    Vec<usize>,
) {
    let color = Color::blue();
    let vs = vec![
        vertex(-width / 2.0, width / 2.0, 3.0),
        vertex(width / 2.0, width / 2.0, 3.0),
        vertex(width / 2.0, -width / 2.0, 3.0),
        vertex(-width / 2.0, -width / 2.0, 3.0),
    ];

    let attrs = vec![color.into(); 4];

    let indices = vec![0, 1, 2, 0, 2, 3];

    (vs, attrs, indices)
}

fn get_triangles() -> Vec<Triangle<math::WorldSpace>> {
    let pos0 = vertex(1.0, 0.0, 0.0);
    let pos1 = vertex(2.0, 0.0, 0.0);
    let pos2 = vertex(1.0, 1.0, 0.0);
    let color0 = Color::blue();
    let color1 = Color::red();
    let color2 = Color::red();

    let vertex_attributes = [color0.into(), color1.into(), color2.into()];

    let tri = Triangle {
        vertices: [pos0, pos1, pos2],
        vertex_attributes,
    };

    let pos0 = vertex(1.0, 0.0, 3.0);
    let pos1 = vertex(2.0, 0.0, 3.0);
    let pos2 = vertex(1.0, 1.0, 3.0);
    let color0 = Color::red();
    let color1 = Color::green();
    let color2 = Color::green();

    let vertex_attributes = [color0.into(), color1.into(), color2.into()];

    let tri2 = Triangle {
        vertices: [pos0, pos1, pos2],
        vertex_attributes,
    };
    vec![tri, tri2]
}

fn main() {
    let camera = camera::Camera::default();
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

    let mut avg = Duration::new(0, 0);
    let mut iterations = 0;

    let view_matrix = camera.get_view_matrix();
    let proj_matrix = math::project(
        2.0,
        100.0,
        HEIGHT as f32 / WIDTH as f32,
        std::f32::consts::FRAC_PI_2,
    );
    let triangles = triangles
        .into_iter()
        .map(|tri| proj_matrix * view_matrix * tri)
        .collect::<Vec<_>>();

    let (quad_vs, quad_attrs, quad_indices) = get_centered_quad(2.0);
    let quad_vs = quad_vs
        .into_iter()
        .map(|vertex| proj_matrix * view_matrix * vertex)
        .collect::<Vec<_>>();

    dbg!(&quad_vs);

    let draw_quad = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let t0 = Instant::now();
        let color_buffer = if draw_quad {
            rasterizer.draw_indirect(&quad_vs, &quad_attrs, &quad_indices)
        } else {
            rasterizer.draw(&triangles)
        };

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
