use crate::math;
use crate::mesh::Mesh;
use crate::rasterizer::*;

pub struct Renderer {
    rasterizer: Rasterizer,
    window: minifb::Window,
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

        Self { rasterizer, window }
    }

    pub fn render(
        &mut self,
        mesh: &Mesh<math::WorldSpace>,
        vertex_shader: impl FnMut(&math::Point3D<math::WorldSpace>) -> math::Point4D<math::ClipSpace>,
    ) {
        let vertices: Vec<math::Point4D<math::ClipSpace>> =
            mesh.vertices.iter().map(vertex_shader).collect::<Vec<_>>();

        self.rasterizer
            .draw_indirect(&vertices, &mesh.attributes, &mesh.indices);
    }

    pub fn display(&mut self) -> minifb::Result<()> {
        if !self.window.is_open() || self.window.is_key_down(minifb::Key::Escape) {
            return Err(minifb::Error::UpdateFailed(
                "Either not open or ESC is pressed!".to_string(),
            ));
        }

        let color_buffer = self.rasterizer.swap_buffers();

        self.window.update_with_buffer(color_buffer.get_raw())?;

        Ok(())
    }
}
