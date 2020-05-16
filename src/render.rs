use crate::color::Color;
use crate::graphics_primitives::*;
use crate::math;
use crate::mesh::Mesh;
use crate::rasterizer::*;
use crate::uniform::Uniforms;

// Debug
fn dump_vertices<CS: math::CoordinateSystem, const N: usize>(
    vertices: &[math::Point<CS, { N }>],
    fname: &str,
) {
    use std::io::Write;
    let mut file = std::fs::File::create(fname).expect("failed to open file");
    for (i, v) in vertices.iter().enumerate() {
        file.write_fmt(format_args!("{}: ({}, {}, {})\n", i, v.x(), v.y(), v.z()));
    }
}

fn dump_indices(indices: &[usize]) {
    use std::io::Write;
    let mut file = std::fs::File::create("index.txt").expect("failed to open file");
    for (i, tri) in indices.chunks(3).enumerate() {
        file.write_fmt(format_args!(
            "{}: ({}, {}, {})\n",
            i, tri[0], tri[1], tri[2]
        ));
    }
}

pub struct Renderer {
    rasterizer: Rasterizer,
    window: minifb::Window,
    uniforms: Uniforms,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Renderer {
        let window = minifb::Window::new(
            "Rusterizer - ESC to exit",
            width,
            height,
            minifb::WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        let rasterizer = Rasterizer::new(width, height);

        Self {
            rasterizer,
            window,
            uniforms: Uniforms::new(),
        }
    }

    pub fn uniforms(&mut self) -> &mut Uniforms {
        &mut self.uniforms
    }

    fn primitive_assembly(
        vertex_buf: &[math::Point4D<math::ClipSpace>],
        attr_buf: &[VertexAttribute],
        idx_buf: &[usize],
    ) -> Vec<Triangle<math::ClipSpace>> {
        let mut triangles = Vec::with_capacity(idx_buf.len() / 3);
        for idxs in idx_buf.chunks(3) {
            let vertices = [
                vertex_buf[idxs[0]],
                vertex_buf[idxs[1]],
                vertex_buf[idxs[2]],
            ];
            let vertex_attributes = [attr_buf[idxs[0]], attr_buf[idxs[1]], attr_buf[idxs[2]]];

            triangles.push(Triangle {
                vertices,
                vertex_attributes,
            });
        }

        triangles
    }

    pub fn render<FragmentShader, VertexShader>(
        &mut self,
        mesh: &Mesh<math::WorldSpace>,
        vertex_shader: VertexShader,
        fragment_shader: FragmentShader,
    ) where
        VertexShader:
            Fn(&Uniforms, &math::Point3D<math::WorldSpace>) -> math::Point4D<math::ClipSpace>,
        FragmentShader: Fn(&Uniforms, &FragCoords, &VertexAttribute) -> Color + Copy,
    {
        let vertices: Vec<math::Point4D<math::ClipSpace>> = mesh
            .vertices
            .iter()
            .map(|v| vertex_shader(&self.uniforms, v))
            .collect::<Vec<_>>();

        let tris = Renderer::primitive_assembly(&vertices, &mesh.attributes, &mesh.indices);

        self.rasterizer
            .rasterize(&tris, &self.uniforms, fragment_shader);
    }

    pub fn display(&mut self) -> minifb::Result<bool> {
        if !self.window.is_open() || self.window.is_key_down(minifb::Key::Escape) {
            return Ok(false);
        }

        let color_buffer = self.rasterizer.swap_buffers();

        self.window.update_with_buffer(color_buffer.get_raw())?;

        Ok(true)
    }
}
