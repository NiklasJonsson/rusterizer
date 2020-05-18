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
        debug_assert!(min_x < max_x, "{} < {}", min_x, max_x);
        debug_assert!(min_y < max_y, "{} < {}", min_y, max_y);
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
        debug_assert_eq!(self.buffer.len(), self.height * self.width);
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
        debug_assert_eq!(self.buffer.len(), self.height * self.width);
        for i in 0..self.width * self.height {
            self.buffer[i] = f32::MAX;
        }
    }

    fn get_depth(&self, row: usize, col: usize) -> f32 {
        self.buffer[row * self.width + col]
    }

    fn set_depth(&mut self, row: usize, col: usize, depth: f32) {
        debug_assert!(depth >= 0.0 && depth <= 1.0, "Invalid depth: {}", depth);
        self.buffer[row * self.width + col] = depth;
    }
}

struct Fragment<'a> {
    pub depth: f32,
    edge_functions: &'a [f32; 3],
    depths_camera_space: &'a [f32; 3],
    triangle_attributes: &'a [VertexAttribute; 3],
}

impl<'a> Fragment<'a> {
    fn interpolate(&self) -> VertexAttribute {
        // Perspective correct barycentrics.
        let f_u = self.edge_functions[1] / self.depths_camera_space[0];
        let f_v = self.edge_functions[2] / self.depths_camera_space[1];
        let f_w = self.edge_functions[0] / self.depths_camera_space[2];
        let sum = f_u + f_v + f_w;
        let u = clamp_bary(f_u / sum);
        let v = clamp_bary(f_v / sum);
        let w = clamp_bary(1.0 - u - v);

        self.triangle_attributes[0] * u
            + self.triangle_attributes[1] * v
            + self.triangle_attributes[2] * w
    }
}

fn clamp_bary(x: f32) -> f32 {
    const EPS: f32 = 0.0005;
    debug_assert!(x >= 0.0 - EPS && x <= 1.0 + EPS, "{}", x);
    x.clamp(0.0, 1.0)
}

#[derive(Debug, Clone)]
struct EdgeFunctions {
    points: [Point2D; 3],
    normals: [Vec2; 3],
    evaluated: [f32; 3],
}

impl EdgeFunctions {
    fn eval(&mut self, x: f32, y: f32) {
        let p = Point2D::new(x, y);
        self.evaluated = [
            self.normals[0].dot(p - self.points[0]),
            self.normals[1].dot(p - self.points[1]),
            self.normals[2].dot(p - self.points[2]),
        ];
    }

    fn step_x(&mut self) {
        for i in 0..self.evaluated.len() {
            self.evaluated[i] += self.normals[i].x();
        }
    }

    fn inside(&self) -> bool {
        self.evaluated
            .iter()
            .zip(self.normals.iter())
            .all(|(val, normal)| {
                if *val > 0.0 {
                    return true;
                }
                if *val < 0.0 {
                    return false;
                }
                if normal.x() > 0.0 {
                    return true;
                }
                if normal.x() < 0.0 {
                    return false;
                }
                if normal.y() < 0.0 {
                    return true;
                }
                return false;
            })
    }
}

fn triangle_2x_area<CS: CoordinateSystem, const N: usize>(vertices: &[Point<CS, { N }>]) -> f32 {
    let v10 = vertices[1].xy() - vertices[0].xy();
    let v20 = vertices[2].xy() - vertices[0].xy();
    v10.cross(v20)
}

const CULL_DEGENERATE_TRIANGLE_AREA_EPS: f32 = 0.000001;

// Implicitly in 2D Screen space
#[derive(Debug, Clone)]
struct RasterizerTriangle {
    edge_functions: EdgeFunctions,
    depths_camera_space: [f32; 3],
    depths: [f32; 3],
    attributes: [VertexAttribute; 3],
    inv_2x_area: f32,
}

impl RasterizerTriangle {
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

        let inv_2x_area = 1.0 / triangle_2x_area(&vertices);

        let edge_functions = EdgeFunctions {
            points: [vertices[0].xy(), vertices[1].xy(), vertices[2].xy()],
            normals: [n0, n1, n2],
            evaluated: [0.0, 0.0, 0.0],
        };

