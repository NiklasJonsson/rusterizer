use crate::math::{CameraSpace, ClipSpace, Mat4, WorldSpace};
use crate::texture::Texture;

#[derive(Clone, Debug)]
pub struct UniformBlock {
    pub world: Mat4<WorldSpace>,
    pub view: Mat4<WorldSpace, CameraSpace>,
    pub projection: Mat4<CameraSpace, ClipSpace>,
}

#[derive(Clone, Debug)]
pub struct Uniforms {
    textures: Vec<Texture>,
    uniform_block: UniformBlock,
}

impl Uniforms {
    pub fn new() -> Self {
        Uniforms {
            textures: Vec::new(),
            uniform_block: UniformBlock {
                world: Mat4::<WorldSpace>::identity(),
                view: Mat4::<WorldSpace, CameraSpace>::identity(),
                projection: Mat4::<CameraSpace, ClipSpace>::identity(),
            },
        }
    }

    pub fn bind_texture(&mut self, index: usize, tex: Texture) {
        // TODO: Proper support for arbitrary (needs remapping vec)
        assert!(self.textures.len() == index);
        self.textures.push(tex);
    }

    pub fn get_texture(&self, index: usize) -> &Texture {
        &self.textures[index]
    }

    pub fn read_block(&self) -> &UniformBlock {
        &self.uniform_block
    }

    pub fn write_block(&mut self) -> &mut UniformBlock {
        &mut self.uniform_block
    }
}
