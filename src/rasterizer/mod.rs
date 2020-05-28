use crate::color::Color;
use crate::graphics_primitives::*;
use crate::math::*;
use crate::uniform::*;

mod bounding_box;
mod buffers;

use crate::rasterizer::bounding_box::*;
use crate::rasterizer::buffers::*;

use std::f32;

fn triangle_2x_area<CS: CoordinateSystem, const N: usize>(vertices: &[Point<CS, { N }>]) -> f32 {
    let v10 = vertices[1].xy() - vertices[0].xy();
    let v20 = vertices[2].xy() - vertices[0].xy();
    v10.cross(v20)
}

const N_MSAA_SAMPLES: u8 = 4;

#[derive(Copy, Clone, Debug)]
pub struct CoverageMask {
    mask: u8,
}

impl CoverageMask {
    const fn len() -> u8 {
        N_MSAA_SAMPLES
    }
    fn new() -> Self {
        CoverageMask { mask: 0u8 }
    }

    fn any(&self) -> bool {
        self.mask != 0
    }

    fn all(&self) -> bool {
        self.mask == 0b1111
    }

    fn empty(&self) -> bool {
        self.mask == 0
    }

    fn get(&self, i: u8) -> bool {
        debug_assert!(i < N_MSAA_SAMPLES);
        ((1 << i) & self.mask) != 0
    }

    fn set(&mut self, i: u8, v: bool) {
        debug_assert!(i < N_MSAA_SAMPLES);
        let v = if v { 1 } else { 0 };
        self.mask = (self.mask & (!(1 << i))) | (v << i);
    }
}

struct Fragment<'a> {
    sampled_depths: [f32; N_MSAA_SAMPLES as usize],
    edge_functions: &'a EdgeFunctions,
    depths_camera_space: &'a [f32; 3],
    triangle_attributes: &'a [VertexAttribute; 3],
}

impl<'a> Fragment<'a> {
    fn interpolate(&self, x: usize, y: usize, cov: CoverageMask) -> VertexAttribute {
        let mut x_sample = x as f32 + 0.5;
        let mut y_sample = y as f32 + 0.5;

        // We have to sample inside the triangle
        if !cov.all() {
            for i in 0..N_MSAA_SAMPLES {
                if cov.get(i) {
                    x_sample = x as f32 + RGSS_SAMPLE_PATTERN[i as usize][0];
                    y_sample = y as f32 + RGSS_SAMPLE_PATTERN[i as usize][1];
                    break;
                }
            }
        }

        let efs = self.edge_functions.eval_single(x_sample, y_sample);

        // Perspective correct barycentrics.
        let f_u = efs[1] / self.depths_camera_space[0];
        let f_v = efs[2] / self.depths_camera_space[1];
        let f_w = efs[0] / self.depths_camera_space[2];
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
    const EPS: f32 = 0.0001;
    debug_assert!(x >= 0.0 - EPS && x <= 1.0 + EPS, "{}", x);
    x.clamp(0.0, 1.0)
}

// Rotated grid super sampling
const RGSS_SAMPLE_PATTERN: [[f32; 2]; N_MSAA_SAMPLES as usize] = [
    [5.0 / 8.0, 1.0 / 8.0],
    [7.0 / 8.0, 5.0 / 8.0],
    [3.0 / 8.0, 7.0 / 8.0],
    [1.0 / 8.0, 3.0 / 8.0],
];

#[derive(Debug, Clone)]
struct EdgeFunctions {
    points: [Point2D; 3],
    normals: [Vec2; 3],
    coverage_evaluated: [[f32; 3]; N_MSAA_SAMPLES as usize],
    coverage_mask: CoverageMask,
}

impl EdgeFunctions {
    fn eval_single(&self, x: f32, y: f32) -> [f32; 3] {
        let p = Point2D::new(x, y);
        [
            self.normals[0].dot(p - self.points[0]),
            self.normals[1].dot(p - self.points[1]),
            self.normals[2].dot(p - self.points[2]),
        ]
    }

    fn eval(&mut self, x: usize, y: usize) {
        for i in 0..N_MSAA_SAMPLES {
            let x_sample = x as f32 + RGSS_SAMPLE_PATTERN[i as usize][0];
            let y_sample = y as f32 + RGSS_SAMPLE_PATTERN[i as usize][1];

            self.coverage_evaluated[i as usize] = self.eval_single(x_sample, y_sample);

            self.coverage_mask.set(
                i,
                EdgeFunctions::inside(&self.normals, &self.coverage_evaluated[i as usize]),
            );
        }
    }

