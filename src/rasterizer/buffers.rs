use super::bounding_box::PixelBoundingBox;
use super::N_MSAA_SAMPLES;
use crate::color::Color;

pub const CLEAR_COLOR: u32 = 0xFF191919;
pub const CLEAR_DEPTH: f32 = f32::MAX;

pub const TILE_SIZE: usize = 64;

// Keeps two masks to allow clearing prev resolve buffer before writing
pub struct BufferTiles {
    tiles: Vec<PixelBoundingBox>,
    masks: [Vec<bool>; 2],
    mask_idx: usize,
    n_horizontal: usize,
}

impl BufferTiles {
    pub fn new(width: usize, height: usize) -> Self {
        let is_width_exact = width % TILE_SIZE == 0;
        let is_height_exact = height % TILE_SIZE == 0;
        let n_horizontal = width / TILE_SIZE + if is_width_exact { 0 } else { 1 };
        let n_vertical = height / TILE_SIZE + if is_height_exact { 0 } else { 1 };

        let n_tiles = n_horizontal * n_vertical;

        let mut tiles = Vec::with_capacity(n_tiles);

        for j in 0..n_vertical {
            for i in 0..n_horizontal {
                tiles.push(PixelBoundingBox {
                    min_x: i * TILE_SIZE,
                    max_x: ((i + 1) * TILE_SIZE).min(width),
                    min_y: j * TILE_SIZE,
                    max_y: ((j + 1) * TILE_SIZE).min(height),
                });
            }
        }
        let masks = [vec![false; n_tiles], vec![false; n_tiles]];

        Self {
            tiles,
            masks,
            n_horizontal,
            mask_idx: 0,
        }
    }

    pub fn tile_idx(&self, row: usize, col: usize) -> usize {
        (row / TILE_SIZE) * self.n_horizontal + (col / TILE_SIZE)
    }

    pub fn mark(&mut self, row: usize, col: usize) {
        let idx = self.tile_idx(row, col);
        self.masks[self.mask_idx][idx] = true;
    }

    pub fn next(&mut self) {
        self.mask_idx = (self.mask_idx + 1) % 2;
        for v in self.masks[self.mask_idx].iter_mut() {
            *v = false;
        }
    }

    pub fn prev_marked(&self) -> impl Iterator<Item = &PixelBoundingBox> {
        self.tiles
            .iter()
            .zip(self.masks[(self.mask_idx + 1) % 2].iter())
            .filter(|(_, &marked)| marked)
            .map(|(tile, _)| tile)
    }

