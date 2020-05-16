use crate::math;
use crate::texture::Texture;

#[derive(Copy, Clone, Debug)]
pub struct TextureHandle {
    index: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct MatrixHandle {
    index: usize,
}

pub struct Uniforms {
    textures: Vec<Texture>,
    matrices: Vec<math::Mat4<math::WorldSpace, math::WorldSpace>>,
}

impl Uniforms {
    pub fn new() -> Self {
        Uniforms {
            textures: Vec::new(),
            matrices: Vec::new(),
        }
    }

    pub fn bind_texture(&mut self, tex: Texture) -> TextureHandle {
        // This does not allow removing textures
        self.textures.push(tex);
        TextureHandle {
            index: self.textures.len() - 1,
        }
    }

    pub fn get_texture(&self, handle: TextureHandle) -> &Texture {
        &self.textures[handle.index]
    }

    pub fn add_matrix(&mut self) -> MatrixHandle {
        self.matrices
            .push(math::Mat4::<math::WorldSpace>::identity());

        MatrixHandle {
            index: self.matrices.len() - 1,
        }
    }

    pub fn read_matrix(
        &self,
        handle: MatrixHandle,
    ) -> &math::Mat4<math::WorldSpace, math::WorldSpace> {
        &self.matrices[handle.index]
    }

    pub fn write_matrix(
        &mut self,
        handle: MatrixHandle,
    ) -> &mut math::Mat4<math::WorldSpace, math::WorldSpace> {
        &mut self.matrices[handle.index]
    }
}
