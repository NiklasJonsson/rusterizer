#![feature(const_generics)]
#![feature(unsized_locals)]

use std::time::{Duration, Instant};

mod camera;
mod graphics_primitives;
mod math;
mod mesh;
mod rasterizer;
mod render;
mod color;
mod texture;
mod uniform;

use crate::render::*;
use crate::graphics_primitives::VertexAttribute;
use crate::uniform::Uniforms;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;

fn main() {
    let camera = camera::Camera::default();

    let mut renderer = Renderer::new(WIDTH, HEIGHT);

    let mut avg = Duration::new(0, 0);
    let mut iterations = 0;

    let view_matrix = camera.get_view_matrix();
    let proj_matrix = math::project(
        2.0,
        100.0,
        HEIGHT as f32 / WIDTH as f32,
        std::f32::consts::FRAC_PI_2,
    );

    let mesh = mesh::centered_quad(1.0);

    let mut meshes = Vec::new();
    meshes.push(mesh);
    let start = Instant::now();

    let tex = texture::Texture::from_png_file("images/checkerboard.png");

    let checkerboard_handle = renderer.bind_texture(tex);

    loop {
        let t0 = Instant::now();

        let diff = start.elapsed().as_secs_f32();
        let world_matrix = math::transform::rotate::<math::WorldSpace>(diff, diff, 0.0);
        let vertex_shader = move |vertex: &math::Point3D<math::WorldSpace>| {
            proj_matrix * view_matrix * vertex.extend(1.0)
        };

        let fragment_shader = move |uniforms: &Uniforms, attr: &VertexAttribute| uniforms.get_texture(checkerboard_handle).sample(attr.uvs[0], attr.uvs[1]);

        for mesh in &meshes {
            renderer.render(&mesh, vertex_shader, fragment_shader);
        }

        avg = (avg * iterations + t0.elapsed()) / (iterations + 1);
        iterations += 1;

        if iterations % 100 == 0 {
            println!("{:?}", avg);
        }

        match renderer.display() {
            Err(e) => {
                println!("{}", e);
                return;
            }
            Ok(false) => return,
            Ok(true) => (),
        }
    }
}