        Self {
            edge_functions,
            depths_camera_space,
            depths: [vertices[0].z(), vertices[1].z(), vertices[2].z()],
            attributes,
            inv_2x_area,
        }
    }

    fn eval_edge_functions(&mut self, x: f32, y: f32) {
        self.edge_functions.eval(x, y);
    }

    fn step_edge_func_x(&mut self) {
        self.edge_functions.step_x();
    }

    fn inside(&self) -> bool {
        self.edge_functions.inside()
    }

    // See realtime rendering on details
    fn fragment<'a>(&'a self) -> Fragment<'a> {
        let edge_functions = &self.edge_functions.evaluated;
        // Linear barycentrics, used only for interpolating z
        let bary0 = clamp_bary(edge_functions[1] * self.inv_2x_area);
        let bary1 = clamp_bary(edge_functions[2] * self.inv_2x_area);
        let bary2 = clamp_bary(1.0 - bary0 - bary1);

        // z here is in NDC and in that transform it was divided by w (camera space depth) which
        // means we can interpolate it with the linear barycentrics. For attributes, we need
        // perspective correct barycentrics
        let depth = bary0 * self.depths[0] + bary1 * self.depths[1] + bary2 * self.depths[2];

        Fragment {
            depth,
            edge_functions,
            depths_camera_space: &self.depths_camera_space,
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

    fn viewport_transform(&self, tri: Triangle<NDC>) -> RasterizerTriangle {
        let zmin = 0.0;
        let zmax = 1.0;
        let new_vert = |vert: Point4D<NDC>| {
            debug_assert!(vert.x() <= 1.0 && vert.x() >= -1.0);
            debug_assert!(vert.y() <= 1.0 && vert.y() >= -1.0);
            debug_assert!(vert.z() <= 1.0 && vert.z() >= -1.0);

            let x = self.width as f32 * (vert.x() + 1.0) / 2.0;
            // Flip y as color buffer start upper left
            let y = self.height as f32 * (1.0 - (vert.y() + 1.0) / 2.0);

            // Remap to z range
            let z = (vert.z() + 1.0) * 0.5 * (zmax - zmin) + zmin;
            debug_assert!(z >= zmin && z <= zmax);
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

    fn can_cull(vertices: &[Point4D<ClipSpace>]) -> bool {
        vertices.iter().all(|x| x.w() <= 0.0)
            || triangle_2x_area(vertices).abs() < CULL_DEGENERATE_TRIANGLE_AREA_EPS
    }

    pub fn rasterize(
        &mut self,
        triangles: &[Triangle<ClipSpace>],
        uniforms: &Uniforms,
        fragment_shader: crate::render::FragmentShader,
    ) {
        for triangle in triangles {
            if Rasterizer::can_cull(&triangle.vertices) {
                continue;
            }

            let triangle = Rasterizer::perspective_divide(triangle);

            let mut triangle = self.viewport_transform(triangle);
            let b_box = PixelBoundingBox::from(&triangle.edge_functions.points);
            for i in b_box.min_y..b_box.max_y {
                triangle.eval_edge_functions(b_box.min_x as f32 + 0.5, i as f32 + 0.5);
                for j in b_box.min_x..b_box.max_x {
                    // Sample middle of pixel
                    if triangle.inside() {
                        let fragment = triangle.fragment();
                        if self.query_depth(i, j) < fragment.depth {
                            continue;
                        }

                        let fc = FragCoords {
                            x: j as f32 + 0.5,
                            y: i as f32 + 0.5,
                            depth: fragment.depth,
                        };

                        let col = fragment_shader(uniforms, &fc, &fragment.interpolate());
                        self.write_pixel(i, j, col, fragment.depth);
                    }
                    triangle.step_edge_func_x();
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_culling() {
        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 0.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.0, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.5, 0.0, 0.0, 1.0),
        ];

        assert_eq!(Rasterizer::can_cull(&vertices), false);

        // Note that this should probably be partially culled
        // and reconstructed
        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 1.0, 0.0, -1.0),
            Point4D::<ClipSpace>::new(0.0, 1.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.5, 1.0, 0.0, 0.0),
        ];

        assert_eq!(Rasterizer::can_cull(&vertices), true);
    }

    #[test]
    fn cull_degenerate() {
        let vertices = [
            Point4D::<ClipSpace>::new(0.0, 0.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.0, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.0, 0.0, 0.0, 1.0),
        ];

        assert_eq!(Rasterizer::can_cull(&vertices), true);

        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.0, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.5, 1.0, 0.0, 1.0),
        ];

        assert_eq!(Rasterizer::can_cull(&vertices), true);
    }

    #[test]
    fn cull_near() {
        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 1.0, 0.0, -1.0),
            Point4D::<ClipSpace>::new(0.0, 1.0, 0.0, -2.0),
            Point4D::<ClipSpace>::new(0.5, 1.0, 0.0, 0.0),
        ];

        assert_eq!(Rasterizer::can_cull(&vertices), true);
    }

    #[test]
    fn perspective_divide() {
        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 0.9, 0.0, 10.0),
            Point4D::<ClipSpace>::new(0.08, 0.3, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.5, -0.3, 0.0, 1.0),
        ];

        let vertex_attributes = [
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
        ];

        let tri = Triangle::<ClipSpace> {
            vertices,
            vertex_attributes,
        };

        let tri_ndc = Rasterizer::perspective_divide(&tri);

        let expected = [
            Point4D::<NDC>::new(-0.05, 0.089999996, 0.0, 10.0),
            Point4D::<NDC>::new(0.04, 0.15, 0.0, 2.0),
            Point4D::<NDC>::new(0.5, -0.3, 0.0, 1.0),
        ];

        assert_eq!(expected.len(), tri_ndc.vertices.len());
        for i in 0..expected.len() {
            assert_eq!(tri_ndc.vertices[i], expected[i]);
        }
    }

    #[test]
    fn bounding_box() {
        let points = [
            Point2D::new(100.0, 200.0),
            Point2D::new(230.0, 200.0),
            Point2D::new(230.0, 300.0),
        ];

        let bb = PixelBoundingBox::from(&points);

        assert_eq!(bb.min_x, 100);
        assert_eq!(bb.max_x, 230);
        assert_eq!(bb.min_y, 200);
        assert_eq!(bb.max_y, 300);

        let points = [
            Point2D::new(50.9, 200.0),
            Point2D::new(230.0, 100.0),
            Point2D::new(500.0, 200.9),
        ];

        let bb = PixelBoundingBox::from(&points);

        assert_eq!(bb.min_x, 50);
        assert_eq!(bb.max_x, 500);
        assert_eq!(bb.min_y, 100);
        assert_eq!(bb.max_y, 201);
    }

    #[test]
    fn viewport_transform_1() {
        const WIDTH: usize = 400;
        const HEIGHT: usize = 500;

        let rasterizer = Rasterizer::new(WIDTH, HEIGHT);

        let vertices = [
            Point4D::<NDC>::new(-1.0, 0.5, -0.5, 5.0),
            Point4D::<NDC>::new(1.0, 0.5, 0.0, 6.0),
            Point4D::<NDC>::new(0.0, -0.5, 0.5, 7.0),
        ];

        let vertex_attributes = [
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
        ];

        let tri = Triangle::<NDC> {
            vertices,
            vertex_attributes,
        };

        let rast_tri = rasterizer.viewport_transform(tri);

        for i in 0..3 {
            assert_eq!(rast_tri.depths_camera_space[i], vertices[i].w());
        }

        assert_eq!(rast_tri.depths[0], 0.25);
        assert_eq!(rast_tri.depths[1], 0.5);
        assert_eq!(rast_tri.depths[2], 0.75);

        assert_eq!(rast_tri.inv_2x_area, 0.00001);

        // Y is flipped in screen space
        assert_eq!(rast_tri.edge_functions.points[0], Point2D::new(0.0, 125.0));
        assert_eq!(
            rast_tri.edge_functions.points[1],
            Point2D::new(400.0, 125.0)
        );
        assert_eq!(
            rast_tri.edge_functions.points[2],
            Point2D::new(200.0, 375.0)
        );
    }

    #[test]
    fn viewport_transform_2() {
        const WIDTH: usize = 400;
        const HEIGHT: usize = 500;

        let rasterizer = Rasterizer::new(WIDTH, HEIGHT);

        let vertices = [
            Point4D::<NDC>::new(-0.25, 1.0, -1.0, 5.0),
            Point4D::<NDC>::new(0.5, 0.0, 0.0, 6.0),
            Point4D::<NDC>::new(0.25, -1.0, 1.0, 7.0),
        ];

        let vertex_attributes = [
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
        ];

        let tri = Triangle::<NDC> {
            vertices,
            vertex_attributes,
        };

        let rast_tri = rasterizer.viewport_transform(tri);

        for i in 0..3 {
            assert_eq!(rast_tri.depths_camera_space[i], vertices[i].w());
        }

        assert_eq!(rast_tri.depths[0], 0.0);
        assert_eq!(rast_tri.depths[1], 0.5);
        assert_eq!(rast_tri.depths[2], 1.0);

        assert_eq!(rast_tri.inv_2x_area, 0.00002);

        assert_eq!(rast_tri.edge_functions.points[0], Point2D::new(150.0, 0.0));
        assert_eq!(
            rast_tri.edge_functions.points[1],
            Point2D::new(300.0, 250.0)
        );
        assert_eq!(
            rast_tri.edge_functions.points[2],
            Point2D::new(250.0, 500.0)
        );
    }

    #[test]
    fn edge_functions() {
        const WIDTH: usize = 400;
        const HEIGHT: usize = 600;

        let rasterizer = Rasterizer::new(WIDTH, HEIGHT);

        let vertices = [
            Point4D::<NDC>::new(-0.5, 0.0, 0.0, 5.0),
            Point4D::<NDC>::new(0.0, 0.5, 0.0, 6.0),
            Point4D::<NDC>::new(0.5, 0.0, 0.0, 7.0),
        ];

        let vertex_attributes = [
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
        ];

        let tri = Triangle::<NDC> {
            vertices,
            vertex_attributes,
        };

        let mut rast_tri = rasterizer.viewport_transform(tri);

        assert_eq!(
            rast_tri.edge_functions.points[0],
            Point2D::new(100.0, 300.0)
        );
        assert_eq!(
            rast_tri.edge_functions.points[1],
            Point2D::new(200.0, 150.0)
        );

        assert_eq!(
            rast_tri.edge_functions.points[2],
            Point2D::new(300.0, 300.0)
        );

        rast_tri.eval_edge_functions(200.0, 200.0);
        assert!(rast_tri.inside());

        rast_tri.eval_edge_functions(99.0, 299.0);
        assert!(!rast_tri.inside());
        rast_tri.eval_edge_functions(101.0, 299.0);
        assert!(rast_tri.inside());

        rast_tri.eval_edge_functions(200.0, 149.0);
        assert!(!rast_tri.inside());
        rast_tri.eval_edge_functions(200.0, 151.0);
        assert!(rast_tri.inside());

        rast_tri.eval_edge_functions(301.0, 300.0);
        assert!(!rast_tri.inside());
        rast_tri.eval_edge_functions(299.0, 299.0);
        assert!(rast_tri.inside());

        // Testing the tie-breaker rules.
        rast_tri.eval_edge_functions(150.0, 225.0);
        assert!(rast_tri.inside());
        assert_eq!(rast_tri.edge_functions.evaluated[0], 0.0);
        assert_eq!(rast_tri.edge_functions.normals[0].x() > 0.0, true);

        rast_tri.eval_edge_functions(250.0, 225.0);
        assert!(!rast_tri.inside());
        assert_eq!(rast_tri.edge_functions.evaluated[1], 0.0);
        assert_eq!(rast_tri.edge_functions.normals[1].x() < 0.0, true);

        rast_tri.eval_edge_functions(250.0, 300.0);
        assert!(rast_tri.inside());
        assert_eq!(rast_tri.edge_functions.evaluated[2], 0.0);
        assert_eq!(rast_tri.edge_functions.normals[2].x() == 0.0, true);
        assert_eq!(rast_tri.edge_functions.normals[2].y() < 0.0, true);
    }
}
