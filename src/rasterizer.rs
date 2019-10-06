use crate::graphics_primitives::*;
use crate::math::*;

use core::ops::{Add, Mul};

use std::f32;

struct PixelBoundingBox {
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
}

impl From<&[Point2D; 3]> for PixelBoundingBox {
    fn from(vertices: &[Point2D; 3]) -> Self {
        let vals = vertices
            .iter()
            .fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |a, p| {
                (
                    a.0.min(p.x()),
                    a.1.max(p.x()),
                    a.2.min(p.y()),
                    a.3.max(p.y()),
                )
            });
        // Convert the min/max bounds into pixel coordinates. Always round
        // away from the center of the box.
        let (min_x, max_x, min_y, max_y) = (
            vals.0.floor() as usize,
            vals.1.ceil() as usize,
            vals.2.floor() as usize,
            vals.3.ceil() as usize,
        );
        assert!(min_x < max_x && min_y < max_y);
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ColorBufferFormat {
    RGBA,
    BGRA,
}

#[derive(Debug)]
pub struct ColorBuffer {
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    format: ColorBufferFormat,
}

impl ColorBuffer {
    pub fn new(width: usize, height: usize, format: ColorBufferFormat) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        // Initialize to black
        for _i in 0..width * height {
            buffer.push(0);
        }
        Self {
            buffer,
            width,
            height,
            format,
        }
    }

    // Clear to black
    fn clear(&mut self) {
        assert_eq!(self.buffer.len(), self.height * self.width);
        for i in 0..self.width * self.height {
            self.buffer[i] = 0;
        }
    }

    fn set_pixel(&mut self, row: usize, col: usize, color: Color) {
        match self.format {
            ColorBufferFormat::BGRA => self.buffer[row * self.width + col] = color.to_bgra(),
            ColorBufferFormat::RGBA => self.buffer[row * self.width + col] = color.to_rgba(),
        }
    }

    pub fn get_raw(&self) -> &Vec<u32> {
        &self.buffer
    }
}

#[derive(Debug)]
pub struct DepthBuffer {
    buffer: Vec<f32>,
    width: usize,
    height: usize,
}

impl DepthBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(width * height);
        // Initialize to max depth => everything will be in front
        for _i in 0..width * height {
            buffer.push(f32::MAX);
        }
        Self {
            buffer,
            width,
            height,
        }
    }

    // Clear to black
    fn clear(&mut self) {
        assert_eq!(self.buffer.len(), self.height * self.width);
        for i in 0..self.width * self.height {
            self.buffer[i] = f32::MAX;
        }
    }

    fn get_depth(&self, row: usize, col: usize) -> f32 {
        self.buffer[row * self.width + col]
    }

    fn set_depth(&mut self, row: usize, col: usize, depth: f32) {
        self.buffer[row * self.width + col] = depth;
    }
}

struct RasterizerTriangle {
    vertices: [Point2D; 3],
    depth: [f32; 3],
    normalized_depth: [f32; 3],
    attributes: [VertexAttribute; 3],
    line_normals: [Vec2; 3],
    area: f32,
}

impl RasterizerTriangle {
    fn area(vertices: &[Point2D; 3]) -> f32 {
        (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]) * 0.5
    }

    pub fn new(vertices: [Point2D; 3], attributes: [VertexAttribute; 3]) -> Self {
        unimplemented!();
        // Clockwise edge equations
        // To have the normals all pointing towards the inner part of the triangle,
        // they all need to have their positive halfspace to the right of the triangle.
        // If we wanted counter-clockwise, then we switch signs on both x and y of normals
        // (and also switch order for v computations above. Note that coordinate system
        // starts in upper left corner.

        /*
        let v0 = vertices[1] - vertices[0];
        let v1 = vertices[2] - vertices[1];
        let v2 = vertices[0] - vertices[2];
        let n0 = vec2(-v0.y(), v0.x());
        let n1 = vec2(-v1.y(), v1.x());
        let n2 = vec2(-v2.y(), v2.x());

        let line_normals = [n0, n1, n2];
        let area = Self::area(&vertices);

        Self {
            vertices,
            line_normals,
            attributes,
            area,
        }
        */
    }

    pub fn is_point_inside(&self, point: Point2D) -> bool {
        // Based on edge equations
        self.line_normals
            .iter()
            .zip(self.vertices.iter())
            .fold(true, |acc, (&n, &p)| (n.dot(point - p) >= 0.0) && acc)
    }

    fn barycentrics_at(&self, point: Point2D) -> Barycentrics {
        assert!(self.is_point_inside(point));
        let barycentric0 = Self::area(&[self.vertices[1], self.vertices[2], point]) / self.area;
        let barycentric1 = Self::area(&[self.vertices[2], self.vertices[0], point]) / self.area;
        let barycentric2 = 1.0 - barycentric0 - barycentric1;

        Barycentrics([barycentric0, barycentric1, barycentric2])
    }

    fn depth_at(&self, barys: &Barycentrics) -> f32 {
        barys.interpolate(self.normalized_depth)
    }

    fn color_at(&self, barys: &Barycentrics) -> Color {
        barys.interpolate(self.attributes).color
    }
}

