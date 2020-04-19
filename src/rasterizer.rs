use crate::color::Color;
use crate::graphics_primitives::*;
use crate::math::*;
use crate::uniform::*;

use std::f32;

#[derive(Debug)]
struct PixelBoundingBox {
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
}

impl From<&[Point3D<ScreenSpace>; 3]> for PixelBoundingBox {
    fn from(vertices: &[Point3D<ScreenSpace>; 3]) -> Self {
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
        assert!(min_x < max_x, "{} < {}", min_x, max_x);
        assert!(min_y < max_y, "{} < {}", min_y, max_y);
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
        let mut ret = Self {
            buffer,
            width,
            height,
            format,
        };

        ret.clear();
        ret
    }

    // Clear to dark grey
    fn clear(&mut self) {
        assert_eq!(self.buffer.len(), self.height * self.width);
        for i in 0..self.width {
            for j in 0..self.height {
                self.set_pixel(
                    i,
                    j,
                    Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.1,
                        a: 1.0,
                    },
                );
            }
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

struct Fragment<'a> {
    pub depth: f32,
    barycentrics: [f32; 3],
    triangle_depths: &'a [f32; 3],
    triangle_attributes: &'a [VertexAttribute; 3],
}

impl<'a> Fragment<'a> {
    fn perspective_interpolation(&self) -> VertexAttribute {
        ((self.triangle_attributes[0] / self.triangle_depths[0]) * self.barycentrics[0]
            + (self.triangle_attributes[1] / self.triangle_depths[1]) * self.barycentrics[1]
            + (self.triangle_attributes[2] / self.triangle_depths[2]) * self.barycentrics[2])
            * self.depth
    }
    fn linear_interpolation(&self) -> VertexAttribute {
        self.triangle_attributes[0] * self.barycentrics[0]
            + self.triangle_attributes[1] * self.barycentrics[1]
            + self.triangle_attributes[2] * self.barycentrics[2]
    }
}

// Implicitly in 2D Screen space
#[derive(Debug, Clone)]
struct RasterizerTriangle {
    vertices: [Point3D<ScreenSpace>; 3],
    depths_camera_space: [f32; 3],
    attributes: [VertexAttribute; 3],
    line_normals: [Vec2; 3],
    area: f32,
}

impl RasterizerTriangle {
    fn area(vertices: &[Point2D; 3]) -> f32 {
        (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]) * 0.5
    }

    pub fn new(
        vertices: [Point3D<ScreenSpace>; 3],
        depths_camera_space: [f32; 3],
        attributes: [VertexAttribute; 3],
    ) -> Self {
        // Clockwise edge equations
        // To have the normals all pointing towards the inner part of the triangle,
        // they all need to have their positive halfspace to the right of the triangle.
        // If we wanted counter-clockwise, then we switch signs on both x and y of normals
        // (and also switch order for v computations above. Note that coordinate system
        // starts in upper left corner.

        let v0 = vertices[1] - vertices[0];
        let v1 = vertices[2] - vertices[1];
        let v2 = vertices[0] - vertices[2];
        let n0 = vec2(-v0.y(), v0.x());
        let n1 = vec2(-v1.y(), v1.x());
        let n2 = vec2(-v2.y(), v2.x());

        let line_normals = [n0, n1, n2];

        // Cross product is the area of the parallelogram
        let v10 = vertices[1].xy() - vertices[0].xy();
        let v20 = vertices[2].xy() - vertices[0].xy();
        let area = v10.cross(v20) * 0.5;

        Self {
            vertices,
            depths_camera_space,
            attributes,
            line_normals,
            area,
        }
    }

    pub fn edge_functions(&self, point: Point2D) -> [f32; 3] {
        [
            self.line_normals[0].dot(point - self.vertices[0].xy()),
            self.line_normals[1].dot(point - self.vertices[1].xy()),
            self.line_normals[2].dot(point - self.vertices[2].xy()),
        ]
    }

    fn fragment_with<'a>(&'a self, edge_functions: &[f32; 3]) -> Fragment<'a> {
        let bary0 = 0.5 * edge_functions[1] / self.area;
        let bary1 = 0.5 * edge_functions[2] / self.area;
        let bary2 = 1.0 - bary0 - bary1;

        let barycentrics = [bary0, bary1, bary2];

        // Perspective correct depth entails interpolating the inverse of the depth with
        // barycentrics
        // See https://www.comp.nus.edu.sg/~lowkl/publications/lowk_persp_interp_techrep.pdf for
        // derivation
        let depth_inv = bary0 * (1.0 / self.depths_camera_space[0])
            + bary1 * (1.0 / self.depths_camera_space[1])
            + bary2 * (1.0 / self.depths_camera_space[2]);
        let depth = 1.0 / depth_inv;

        let _linear_depth = bary0 * self.depths_camera_space[0]
            + bary1 * self.depths_camera_space[1]
            + bary2 * self.depths_camera_space[2];

        //let depth = _linear_depth;

        Fragment {
            depth,
            barycentrics,
            triangle_depths: &self.depths_camera_space,
            triangle_attributes: &self.attributes,
        }
    }
}

pub struct FragCoords {
    // x,y are screen space
    pub x: f32,
    pub y: f32,
    pub depth: f32,
}

