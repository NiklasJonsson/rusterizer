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

    let meshes = [mesh::cube(1.0), mesh::sphere(0.5)];
    let mut matrices = [math::Mat4::<math::WorldSpace>::identity(); 2];
    let start = Instant::now();

    let tex = texture::Texture::from_png_file("images/checkerboard.png");

    let checkerboard_handle = renderer.uniforms().bind_texture(tex);
    let fs_tex = move |uniforms: &Uniforms, _: &rasterizer::FragCoords, attr: &VertexAttribute| {
        uniforms
            .get_texture(checkerboard_handle)
            .sample(attr.uvs[0], attr.uvs[1])
    };

    let fs_color =
        move |_: &Uniforms, _: &rasterizer::FragCoords, attr: &VertexAttribute| attr.color;
    let fs_debug =
        move |_: &Uniforms, frag_coords: &rasterizer::FragCoords, _: &VertexAttribute| {
            Color::grayscale(frag_coords.depth)
        };

    let world_handle = renderer.uniforms().add_matrix();

    let vertex_shader = move |uniforms: &Uniforms, vertex: &math::Point3D<math::WorldSpace>| {
        proj_matrix * view_matrix * *uniforms.read_matrix(world_handle) * vertex.extend(1.0)
    };
    loop {
        let t0 = Instant::now();

        let diff = start.elapsed().as_secs_f32();

        matrices[0] = math::rotate::<math::WorldSpace>(diff, diff, 0.0);
        matrices[1] = math::rotate::<math::WorldSpace>(diff, 0.0, std::f32::consts::FRAC_PI_4)
            * math::translate::<math::WorldSpace>(0.0, 3.0, 0.0);

        for (mesh, mat) in meshes.iter().zip(matrices.iter()) {
            *renderer.uniforms().write_matrix(world_handle) = *mat;
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