// TODO: Perspective correct
// Rename to fragment?
struct Barycentrics([f32; 3]);

impl Barycentrics {
    fn interpolate<T>(&self, vals: [T; 3]) -> T
    where
        T: Default + Copy + Mul<f32, Output = T> + Add<Output = T>,
    {
        self.0
            .iter()
            .zip(vals.iter())
            .fold(T::default(), |acc, (&b, &v)| acc + v * b)
    }
}

pub struct Rasterizer {
    color_buffer: ColorBuffer,
    depth_buffer: DepthBuffer,
}

impl Rasterizer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            color_buffer: ColorBuffer::new(width, height, ColorBufferFormat::BGRA),
            depth_buffer: DepthBuffer::new(width, height),
        }
    }

    fn perspective_divide(triangle: &Triangle<ClipSpace>) -> Triangle<NDC> {
        let old_verts = triangle.vertices;

        let v0 = Vertex::<NDC>::new(
            old_verts[0].x() / old_verts[0].w(),
            old_verts[0].y() / old_verts[0].w(),
            old_verts[0].z() / old_verts[0].w(),
            old_verts[0].w(),
        );

        let v1 = Vertex::<NDC>::new(
            old_verts[1].x() / old_verts[1].w(),
            old_verts[1].y() / old_verts[1].w(),
            old_verts[1].z() / old_verts[1].w(),
            old_verts[1].w(),
        );

        let v2 = Vertex::<NDC>::new(
            old_verts[2].x() / old_verts[2].w(),
            old_verts[2].y() / old_verts[2].w(),
            old_verts[2].z() / old_verts[2].w(),
            old_verts[2].w(),
        );

        let vertices = [v0, v1, v2];
        Triangle::<NDC> {
            vertices,
            vertex_attributes: triangle.vertex_attributes,
        }
    }

    fn to_screen_space(&self, tri: &Triangle<NDC>) -> RasterizerTriangle {
        unimplemented!();
    }

    fn query_depth(&self, row: usize, col: usize) -> f32 {
        self.depth_buffer.get_depth(row, col)
    }

    fn write_pixel(&mut self, row: usize, col: usize, color: Color, depth: f32) {
        self.color_buffer.set_pixel(row, col, color);
        self.depth_buffer.set_depth(row, col, depth);
    }

    pub fn rasterize(&mut self, triangles: &[Triangle<ClipSpace>]) -> &ColorBuffer {
        self.color_buffer.clear();
        for triangle in triangles {
            let triangle = Rasterizer::perspective_divide(triangle);
            let triangle = self.to_screen_space(&triangle);
            let bounding_box = PixelBoundingBox::from(&triangle.vertices);
            for i in bounding_box.min_y..bounding_box.max_y {
                for j in bounding_box.min_x..bounding_box.max_x {
                    // Sample middle of pixel
                    let x = j as f32 + 0.5;
                    let y = i as f32 + 0.5;
                    let pos = Point2D::new(x, y);
                    if triangle.is_point_inside(pos) {
                        let barys = triangle.barycentrics_at(pos);
                        let tri_depth = triangle.depth_at(&barys);
                        if self.query_depth(i, j) < tri_depth {
                            continue;
                        }

                        let col = triangle.color_at(&barys);
                        self.write_pixel(i, j, col, tri_depth);
                    }
                }
            }
        }

        &self.color_buffer
    }
}
