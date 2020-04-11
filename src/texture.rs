use std::fs::{File};
use std::path::Path;

use crate::color::Color;

pub struct Texture {
    buf: Vec<u8>,
    width: usize,
    height: usize,
    texel_width: usize,
}

impl Texture {
    pub fn from_png_file(path: impl AsRef<Path>) -> Self {
        let file = File::open(path).expect("Failed to read file");
        let decoder = png::Decoder::new(file);
        let (info, mut reader) = decoder.read_info().expect("Failed to read info");
        dbg!(&info);
        // Allocate the output buffer.
        let mut buf = Vec::with_capacity(info.buffer_size());
        buf.resize(info.buffer_size(), 0);
        // Read the next frame. Currently this function should only called once.
        // The default options
        reader.next_frame(&mut buf).unwrap();
        assert_eq!(info.color_type, png::ColorType::RGBA);
        assert_eq!(info.bit_depth, png::BitDepth::Eight);

        Texture { 
            buf,
            width: info.width as usize,
            height: info.height as usize,
            texel_width: 4,
        }
    }

    pub fn read_texel(&self, x: usize, y: usize) -> Color {
        assert!(self.texel_width == 3 || self.texel_width == 4);
        let texel_start = x * self.texel_width + y * self.texel_width * self.width;
        let mut rgba: [u8; 4] = 
            [self.buf[texel_start],
            self.buf[texel_start + 1],
            self.buf[texel_start + 2],
            255,
            ];
        if self.texel_width == 4 {
            rgba[3] = self.buf[texel_start + 3];
        }

        Color::from_rgba(rgba)

    }


    pub fn sample(&self, u: f32, v: f32) -> Color
    {
        assert!(u >= 0.0 && u <= 1.0, "Incorrect u coordinate!");
        assert!(v >= 0.0 && v <= 1.0, "Incorrect v coordinate!");
        self.read_texel((u * self.width as f32) as usize, (v * self.height as f32) as usize)
    }

}
