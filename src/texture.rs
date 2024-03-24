use std::fs::File;
use std::path::Path;

use crate::color::Color;

// (0, 0) is upper left corner
#[derive(Clone)]
pub struct Texture {
    buf: Vec<u8>,
    width: usize,
    height: usize,
    texel_width: usize,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Texture ({} channels), w: {}, h: {}",
            self.texel_width, self.width, self.height
        )
    }
}

impl Texture {
    pub fn from_png_file(path: impl AsRef<Path>) -> Self {
        let file = File::open(path).expect("Failed to read file");
        let decoder = png::Decoder::new(file);
        let (info, mut reader) = decoder.read_info().expect("Failed to read info");
        debug_assert!(!reader.info().interlaced);
        // Allocate the output buffer.
        let mut buf = vec![0; info.buffer_size()];
        // Read the next frame. Currently this function should only called once.
        // The default options
        reader.next_frame(&mut buf).unwrap();
        debug_assert_eq!(info.color_type, png::ColorType::RGBA);
        debug_assert_eq!(info.bit_depth, png::BitDepth::Eight);

        Texture {
            buf,
            width: info.width as usize,
            height: info.height as usize,
            texel_width: 4,
        }
    }

    pub fn read_texel(&self, x: usize, y: usize) -> Color {
        debug_assert!(self.texel_width == 3 || self.texel_width == 4);
        debug_assert!(x < self.width, "x: {}", x);
        debug_assert!(y < self.height, "y: {}", y);
        let texel_start = x * self.texel_width + y * self.texel_width * self.width;
        let mut rgba: [u8; 4] = [
            self.buf[texel_start],
            self.buf[texel_start + 1],
            self.buf[texel_start + 2],
            255,
        ];
        if self.texel_width == 4 {
            rgba[3] = self.buf[texel_start + 3];
        }

        Color::from_rgba(rgba)
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        debug_assert!((0.0..=1.0).contains(&u), "Incorrect u coordinate: {}", u);
        debug_assert!((0.0..=1.0).contains(&v), "Incorrect v coordinate: {}", v);
        let x = u * (self.width - 1) as f32;
        let y = v * (self.height - 1) as f32;

        let topleft = self.read_texel(x.floor() as usize, y.floor() as usize);
        let topright = self.read_texel(x.ceil() as usize, y.floor() as usize);
        let botleft = self.read_texel(x.floor() as usize, y.ceil() as usize);
        let botright = self.read_texel(x.ceil() as usize, y.ceil() as usize);

        let x_f = x.fract();
        let y_f = y.fract();

        let y0 = topleft * (1.0 - x_f) + topright * x_f;
        let y1 = botleft * (1.0 - x_f) + botright * x_f;

        y0 * (1.0 - y_f) + y1 * y_f
    }
}
