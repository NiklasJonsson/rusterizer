#![feature(const_generics)]
#![feature(unsized_locals)]
#![feature(clamp)]

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

fn choose_shader(args: &Args) -> FragmentShader {
    match args.fs {
        FS::Texture => |uniforms: &Uniforms, _: &rasterizer::FragCoords, attr: &VertexAttribute| {
            uniforms.get_texture(0).sample(attr.uvs[0], attr.uvs[1])
        },
        FS::Color => |_: &Uniforms, _: &rasterizer::FragCoords, attr: &VertexAttribute| attr.color,
        FS::Debug => |_: &Uniforms, frag_coords: &rasterizer::FragCoords, _: &VertexAttribute| {
            Color::grayscale(frag_coords.depths[0])
        },
    }
}

fn main() {
    let args = parse_args();
    let camera = camera::Camera::default();

    let mut renderer = Renderer::new(WIDTH, HEIGHT);

    let mut avg = Duration::new(0, 0);
    let mut iterations = 0;

    {
        let block = renderer.uniforms().write_block();
        block.view = camera.get_view_matrix();
        block.projection = math::project(
            1.0,
            200.0,
            HEIGHT as f32 / WIDTH as f32,
            std::f32::consts::FRAC_PI_2,
        );
    }

    let tex = texture::Texture::from_png_file("images/checkerboard.png");
    renderer.uniforms().bind_texture(0, tex);

    let meshes = [mesh::cube(1.0), mesh::sphere(0.5)];
    let mut matrices = [math::Mat4::<math::WorldSpace>::identity(); 2];

    let vertex_shader = |uniforms: &Uniforms, vertex: &math::Point3D<math::WorldSpace>| {
        uniforms.read_block().projection
            * uniforms.read_block().view
            * uniforms.read_block().world
            * vertex.extend(1.0)
    };

    let fragment_shader = choose_shader(&args);

    let start = Instant::now();
    loop {
        let t0 = Instant::now();

        let diff = start.elapsed().as_secs_f32();

        matrices[0] = math::rotate::<math::WorldSpace>(diff, diff, 0.0);
        matrices[1] = math::rotate::<math::WorldSpace>(diff, 0.0, std::f32::consts::FRAC_PI_4)
            * math::translate::<math::WorldSpace>(0.0, 3.0, 0.0);

        for (mesh, mat) in meshes.iter().zip(matrices.iter()) {
            renderer.uniforms().write_block().world = *mat;
            renderer.render(&mesh, vertex_shader, fragment_shader);
        }

        print!("{:?}", t0.elapsed());

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
