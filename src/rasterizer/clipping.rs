use crate::graphics_primitives::{Triangle, VertexAttribute};
use crate::math::point::*;
use crate::math::vector::*;
use crate::math::ClipSpace;

#[derive(Debug, Clone)]
pub enum ClipResult {
    Outside,
    Inside,
    ClippedToSingle(Triangle<ClipSpace>),
    ClippedToDouble(Triangle<ClipSpace>, Triangle<ClipSpace>),
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum Intersection {
    BothOutside,
    BothInside,
    FirstInside {
        point: Point4D<ClipSpace>,
        line_param: f32,
    },
    SecondInside {
        point: Point4D<ClipSpace>,
        line_param: f32,
    },
}

const CULL_DEGENERATE_TRIANGLE_AREA_EPS: f32 = 0.000001;

fn intersect(
    plane_normal: &Vec4<ClipSpace>,
    p0: &Point4D<ClipSpace>,
    p1: &Point4D<ClipSpace>,
) -> Intersection {
    let p0_signed_dist = plane_normal.dot(p0.to_vec());
    let p1_signed_dist = plane_normal.dot(p1.to_vec());
    if p0_signed_dist > 0.0 && p1_signed_dist > 0.0 {
        return Intersection::BothInside;
    }

    if p0_signed_dist < 0.0 && p1_signed_dist < 0.0 {
        return Intersection::BothOutside;
    }

    // (1) Line: L(t) = p0 + (p1 - p0) * t
    // (2) Plane: n * (p - p_a) == 0. For all our planes, p_a == 0 => n * p == 0
    // Insert L(t) as p into (2) and solve for t (== line_param)
    // t = -p0 * n / (p1 - p0) * n
    let line_param_top = -plane_normal.dot(p0.to_vec());
    let line_param_bottom = plane_normal.dot(*p1 - *p0);

    // This means the line and the plane is parallell
    if line_param_bottom == 0.0 {
        // TODO: When is inside?
        return Intersection::BothOutside;
    }

    let line_param = line_param_top / line_param_bottom;
    let intersection = *p0 + (*p1 - *p0) * line_param;

    if p0_signed_dist > 0.0 {
        return Intersection::FirstInside {
            point: intersection,
            line_param,
        };
    }

    debug_assert!(p1_signed_dist > 0.0);
    Intersection::SecondInside {
        point: intersection,
        line_param,
    }
}

pub fn try_clip(triangle: &Triangle<ClipSpace>) -> ClipResult {
    let outside = |v: &Point4D<ClipSpace>| {
        (v.x().abs() > v.w() && v.y().abs() > v.w() && v.z().abs() > v.w()) || (v.w() < 0.0)
    };
    if triangle.vertices.iter().all(outside) {
        return ClipResult::Outside;
    }

    if super::triangle_2x_area(&triangle.vertices).abs() < CULL_DEGENERATE_TRIANGLE_AREA_EPS {
        return ClipResult::Outside;
    }

    let inside = |v: &Point4D<ClipSpace>| {
        v.x().abs() <= v.w() && v.y().abs() <= v.w() && v.z().abs() <= v.w()
    };
    if triangle.vertices.iter().all(inside) {
        return ClipResult::Inside;
    }

    // With these definitions, the positive half space points to inside the bounding box.
    // => A point is inside for dot() > 0.0
    let clip_planes: [Vec4<ClipSpace>; 6] = [
        vec4(1.0, 0.0, 0.0, 1.0),
        vec4(-1.0, 0.0, 0.0, 1.0),
        vec4(0.0, 1.0, 0.0, 1.0),
        vec4(0.0, -1.0, 0.0, 1.0),
        vec4(0.0, 0.0, 1.0, 1.0),
        vec4(0.0, 0.0, -1.0, 1.0),
    ];

    let mut out_vertices: Vec<Point4D<ClipSpace>> = triangle.vertices.to_vec();
    let mut out_attrs: Vec<VertexAttribute> = triangle.vertex_attributes.to_vec();

    // Sotherland-Hodgeman
    for clip_plane in clip_planes.iter() {
        let in_vertices = out_vertices.clone();
        let in_attrs = out_attrs.clone();
        out_attrs.clear();
        out_vertices.clear();
        for (i, (vert, attr)) in in_vertices.iter().zip(in_attrs.iter()).enumerate() {
            let next_vert = in_vertices[(i + 1) % in_vertices.len()];
            let next_attr = in_attrs[(i + 1) % in_attrs.len()];
            match intersect(clip_plane, &vert, &next_vert) {
                Intersection::BothOutside => continue,
                Intersection::BothInside => {
                    out_vertices.push(next_vert);
                    out_attrs.push(next_attr);
                }
                Intersection::FirstInside { point, line_param } => {
                    out_vertices.push(point);
                    // Interpolate
                    out_attrs.push((next_attr - *attr) * line_param + *attr);
                }
                Intersection::SecondInside { point, line_param } => {
                    out_vertices.push(point);
                    // Interpolate
                    out_attrs.push((next_attr - *attr) * line_param + *attr);
                    out_vertices.push(next_vert);
                    out_attrs.push(next_attr);
                }
            };
        }
    }

    debug_assert_eq!(out_attrs.len(), out_vertices.len());

    if out_vertices.len() == 3 {
        return ClipResult::ClippedToSingle(Triangle::<ClipSpace> {
            vertices: [out_vertices[0], out_vertices[1], out_vertices[2]],
            vertex_attributes: [out_attrs[0], out_attrs[1], out_attrs[2]],
        });
    }

    debug_assert_eq!(out_vertices.len(), 4);

    ClipResult::ClippedToDouble(
        Triangle::<ClipSpace> {
            vertices: [out_vertices[0], out_vertices[1], out_vertices[2]],
            vertex_attributes: [out_attrs[0], out_attrs[1], out_attrs[2]],
        },
        Triangle::<ClipSpace> {
            vertices: [out_vertices[2], out_vertices[3], out_vertices[0]],
            vertex_attributes: [out_attrs[2], out_attrs[3], out_attrs[0]],
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::color::Color;

    static vertex_attributes: [VertexAttribute; 3] = [
        VertexAttribute {
            color: Color::red(),
            uvs: [0.0, 0.0],
        },
        VertexAttribute {
            color: Color::red(),
            uvs: [0.0, 0.0],
        },
        VertexAttribute {
            color: Color::red(),
            uvs: [0.0, 0.0],
        },
    ];

    #[test]
    fn fully_inside() {
        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 0.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.0, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.5, 0.0, 0.0, 1.0),
        ];

        let tri = Triangle::<ClipSpace> {
            vertices,
            vertex_attributes,
        };

        assert!(std::matches!(try_clip(&tri), ClipResult::Inside));
    }

    fn fully_inside_2() {
        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 1.0, 0.0, -1.0),
            Point4D::<ClipSpace>::new(0.0, 1.5, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.5, 1.0, 0.0, 0.0),
        ];

