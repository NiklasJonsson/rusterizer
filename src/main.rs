#![feature(const_generics)]
#![feature(unsized_locals)]

use std::time::{Duration, Instant};

mod camera;
mod color;
mod graphics_primitives;
mod math;
mod mesh;
mod rasterizer;
mod render;
mod texture;
mod uniform;

use crate::color::Color;
use crate::graphics_primitives::VertexAttribute;
use crate::render::*;
use crate::uniform::Uniforms;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;

enum FS {
    Texture,
    Color,
    Debug,
}

struct Args {
    fs: FS,
}

fn parse_args() -> Args {
    let ret = Args { fs: FS::Texture };

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return ret;
    }

    if args[1] == "--color-fs" {
        return Args { fs: FS::Color };
    }

    if args[1] == "--debug-fs" {
        return Args { fs: FS::Debug };
    }

    ret
}

fn main() {
    let args = parse_args();
    let camera = camera::Camera::default();

    let mut renderer = Renderer::new(WIDTH, HEIGHT);

    let mut avg = Duration::new(0, 0);
    let mut iterations = 0;

    let view_matrix = camera.get_view_matrix();
    let proj_matrix = math::project(
        1.0,
        200.0,
        HEIGHT as f32 / WIDTH as f32,
        std::f32::consts::FRAC_PI_2,
    );

    let mesh = mesh::centered_quad(2.0);

    let mut meshes = Vec::new();
    meshes.push(mesh);
    let start = Instant::now();

    let tex = texture::Texture::from_png_file("images/checkerboard.png");

    let checkerboard_handle = renderer.bind_texture(tex);
    let fs_tex = move |uniforms: &Uniforms, _: &rasterizer::FragCoords, attr: &VertexAttribute| {
        uniforms
            .get_texture(checkerboard_handle)
            .sample(attr.uvs[0], attr.uvs[1])
    };

    let fs_color =
        move |uniforms: &Uniforms, _: &rasterizer::FragCoords, attr: &VertexAttribute| attr.color;
    let fs_debug = move |uniforms: &Uniforms,
                         frag_coords: &rasterizer::FragCoords,
                         attr: &VertexAttribute| Color::grayscale(attr.uvs[1]);
    loop {
        let t0 = Instant::now();

        let diff = start.elapsed().as_secs_f32();
        let _world_matrix = math::transform::rotate::<math::WorldSpace>(diff, diff, 0.0);
        let w = math::transform::rotate::<math::WorldSpace>(0.0, 0.0, diff);
        let vertex_shader = move |vertex: &math::Point3D<math::WorldSpace>| {
            proj_matrix * view_matrix * vertex.extend(1.0)
        };

        for mesh in &meshes {
            match args.fs {
                FS::Texture => renderer.render(&mesh, vertex_shader, fs_tex),
                FS::Color => renderer.render(&mesh, vertex_shader, fs_color),
                FS::Debug => renderer.render(&mesh, vertex_shader, fs_debug),
            }
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