pub struct Rasterizer {
    color_buffers: [ColorBuffer; 2],
    depth_buffers: [DepthBuffer; 2],
    buf_idx: usize,
    width: usize,
    height: usize,
}

impl Rasterizer {
    pub fn new(width: usize, height: usize) -> Self {
        let color_buffers = [
            ColorBuffer::new(width, height, ColorBufferFormat::BGRA),
            ColorBuffer::new(width, height, ColorBufferFormat::BGRA),
        ];

        let depth_buffers = [
            DepthBuffer::new(width, height),
            DepthBuffer::new(width, height),
        ];

        Self {
            width,
            height,
            buf_idx: 0,
            color_buffers,
            depth_buffers,
        }
    }

    // Divide x, y and z by w
    fn perspective_divide(triangle: &Triangle<ClipSpace>) -> Triangle<NDC> {
        let old_verts = triangle.vertices;

        let v0 = Point4D::<NDC>::new(
            old_verts[0].x() / old_verts[0].w(),
            old_verts[0].y() / old_verts[0].w(),
            old_verts[0].z() / old_verts[0].w(),
            old_verts[0].w(),
        );

        let v1 = Point4D::<NDC>::new(
            old_verts[1].x() / old_verts[1].w(),
            old_verts[1].y() / old_verts[1].w(),
            old_verts[1].z() / old_verts[1].w(),
            old_verts[1].w(),
        );

        let v2 = Point4D::<NDC>::new(
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

    fn viewport_transform(&self, tri: &Triangle<NDC>) -> RasterizerTriangle {
        let zmin = 0.0;
        let zmax = 1.0;
        let new_vert = |vert: Point4D<NDC>| {
            assert!(vert.x() <= 1.0 && vert.x() >= -1.0);
            assert!(vert.y() <= 1.0 && vert.y() >= -1.0);
            assert!(vert.z() <= 1.0 && vert.z() >= -1.0);

            let x = self.width as f32 * (vert.x() + 1.0) / 2.0;
            // Flip y as color buffer start upper left
            let y = self.height as f32 * (1.0 - (vert.y() + 1.0) / 2.0);

            // Remap to z range
            let z = (vert.z() + 1.0) * 0.5 * (zmax - zmin) + zmin;
            Point3D::new(x, y, z)
        };
        let vertices = [
            new_vert(tri.vertices[0]),
            new_vert(tri.vertices[1]),
            new_vert(tri.vertices[2]),
        ];

        let depths = [
            tri.vertices[0].w(),
            tri.vertices[1].w(),
            tri.vertices[2].w(),
        ];

        RasterizerTriangle::new(vertices, depths, tri.vertex_attributes)
    }

    fn query_depth(&self, row: usize, col: usize) -> f32 {
        self.depth_buffers[self.buf_idx].get_depth(row, col)
    }

    fn write_pixel(&mut self, row: usize, col: usize, color: Color, depth: f32) {
        self.color_buffers[self.buf_idx].set_pixel(row, col, color);
        self.depth_buffers[self.buf_idx].set_depth(row, col, depth);
    }

    fn can_cull(triangle: &Triangle<ClipSpace>) -> bool {
        return triangle.vertices.iter().all(|x| x.w() <= 0.0);
    }

    fn rasterize_triangle<FS>(
        &mut self,
        triangle: &RasterizerTriangle,
        uniforms: &Uniforms,
        fragment_shader: FS,
    ) where
        FS: Fn(&Uniforms, &FragCoords, &VertexAttribute) -> Color + Copy,
    {
        let bounding_box = PixelBoundingBox::from(&triangle.vertices);
        for i in bounding_box.min_y..bounding_box.max_y {
            for j in bounding_box.min_x..bounding_box.max_x {
                // Sample middle of pixel
                let x = j as f32 + 0.5;
                let y = i as f32 + 0.5;
                let edge_functions = triangle.edge_functions(Point2D::new(x, y));
                if edge_functions.iter().all(|&x| x > 0.0) {
                    let fragment = triangle.fragment_with(&edge_functions);
                    if self.query_depth(i, j) < fragment.depth {
                        continue;
                    }

                    let fc = FragCoords {
                        x,
                        y,
                        depth: fragment.depth,
                    };

                    let col = fragment_shader(uniforms, &fc, &fragment.perspective_interpolation());
                    self.write_pixel(i, j, col, fragment.depth);
                }
            }
        }
    }

    pub fn rasterize<FS>(
        &mut self,
        triangles: &[Triangle<ClipSpace>],
        uniforms: &Uniforms,
        fragment_shader: FS,
    ) where
        FS: Fn(&Uniforms, &FragCoords, &VertexAttribute) -> Color + Copy,
    {
        for triangle in triangles {
            if Rasterizer::can_cull(triangle) {
                continue;
            }

            let triangle = Rasterizer::perspective_divide(triangle);
            let triangle = self.viewport_transform(&triangle);
            self.rasterize_triangle(&triangle, uniforms, fragment_shader);
        }
    }

    pub fn swap_buffers(&mut self) -> &ColorBuffer {
        let prev = self.buf_idx;
        self.buf_idx = (self.buf_idx + 1) % 2;
        self.depth_buffers[self.buf_idx].clear();
        self.color_buffers[self.buf_idx].clear();
        &self.color_buffers[prev]
    }
}
