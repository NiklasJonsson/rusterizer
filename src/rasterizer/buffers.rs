use super::N_MSAA_SAMPLES;
use crate::color::Color;

use std::f32;

#[derive(Debug, Copy, Clone)]
pub enum ColorBufferFormat {
    RGBA,
    BGRA,
    ARGB,
}

#[derive(Debug)]
pub struct ColorBuffer {
    buffer: Vec<[u32; N_MSAA_SAMPLES as usize]>,
    resolve_buffer: Vec<u32>,
    width: usize,
    height: usize,
}

impl ColorBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        let mut resolve_buffer = Vec::with_capacity(width * height);
        // Initialize to black
        for _i in 0..width * height {
            buffer.push([0; N_MSAA_SAMPLES as usize]);
            resolve_buffer.push(0);
        }
        let mut ret = Self {
            buffer,
            resolve_buffer,
            width,
            height,
        };

        ret.clear();
        ret
    }

    // Clear to dark grey
    pub fn clear(&mut self) {
        debug_assert_eq!(self.buffer.len(), self.height * self.width);
        for i in 0..self.height {
            for j in 0..self.width {
                for mask_idx in 0..N_MSAA_SAMPLES {
                    self.set_pixel(
                        i,
                        j,
                        Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        },
                        mask_idx,
                    );
                }
            }
        }
    }

    pub fn set_pixel(&mut self, row: usize, col: usize, color: Color, mask_idx: u8) {
        self.buffer[row * self.width + col][mask_idx as usize] = color.to_argb();
    }

    fn box_filter_color(colors: &[u32; N_MSAA_SAMPLES as usize]) -> u32 {
        let mut red_sum = 0;
        let mut blue_sum = 0;
        let mut green_sum = 0;
        for i in 0..N_MSAA_SAMPLES {
            red_sum += (colors[i as usize] & 0x00FF0000) >> 16;
            green_sum += (colors[i as usize] & 0x0000FF00) >> 8;
            blue_sum += colors[i as usize] & 0x000000FF;
        }

        (0xFF << 24)
            | (red_sum / N_MSAA_SAMPLES as u32) << 16
            | (green_sum / N_MSAA_SAMPLES as u32) << 8
            | (blue_sum / N_MSAA_SAMPLES as u32)
    }

    pub fn resolve(&mut self) -> &[u32] {
        for i in 0..self.height {
            for j in 0..self.width {
                let idx = i * self.width + j;
                self.resolve_buffer[idx] = Self::box_filter_color(&self.buffer[idx]);
            }
        }

        &self.resolve_buffer
    }
}

#[derive(Debug)]
pub struct DepthBuffer {
    buffer: Vec<[f32; N_MSAA_SAMPLES as usize]>,
    width: usize,
    height: usize,
}

impl DepthBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        // Initialize to max depth => everything will be in front
        for _i in 0..width * height {
            buffer.push([f32::MAX; N_MSAA_SAMPLES as usize]);
        }
        Self {
            buffer,
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        debug_assert_eq!(self.buffer.len(), self.height * self.width);
        for i in 0..self.width * self.height {
            self.buffer[i] = [f32::MAX; N_MSAA_SAMPLES as usize];
        }
    }

    pub fn get_depth(&self, row: usize, col: usize) -> [f32; N_MSAA_SAMPLES as usize] {
        self.buffer[row * self.width + col]
    }

    pub fn set_depth(&mut self, row: usize, col: usize, depth: f32, mask_idx: u8) {
        debug_assert!(depth >= 0.0 && depth <= 1.0, "Invalid depth: {}", depth);
        self.buffer[row * self.width + col][mask_idx as usize] = depth;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: u32 = 0xFFFF0000u32;
    const BLUE: u32 = 0xFF0000FFu32;
    const GREEN: u32 = 0xFF00FF00u32;

    fn verify_avg_same(c: u32) {
        let colors = [c; 4];
        let avg = ColorBuffer::box_filter_color(&colors);
        assert_eq!(c, avg, "{:x}, {:x}", c, avg);
    }

    #[test]
    fn average_same_color() {
        verify_avg_same(BLUE);
        verify_avg_same(GREEN);
        verify_avg_same(RED);
    }

    #[test]
    fn average_two_colors() {
        let colors = [RED, BLUE, RED, BLUE];
        let avg = ColorBuffer::box_filter_color(&colors);
        let expected = 0xFF7F007Fu32;
        assert_eq!(expected, avg, "{:x}, {:x}", expected, avg);

        let colors = [RED, RED, BLUE, BLUE];
        let expected = 0xFF7F007Fu32;
        let avg = ColorBuffer::box_filter_color(&colors);

        let colors = [RED, GREEN, RED, GREEN];
        let expected = 0xFF7F7F00u32;
        let avg = ColorBuffer::box_filter_color(&colors);
    }

    #[test]
    fn average_three_colors() {
        let colors = [RED, GREEN, RED, BLUE];
        let avg = ColorBuffer::box_filter_color(&colors);
        let expected = 0xFF7F3F3Fu32;
        assert_eq!(expected, avg, "{:x}, {:x}", expected, avg);
    }

    #[test]
    fn average_colors() {
        let colors = [0xFF35B565, 0xFFF3FA12, 0xFF3E5469, 0xFF435623];

        let avg = ColorBuffer::box_filter_color(&colors);
        let expected = 0xFF6A9640u32;
        assert_eq!(expected, avg, "{:x}, {:x}", expected, avg);
    }
}
