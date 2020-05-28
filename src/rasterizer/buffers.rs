use super::N_MSAA_SAMPLES;
use crate::color::Color;

use std::f32;

pub const CLEAR_COLOR: u32 = 0xFF191919;
pub const CLEAR_DEPTH: f32 = f32::MAX;

#[derive(Debug)]
pub struct ColorBuffer {
    pub clear_buffer: Vec<[u32; N_MSAA_SAMPLES as usize]>,
    pub buffer: Vec<[u32; N_MSAA_SAMPLES as usize]>,
    pub resolve_buffer: Vec<u32>,
}

impl ColorBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        let mut clear_buffer = Vec::with_capacity(width * height);
        let resolve_buffer = vec![0u32; width * height];
        // Initialize to black
        for _i in 0..width * height {
            buffer.push([CLEAR_COLOR; N_MSAA_SAMPLES as usize]);
            clear_buffer.push([CLEAR_COLOR; N_MSAA_SAMPLES as usize]);
        }

        Self {
            clear_buffer,
            buffer,
            resolve_buffer,
        }
    }

    pub fn set_pixel(&mut self, pixel_idx: usize, mask_idx: u8, color: Color) {
        self.buffer[pixel_idx][mask_idx as usize] = color.to_argb();
    }

    pub fn box_filter_color(colors: &[u32; N_MSAA_SAMPLES as usize]) -> u32 {
        let mut red_sum = 0;
        let mut blue_sum = 0;
        let mut green_sum = 0;
        for c in colors.iter() {
            red_sum += (c & 0x00FF0000) >> 16;
            green_sum += (c & 0x0000FF00) >> 8;
            blue_sum += c & 0x000000FF;
        }

        (0xFF << 24)
            | (red_sum / N_MSAA_SAMPLES as u32) << 16
            | (green_sum / N_MSAA_SAMPLES as u32) << 8
            | (blue_sum / N_MSAA_SAMPLES as u32)
    }
}

#[derive(Debug)]
pub struct DepthBuffer {
    pub buffer: Vec<[f32; N_MSAA_SAMPLES as usize]>,
    pub clear_buffer: Vec<[f32; N_MSAA_SAMPLES as usize]>,
}

impl DepthBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        let mut clear_buffer = Vec::with_capacity(width * height);
        // Initialize to max depth => everything will be in front
        for _i in 0..width * height {
            buffer.push([CLEAR_DEPTH; N_MSAA_SAMPLES as usize]);
            clear_buffer.push([CLEAR_DEPTH; N_MSAA_SAMPLES as usize]);
        }
        Self {
            buffer,
            clear_buffer,
        }
    }

    pub fn get_depth(&self, idx: usize) -> &[f32; N_MSAA_SAMPLES as usize] {
        &self.buffer[idx]
    }

    pub fn set_depth(&mut self, idx: usize, mask_idx: u8, depth: f32) {
        debug_assert!(depth >= 0.0 && depth <= 1.0, "Invalid depth: {}", depth);
        self.buffer[idx][mask_idx as usize] = depth;
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
        assert_eq!(expected, avg, "{:x}, {:x}", expected, avg);

        let colors = [RED, GREEN, RED, GREEN];
        let expected = 0xFF7F7F00u32;
        let avg = ColorBuffer::box_filter_color(&colors);
        assert_eq!(expected, avg, "{:x}, {:x}", expected, avg);
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
