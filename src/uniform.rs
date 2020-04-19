use crate::texture::Texture;

#[derive(Copy, Clone, Debug)]
pub enum UniformHandle {
    Texture { index: usize },
}

pub struct Uniforms {
    textures: Vec<Texture>,
}

impl Uniforms {
    pub fn new() -> Self {
        Uniforms {
            textures: Vec::new(),
        }
    }

    pub fn bind_texture(&mut self, tex: Texture) -> UniformHandle {
        // This does not allow removing textures
        self.textures.push(tex);
        UniformHandle::Texture {
            index: self.textures.len() - 1,
        }
    }

    pub fn get_texture(&self, handle: UniformHandle) -> &Texture {
        match handle {
            UniformHandle::Texture { index } => &self.textures[index],
        }
    }
}