        let tri = Triangle::<ClipSpace> {
            vertices,
            vertex_attributes,
        };

        assert!(std::matches!(try_clip(&tri), ClipResult::Inside));
    }

    #[test]
    fn cull_degenerate() {
        let vertices = [
            Point4D::<ClipSpace>::new(0.0, 0.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.0, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.0, 0.0, 0.0, 1.0),
        ];

        let tri = Triangle::<ClipSpace> {
            vertices,
            vertex_attributes,
        };

        assert!(std::matches!(try_clip(&tri), ClipResult::Outside));
    }

    fn cull_degenerate_2() {
        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.0, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.5, 1.0, 0.0, 1.0),
        ];

        let tri = Triangle::<ClipSpace> {
            vertices,
            vertex_attributes,
        };
        assert!(std::matches!(try_clip(&tri), ClipResult::Outside));
    }

    #[test]
    fn outside() {
        let vertices = [
            Point4D::<ClipSpace>::new(-0.6, 1.0, -1.0, 0.5),
            Point4D::<ClipSpace>::new(0.6, 1.2, -2.0, 0.5),
            Point4D::<ClipSpace>::new(0.6, 1.0, -1.5, 0.5),
        ];

        let tri = Triangle::<ClipSpace> {
            vertices,
            vertex_attributes,
        };
        assert!(std::matches!(try_clip(&tri), ClipResult::Outside));
    }

    #[test]
    fn partial_right_side_overlap() {
        let vertices = [
            Point4D::<ClipSpace>::new(1.5, 0.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(2.5, 1.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.6, 1.0, 0.0, 2.0),
        ];

        // Note that the algorithm reorders
        let expected0 = [
            Point4D::<ClipSpace>::new(2.0, 1.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.6, 1.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(1.5, 0.0, 0.0, 2.0),
        ];

        let expected1 = [
            Point4D::<ClipSpace>::new(1.5, 0.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(2.0, 0.5, 0.0, 2.0),
            Point4D::<ClipSpace>::new(2.0, 1.0, 0.0, 2.0),
        ];

        let tri = Triangle::<ClipSpace> {
            vertices,
            vertex_attributes,
        };

        match try_clip(&tri) {
            ClipResult::ClippedToDouble(tri0, tri1) => {
                assert_eq!(tri0.vertices, expected0);
                assert_eq!(tri1.vertices, expected1);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn partial_right_side_overlap_single() {
        let vertices = [
            Point4D::<ClipSpace>::new(2.4, 0.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(2.5, 1.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.6, 1.0, 0.0, 2.0),
        ];

        // Note that the algorithm reorders
        let expected = [
            Point4D::<ClipSpace>::new(1.5, 1.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.5, 2.1, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.6, 2.2, 0.0, 2.0),
        ];

        let tri = Triangle::<ClipSpace> {
            vertices,
            vertex_attributes,
        };

        match try_clip(&tri) {
            ClipResult::ClippedToSingle(tri) => {
                assert_eq!(tri.vertices, expected);
            }
            _ => unreachable!(),
        }
    }

    // Test TODO:
    // - parallel with clip volume
    // - All outside but covers the whole clipspace
}