    fn inside(normals: &[Vec2; 3], eval_edge_funcs: &[f32; 3]) -> bool {
        eval_edge_funcs
            .iter()
            .zip(normals.iter())
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

    fn any_coverage(&self) -> bool {
        self.coverage_mask.any()
    }
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
            coverage_evaluated: [[0.0; 3]; N_MSAA_SAMPLES as usize],
            coverage_mask: CoverageMask::new(),
        };

        Self {
            edge_functions,
            depths_camera_space,
            depths: [vertices[0].z(), vertices[1].z(), vertices[2].z()],
            attributes,
            inv_2x_area,
        }
    }

    // See realtime rendering on details
    fn fragment(&self) -> Fragment<'_> {
        let interpolate_depth = |edge_functions: &[f32; 3]| -> f32 {
            // Linear barycentrics, used only for interpolating z
            let bary0 = clamp_bary(edge_functions[1] * self.inv_2x_area);
            let bary1 = clamp_bary(edge_functions[2] * self.inv_2x_area);
            let bary2 = clamp_bary(1.0 - bary0 - bary1);

            // z here is in NDC and in that transform it was divided by w (camera space depth) which
            // means we can interpolate it with the linear barycentrics. For attributes, we need
            // perspective correct barycentrics
            bary0 * self.depths[0] + bary1 * self.depths[1] + bary2 * self.depths[2]
        };

        let mut sampled_depths = [0.0; N_MSAA_SAMPLES as usize];

        for i in 0..N_MSAA_SAMPLES {
            if self.edge_functions.coverage_mask.get(i) {
                sampled_depths[i as usize] =
                    interpolate_depth(&self.edge_functions.coverage_evaluated[i as usize]);
            }
        }

        Fragment {
            sampled_depths,
            edge_functions: &self.edge_functions,
            depths_camera_space: &self.depths_camera_space,
            triangle_attributes: &self.attributes,
        }
    }
}

pub struct FragCoords {
    // x,y are screen space
    pub x: f32,
    pub y: f32,
    pub depths: [f32; 4],
    pub mask: CoverageMask,
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
            ColorBuffer::new(width, height),
            ColorBuffer::new(width, height),
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

    fn depth_coverage(
        &self,
        row: usize,
        col: usize,
        cov: CoverageMask,
        sampled_depths: &[f32; N_MSAA_SAMPLES as usize],
    ) -> CoverageMask {
        let cur_depths = self.depth_buffers[self.buf_idx].get_depth(row * self.width + col);
        let mut depth_cov = CoverageMask::new();
        for i in 0..N_MSAA_SAMPLES {
            if cov.get(i) {
                depth_cov.set(i, sampled_depths[i as usize] < cur_depths[i as usize]);
            }
        }
        depth_cov
    }