    pub fn marked(&self) -> impl Iterator<Item = &PixelBoundingBox> {
        self.tiles
            .iter()
            .zip(self.masks[self.mask_idx].iter())
            .filter(|(_, &marked)| marked)
            .map(|(tile, _)| tile)
    }
}

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
        let resolve_buffer = vec![CLEAR_COLOR; width * height];
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
        debug_assert!((0.0..=1.0).contains(&depth), "Invalid depth: {}", depth);
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

    #[test]
    fn buffer_tiles_pow_2_square() {
        let mut tiles = BufferTiles::new(128, 128);

        assert_eq!(tiles.tiles.len(), 128 / TILE_SIZE * 128 / TILE_SIZE);
        assert!(tiles.masks[0].iter().all(|x| !x));
        assert_eq!(tiles.tiles[0].min_x, 0);
        assert_eq!(tiles.tiles[0].max_x, TILE_SIZE);
        assert_eq!(tiles.tiles[0].min_y, 0);
        assert_eq!(tiles.tiles[0].max_y, TILE_SIZE);

        dbg!(&tiles.tiles);
        assert_eq!(tiles.tiles[128 / TILE_SIZE - 1].min_x, 128 - TILE_SIZE);
        assert_eq!(tiles.tiles[128 / TILE_SIZE - 1].max_x, 128);
        assert_eq!(tiles.tiles[128 / TILE_SIZE - 1].min_y, 0);
        assert_eq!(tiles.tiles[128 / TILE_SIZE - 1].max_y, TILE_SIZE);

        assert_eq!(tiles.tiles.last().unwrap().min_x, 128 - TILE_SIZE);
        assert_eq!(tiles.tiles.last().unwrap().max_x, 128);
        assert_eq!(tiles.tiles.last().unwrap().min_y, 128 - TILE_SIZE);
        assert_eq!(tiles.tiles.last().unwrap().max_y, 128);

        tiles.mark(0, 0);
        assert!(!tiles.masks[0].iter().all(|x| !x));
        assert!(tiles.masks[0][0]);

        tiles.mark(TILE_SIZE - 1, TILE_SIZE - 1);
        assert_eq!(tiles.marked().count(), 1);
        assert!(tiles.masks[0][0]);

        tiles.mark(127, 127);
        assert_eq!(tiles.marked().count(), 2);
        assert!(tiles.masks[0].last().unwrap());

        tiles.mark(64, 64);
        let expected = if TILE_SIZE >= 64 { 2 } else { 3 };
        assert_eq!(tiles.marked().count(), expected);
        assert!(tiles.masks[0][64 / TILE_SIZE * (128 / TILE_SIZE) + 64 / TILE_SIZE]);
    }

    #[test]
    fn buffer_tiles_uneven_rect() {
        let mut tiles = BufferTiles::new(442, 711);
        let n_horizontal = 7;
        let n_vertical = 12;

        assert_eq!(tiles.tiles.len(), n_horizontal * n_vertical);

        assert_eq!(tiles.tiles[0].min_x, 0);
        assert_eq!(tiles.tiles[0].max_x, TILE_SIZE);
        assert_eq!(tiles.tiles[0].min_y, 0);
        assert_eq!(tiles.tiles[0].max_y, TILE_SIZE);

        assert_eq!(tiles.tiles.last().unwrap().min_x, 442 - (442 % TILE_SIZE));
        assert_eq!(tiles.tiles.last().unwrap().max_x, 442);
        assert_eq!(tiles.tiles.last().unwrap().min_y, 711 - (711 % TILE_SIZE));
        assert_eq!(tiles.tiles.last().unwrap().max_y, 711);

        // End of first row
        assert_eq!(tiles.tiles[442 / TILE_SIZE].min_x, 442 - (442 % TILE_SIZE));
        assert_eq!(tiles.tiles[442 / TILE_SIZE].max_x, 442);
        assert_eq!(tiles.tiles[442 / TILE_SIZE].min_y, 0);
        assert_eq!(tiles.tiles[442 / TILE_SIZE].max_y, TILE_SIZE);

        // First column
        assert_eq!(tiles.tiles[n_horizontal * (n_vertical - 1)].min_x, 0);
        assert_eq!(
            tiles.tiles[n_horizontal * (n_vertical - 1)].max_x,
            TILE_SIZE
        );
        assert_eq!(
            tiles.tiles[n_horizontal * (n_vertical - 1)].min_y,
            711 - (711 % TILE_SIZE)
        );
        assert_eq!(tiles.tiles[n_horizontal * (n_vertical - 1)].max_y, 711);

        tiles.mark(0, 0);
        assert!(!tiles.masks[0].iter().all(|x| !x));
        assert!(tiles.masks[0][0]);

        tiles.mark(TILE_SIZE - 1, TILE_SIZE - 1);
        assert_eq!(tiles.marked().count(), 1);
        assert!(tiles.masks[0][0]);

        tiles.mark(711, 442);
        assert_eq!(tiles.marked().count(), 2);
        assert!(tiles.masks[0].last().unwrap());

        tiles.mark(64, 64);
        assert_eq!(tiles.marked().count(), 3);
        assert!(tiles.masks[0][64 / TILE_SIZE * (442 / TILE_SIZE + 1) + 64 / TILE_SIZE]);

        tiles.next();

        assert_eq!(tiles.marked().count(), 0);
        assert_eq!(tiles.prev_marked().count(), 3);
        assert!(tiles.masks[0][64 / TILE_SIZE * (442 / TILE_SIZE + 1) + 64 / TILE_SIZE]);
        assert!(tiles.masks[0].last().unwrap());

        tiles.next();

        assert_eq!(tiles.marked().count(), 0);
        assert_eq!(tiles.prev_marked().count(), 0);
    }
}
