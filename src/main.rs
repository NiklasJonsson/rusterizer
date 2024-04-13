use std::time::Instant;

mod camera;
mod color;
mod graphics_primitives;
mod math;
mod mesh;
mod rasterizer;
mod render;
mod texture;
mod uniform;

use math::WorldSpace;

use crate::color::Color;
use crate::graphics_primitives::VertexAttribute;
use crate::render::*;
use crate::uniform::Uniforms;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

enum FS {
    Texture,
    Color,
    Debug,
}

enum Mode {
    Demo,
    ClipTest,
}

struct Args {
    fs: FS,
    mode: Mode,
}

// Lazy, dependency-free CLI parsing
fn parse_args() -> Args {
    let mut ret = Args {
        fs: FS::Texture,
        mode: Mode::Demo,
    };

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return ret;
    }

    // Only supports flags
    for arg in args.iter().skip(1) {
        if arg == "--color-fs" {
            ret.fs = FS::Color;
        } else if arg == "--debug-fs" {
            ret.fs = FS::Debug;
        } else if arg == "--clip-test" {
            ret.mode = Mode::ClipTest;
        } else {
            panic!("Invalid argument: {arg}");
        }
    }

    ret
}

fn choose_shader(fs: FS) -> FragmentShader {
    match fs {
        FS::Texture => |uniforms: &Uniforms, _: &rasterizer::FragCoords, attr: &VertexAttribute| {
            uniforms.get_texture(0).sample(attr.uvs[0], attr.uvs[1])
        },
        FS::Color => |_: &Uniforms, _: &rasterizer::FragCoords, attr: &VertexAttribute| attr.color,
        FS::Debug => |_: &Uniforms, frag_coords: &rasterizer::FragCoords, _: &VertexAttribute| {
            Color::grayscale(frag_coords.depths[0])
        },
    }
}

struct Scene {
    // matrices and meshes should always be the same length
    matrices: Vec<math::Mat4<math::WorldSpace>>,
    meshes: Vec<mesh::Mesh<WorldSpace>>,
}

struct Time {
    elapsed: std::time::Duration,
}

type Update = Box<dyn Fn(&mut Scene, &Time)>;

fn setup_scene(mode: Mode) -> (Scene, Update) {
    match mode {
        Mode::Demo => {
            let update = |scene: &mut Scene, time: &Time| {
                let elapsed = time.elapsed.as_secs_f32();
                scene.matrices[0] = math::rotate::<math::WorldSpace>(elapsed, elapsed, 0.0);
                scene.matrices[1] =
                    math::rotate::<math::WorldSpace>(elapsed, 0.0, std::f32::consts::FRAC_PI_4)
                        * math::translate::<math::WorldSpace>(0.0, 3.0, 0.0);
            };

            let meshes = vec![mesh::cube(1.0), mesh::sphere(0.5)];
            let matrices = vec![math::Mat4::<math::WorldSpace>::identity(); meshes.len()];
            (Scene { matrices, meshes }, Box::new(update))
        }
        Mode::ClipTest => {
            // Here is some hackery to stress test the clipping
            let meshes = vec![mesh::triangle::<math::WorldSpace>()];
            let matrices = vec![math::Mat4::<math::WorldSpace>::identity(); meshes.len()];
            let update = |scene: &mut Scene, time: &Time| {
                let elapsed = time.elapsed.as_secs_f32();
                let stage_duration = 10.0;
                let n_stages = 2;
                // Create a loop of stages, each getting stage_duration seconds.
                let loop_progress = elapsed % (stage_duration * n_stages as f32);
                let stage = (loop_progress / stage_duration).floor() as u32;
                let stage_progress = loop_progress % stage_duration;
                if stage == 0 {
                    // Triangle that "follows the window border" (rotate around z at approx window width)
                    scene.matrices[0] = math::rotate_z(elapsed)
                    // This is hardcoded based on the current width/height of the window :(
                        * math::translate::<math::WorldSpace>(7.3, 0.0, 0.0)
                        * math::rotate_z(-elapsed);
                } else if stage == 1 {
                    // * Fix the "sign" so that we test both near and far culling
                    let sign = if stage_progress < (stage_duration / 2.0) {
                        -1.0
                    } else {
                        1.0
                    };
                    scene.matrices[0] = math::translate::<math::WorldSpace>(
                        0.0,
                        0.0,
                        20.0 * sign * stage_progress / stage_duration,
                    );
                }

                // START HERE:
                // 1. Improve --clip-test
                // 2. Have another look at the complete_coverage

                // TODO:
                // * Partial near clipping with an angled (in z) triangle
                // * Massive triangle that covers the whole viewport
            };
            (Scene { matrices, meshes }, Box::new(update))
        }
    }
}

fn main() {
    let args = parse_args();
    let camera = camera::Camera::default();

    let mut renderer = Renderer::new(WIDTH, HEIGHT);

    let block = renderer.uniforms().write_block();
    block.view = camera.get_view_matrix();
    block.projection = math::project(
        1.0,
        200.0,
        HEIGHT as f32 / WIDTH as f32,
        std::f32::consts::FRAC_PI_2,
    );

    let tex = texture::Texture::from_png_file("images/checkerboard.png");
    renderer.uniforms().bind_texture(0, tex);

    let vertex_shader = |uniforms: &Uniforms, vertex: &math::Point3D<math::WorldSpace>| {
        uniforms.read_block().projection
            * uniforms.read_block().view
            * uniforms.read_block().world
            * vertex.extend(1.0)
    };

    let fragment_shader = choose_shader(args.fs);
    let (mut scene, update) = setup_scene(args.mode);

    let start = Instant::now();
    let mut now = Instant::now();
    loop {
        renderer.display_frame_time(&now.elapsed());
        now = Instant::now();

        update(
            &mut scene,
            &Time {
                elapsed: start.elapsed(),
            },
        );

        for (mesh, mat) in scene.meshes.iter().zip(scene.matrices.iter()) {
            renderer.uniforms().write_block().world = *mat;
            renderer.render(mesh, vertex_shader, fragment_shader);
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
