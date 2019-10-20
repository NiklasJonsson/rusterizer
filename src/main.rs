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

fn get_triangles() -> Vec<Triangle<WorldSpace>> {
    let pos0 = vertex(0.3, 0.1, -3.0);
    let pos1 = vertex(0.4, 0.3, -3.0);
    let pos2 = vertex(0.2, 0.3, -3.0);
    let color0 = Color::red();
    let color1 = Color::red();
    let color2 = Color::red();

    let vertex_attributes = [color0.into(), color1.into(), color2.into()];

    let tri = Triangle {
        vertices: [pos0, pos1, pos2],
        vertex_attributes,
    };

    // Triangle 2, slightly shifted to the left
    let pos0 = vertex(0.4, 0.1, -2.0);
    let pos1 = vertex(0.5, 0.3, -2.0);
    let pos2 = vertex(0.3, 0.3, -2.0);
    let color0 = Color::green();
    let color1 = Color::green();
    let color2 = Color::green();

    let vertex_attributes = [color0.into(), color1.into(), color2.into()];

    let tri2 = Triangle {
        vertices: [pos0, pos1, pos2],
        vertex_attributes,
    };
    vec![tri, tri2]
}

struct Camera {
    pos: Point4D<WorldSpace>,
    up: Vec4<WorldSpace>,
    dir: Vec4<WorldSpace>,
}

impl Camera {
    fn get_view_matrix(&self) -> Mat4<WorldSpace, CameraSpace> {
        // cam_transform = T * R, view = inverse(cam_transform) = inv(R) * inv(T)

        let cam_forward = -self.dir;
        let cam_right = self.up.cross(cam_forward).normalize();
        let cam_up = cam_forward.cross(cam_right).normalize();

        let rotation_inv = mat4(
            cam_right.x(),
            cam_right.y(),
            cam_right.z(),
            0.0,
            cam_up.x(),
            cam_up.y(),
            cam_up.z(),
            0.0,
            cam_forward.x(),
            cam_forward.y(),
            cam_forward.z(),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );

        let vec_to_pos = self.pos - math::point::origin();

        let translation_inv = transform::translation_along(-vec_to_pos);
        rotation_inv * translation_inv
    }
}

fn get_camera() -> Camera {
    let pos = Point4D::<WorldSpace>::new(0.0, 0.0, 2.0, 1.0);
    let up = vec4::<WorldSpace>(0.0, 1.0, 0.0, 0.0).normalize();
    let dir = vec4::<WorldSpace>(0.0, 0.0, -1.0, 0.0).normalize();

    Camera { pos, up, dir }
}

fn main() {
    let camera = get_camera();
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
    let proj_matrix = project(
        2.0,
        100.0,
        HEIGHT as f32 / WIDTH as f32,
        std::f32::consts::FRAC_PI_2,
    );
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