    fn write_pixel(
        &mut self,
        row: usize,
        col: usize,
        color: Color,
        depths: &[f32; N_MSAA_SAMPLES as usize],
        cov_mask: CoverageMask,
    ) {
        for i in 0..N_MSAA_SAMPLES {
            if cov_mask.get(i) {
                let idx = row * self.width + col;
                self.color_buffers[self.buf_idx].set_pixel(idx, i, color);
                self.depth_buffers[self.buf_idx].set_depth(idx, i, depths[i as usize]);
            }
        }
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
                for j in b_box.min_x..b_box.max_x {
                    triangle.edge_functions.eval(j, i);
                    if triangle.edge_functions.any_coverage() {
                        let fragment = triangle.fragment();
                        let cov_mask = self.depth_coverage(
                            i,
                            j,
                            triangle.edge_functions.coverage_mask,
                            &fragment.sampled_depths,
                        );
                        if cov_mask.empty() {
                            continue;
                        }

                        let fc = FragCoords {
                            x: j as f32 + 0.5,
                            y: i as f32 + 0.5,
                            depths: fragment.sampled_depths,
                            mask: fragment.edge_functions.coverage_mask,
                        };

                        let col =
                            fragment_shader(uniforms, &fc, &fragment.interpolate(j, i, cov_mask));
                        self.write_pixel(i, j, col, &fragment.sampled_depths, cov_mask);
                    }
                }
            }
        }
    }

    fn resolve_and_clear(&mut self, buf_idx: usize) -> &[u32] {
        debug_assert_eq!(
            self.width * self.height,
            self.color_buffers[self.buf_idx].buffer.len()
        );
        debug_assert_eq!(
            self.width * self.height,
            self.depth_buffers[self.buf_idx].buffer.len()
        );

        let resolve = &mut self.color_buffers[self.buf_idx].resolve_buffer;
        let cbuf = &mut self.color_buffers[self.buf_idx].buffer;
        let dbuf = &mut self.depth_buffers[self.buf_idx].buffer;
        for (r, (c, d)) in resolve.iter_mut().zip(cbuf.iter_mut().zip(dbuf.iter_mut())) {
            *r = ColorBuffer::box_filter_color(c);
            *c = [buffers::CLEAR_COLOR; N_MSAA_SAMPLES as usize];
            *d = [buffers::CLEAR_DEPTH; N_MSAA_SAMPLES as usize];
        }

        resolve
    }

    pub fn swap_buffers(&mut self) -> &[u32] {
        let prev = self.buf_idx;
        self.buf_idx = (self.buf_idx + 1) % 2;
        self.resolve_and_clear(prev)
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
    fn coverage_mask() {
        let mut m = CoverageMask::new();
        assert!(m.empty());
        assert!(!m.any());

        m.set(0, true);
        assert!(!m.empty());
        assert!(m.any());
        assert!(m.get(0));
        assert!(!m.get(1));
        assert!(!m.get(2));
        assert!(!m.get(3));

        m.set(2, true);
        assert!(m.get(0));
        assert!(!m.get(1));
        assert!(m.get(2));
        assert!(!m.get(3));
        assert!(m.any());
        assert!(!m.empty());

        m.set(3, false);
        assert!(m.get(0));
        assert!(!m.get(1));
        assert!(m.get(2));
        assert!(!m.get(3));
        assert!(m.any());
        assert!(!m.empty());

        m.set(0, false);
        assert!(!m.get(0));
        assert!(!m.get(1));
        assert!(m.get(2));
        assert!(!m.get(3));
        assert!(m.any());
        assert!(!m.empty());

        m.set(2, false);
        assert!(!m.get(0));
        assert!(!m.get(1));
        assert!(!m.get(2));
        assert!(!m.get(3));
        assert!(!m.any());
        assert!(m.empty());
    }

    fn setup_rasterizer_triangle() -> RasterizerTriangle {
        let vertices = [
            Point3D::<ScreenSpace>::new(100.0, 300.0, 0.5),
            Point3D::<ScreenSpace>::new(200.0, 150.0, 0.5),
            Point3D::<ScreenSpace>::new(300.0, 300.0, 0.5),
        ];

        let depths = [5.0, 6.0, 7.0];

        let vertex_attributes = [
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
        ];

        RasterizerTriangle::new(vertices, depths, vertex_attributes)
    }

    #[test]
    fn edge_functions_basic() {
        let mut rast_tri = setup_rasterizer_triangle();

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

        rast_tri.edge_functions.eval(200, 200);
        assert!(rast_tri.edge_functions.any_coverage());
        assert_eq!(rast_tri.edge_functions.coverage_mask.mask, 0b1111);

        rast_tri.edge_functions.eval(99, 299);
        assert!(!rast_tri.edge_functions.any_coverage());
        assert_eq!(rast_tri.edge_functions.coverage_mask.mask, 0);

        rast_tri.edge_functions.eval(101, 299);
        assert!(rast_tri.edge_functions.any_coverage());
        assert_eq!(rast_tri.edge_functions.coverage_mask.mask, 0b1111);

        rast_tri.edge_functions.eval(200, 149);
        assert!(!rast_tri.edge_functions.any_coverage());
        assert_eq!(rast_tri.edge_functions.coverage_mask.mask, 0);
        rast_tri.edge_functions.eval(200, 151);
        assert!(rast_tri.edge_functions.any_coverage());
        assert_eq!(rast_tri.edge_functions.coverage_mask.mask, 0b1111);

        rast_tri.edge_functions.eval(301, 300);
        assert!(!rast_tri.edge_functions.any_coverage());
        assert_eq!(rast_tri.edge_functions.coverage_mask.mask, 0);
    }

    #[test]
    fn edge_functions_partial() {
        let mut rast_tri = setup_rasterizer_triangle();

        rast_tri.edge_functions.eval(299, 299);
        assert!(rast_tri.edge_functions.any_coverage());
        assert_eq!(rast_tri.edge_functions.coverage_mask.mask, 0b1100);

        rast_tri.edge_functions.eval(150, 224);
        assert!(rast_tri.edge_functions.any_coverage());
        assert_eq!(rast_tri.edge_functions.coverage_mask.mask, 0b0111);

        rast_tri.edge_functions.eval(250, 225);
        assert!(rast_tri.edge_functions.any_coverage());
        assert_eq!(rast_tri.edge_functions.coverage_mask.mask, 0b1100);
    }

    #[test]
    fn edge_functions_tie_breaker() {
        let rast_tri = setup_rasterizer_triangle();

        // Testing the tie-breaker rules.
        let e = rast_tri.edge_functions.eval_single(150.0, 225.0);
        assert!(EdgeFunctions::inside(&rast_tri.edge_functions.normals, &e));
        assert_eq!(e[0], 0.0);
        assert_eq!(rast_tri.edge_functions.normals[0].x() > 0.0, true);

        let e = rast_tri.edge_functions.eval_single(250.0, 225.0);
        assert!(!EdgeFunctions::inside(&rast_tri.edge_functions.normals, &e));
        assert_eq!(e[1], 0.0);
        assert_eq!(rast_tri.edge_functions.normals[1].x() < 0.0, true);

        let e = rast_tri.edge_functions.eval_single(250.0, 300.0);
        assert!(EdgeFunctions::inside(&rast_tri.edge_functions.normals, &e));
        assert_eq!(e[2], 0.0);
        assert_eq!(rast_tri.edge_functions.normals[2].x() == 0.0, true);
        assert_eq!(rast_tri.edge_functions.normals[2].y() < 0.0, true);
    }

    #[test]
    fn fragment_creation_same_depth() {
        let mut rast_tri = setup_rasterizer_triangle();

        rast_tri.edge_functions.eval(200, 200);

        let fragment = rast_tri.fragment();
        assert_eq!(fragment.sampled_depths, [0.5; 4]);
    }

    #[test]
    fn fragment_creation_same_depth_partial_coverage() {
        let mut rast_tri = setup_rasterizer_triangle();
        rast_tri.edge_functions.eval(299, 299);

        let fragment = rast_tri.fragment();
        assert_eq!(fragment.sampled_depths, [0.0, 0.0, 0.5, 0.5]);
    }

    #[test]
    fn fragment_creation_interp_depth() {
        let vertices = [
            Point3D::<ScreenSpace>::new(100.0, 300.0, 0.5),
            Point3D::<ScreenSpace>::new(200.0, 150.0, 0.3),
            Point3D::<ScreenSpace>::new(300.0, 300.0, 0.8),
        ];

        // Not important atm, so can be anything
        let depths = [5.0, 6.0, 7.0];

        let vertex_attributes = [
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 0.0]).into(),
        ];

        let mut rast_tri = RasterizerTriangle::new(vertices, depths, vertex_attributes);

        rast_tri.edge_functions.eval(101, 299);
        let fragment = rast_tri.fragment();
        // This is expected to be very close to the attribute
        assert_eq!(
            fragment.sampled_depths,
            [0.50039583, 0.5019375, 0.50177085, 0.5002292]
        );

        rast_tri.edge_functions.eval(200, 151);
        let fragment = rast_tri.fragment();
        // This is expected to be very close to the attribute
        assert_eq!(
            fragment.sampled_depths,
            [0.30356252, 0.30510417, 0.3049375, 0.30339584]
        );

        rast_tri.edge_functions.eval(298, 299);
        let fragment = rast_tri.fragment();
        // This is expected to be very close to the attribute
        assert_eq!(
            fragment.sampled_depths,
            [0.7958958, 0.79743755, 0.79727083, 0.79572916]
        );

        // Sample in the middle
        rast_tri.edge_functions.eval(200, 258);
        let fragment = rast_tri.fragment();
        assert_eq!(
            fragment.sampled_depths,
            [0.55322915, 0.5547708, 0.5546042, 0.55306244]
        );
    }

    fn verify_uvs_at(rast_tri: &mut RasterizerTriangle, x: usize, y: usize, expected: &[f32; 2]) {
        rast_tri.edge_functions.eval(x, y);
        let fragment = rast_tri.fragment();
        let attrs = fragment.interpolate(x, y, rast_tri.edge_functions.coverage_mask);
        assert_eq!(&attrs.uvs, expected);
    }

    #[test]
    fn fragment_creation_interp_attr_same_depth() {
        // These depths are not used for attribute interpolation
        let vertices = [
            Point3D::<ScreenSpace>::new(100.0, 300.0, 0.5),
            Point3D::<ScreenSpace>::new(200.0, 150.0, 0.5),
            Point3D::<ScreenSpace>::new(300.0, 300.0, 0.5),
        ];

        // These are used for interpolation
        let depths = [5.0, 5.0, 5.0];

        let vertex_attributes = [
            (Color::red(), [0.0, 0.0]).into(),
            (Color::red(), [0.0, 1.0]).into(),
            (Color::red(), [1.0, 1.0]).into(),
        ];

        let mut rast_tri = RasterizerTriangle::new(vertices, depths, vertex_attributes);

        // This is expected to be very close to the attribute
        verify_uvs_at(&mut rast_tri, 100, 299, &[0.00020831265, 0.006041646]);
        verify_uvs_at(&mut rast_tri, 200, 150, &[0.004791677, 0.99895835]);
        verify_uvs_at(&mut rast_tri, 299, 299, &[0.99645835, 0.9972917]);

        // Sample in the middle
        verify_uvs_at(&mut rast_tri, 200, 258, &[0.3641667, 0.6408334]);
    }
}
