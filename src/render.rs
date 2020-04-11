use crate::math;
use crate::mesh::Mesh;
use crate::rasterizer::*;
use crate::graphics_primitives::*;
use crate::color::Color;
use crate::uniform::{Uniforms, UniformHandle};
use crate::texture::Texture;

pub struct Renderer {
    rasterizer: Rasterizer,
    window: minifb::Window,
    uniforms: Uniforms,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Renderer {
        let window = minifb::Window::new(
            "Test - ESC to exit",
            width,
            height,
            minifb::WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        let rasterizer = Rasterizer::new(width, height);

        Self { rasterizer, window, uniforms: Uniforms::new() }
    }

    pub fn bind_texture(&mut self, tex: Texture) -> UniformHandle {
        self.uniforms.bind_texture(tex)
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

    pub fn render<FragmentShader>(
        &mut self,
        mesh: &Mesh<math::WorldSpace>,
        vertex_shader: impl Fn(&math::Point3D<math::WorldSpace>) -> math::Point4D<math::ClipSpace>,
        fragment_shader: FragmentShader,
    )
        where FragmentShader: Fn(&Uniforms, &VertexAttribute) -> Color + Copy
    {
        let vertices: Vec<math::Point4D<math::ClipSpace>> =
            mesh.vertices.iter().map(vertex_shader).collect::<Vec<_>>();

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
