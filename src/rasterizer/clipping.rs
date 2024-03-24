use crate::graphics_primitives::{Triangle, VertexAttribute};
use crate::math::point::*;
use crate::math::vector::*;
use crate::math::ClipSpace;

#[derive(Debug, Clone)]
pub enum ClipResult {
    Outside,
    Inside,
    Clipped(Vec<Triangle<ClipSpace>>),
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
    if p0_signed_dist >= 0.0 && p1_signed_dist >= 0.0 {
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

    // This means the line and the plane is parallel
    if line_param_bottom == 0.0 {
        // If this is true, the point fulfills the plane equation and is inside the plane
        if line_param_top == 0.0 {
            return Intersection::BothInside;
        }
        return Intersection::BothOutside;
    }

    let line_param = line_param_top / line_param_bottom;
    let intersection = *p0 + (*p1 - *p0) * line_param;

    if p0_signed_dist >= 0.0 {
        return Intersection::FirstInside {
            point: intersection,
            line_param,
        };
    }

    debug_assert!(p1_signed_dist >= 0.0);
    Intersection::SecondInside {
        point: intersection,
        line_param,
    }
}

pub fn try_clip(triangle: &Triangle<ClipSpace>) -> ClipResult {
    if super::triangle_2x_area(&triangle.vertices).abs() < CULL_DEGENERATE_TRIANGLE_AREA_EPS {
        return ClipResult::Outside;
    }

    let mut inside = [[true; 2]; 3];
    let mut outside = [[true; 2]; 3];
    for v in triangle.vertices.iter() {
        inside[0][0] &= v.x() >= -v.w();
        inside[1][0] &= v.y() >= -v.w();
        inside[2][0] &= v.z() >= -v.w();

        inside[0][1] &= v.x() <= v.w();
        inside[1][1] &= v.y() <= v.w();
        inside[2][1] &= v.z() <= v.w();

        outside[0][0] &= v.x() < -v.w();
        outside[1][0] &= v.y() < -v.w();
        outside[2][0] &= v.z() < -v.w();

        outside[0][1] &= v.x() > v.w();
        outside[1][1] &= v.y() > v.w();
        outside[2][1] &= v.z() > v.w();
    }

    if outside.iter().any(|&x| x.iter().any(|&x| x)) {
        return ClipResult::Outside;
    }

    if inside.iter().all(|&x| x.iter().all(|&x| x)) {
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
            let prev_vert = in_vertices[(i + in_vertices.len() - 1) % in_vertices.len()];
            let prev_attr = in_attrs[(i + in_vertices.len() - 1) % in_attrs.len()];
            match intersect(clip_plane, &prev_vert, vert) {
                Intersection::BothOutside => continue,
                Intersection::BothInside => {
                    out_vertices.push(*vert);
                    out_attrs.push(*attr);
                }
                Intersection::FirstInside { point, line_param } => {
                    out_vertices.push(point);
                    // Interpolate
                    out_attrs.push((*attr - prev_attr) * line_param + prev_attr);
                }
                Intersection::SecondInside { point, line_param } => {
                    out_vertices.push(point);
                    // Interpolate
                    out_attrs.push((*attr - prev_attr) * line_param + prev_attr);
                    out_vertices.push(*vert);
                    out_attrs.push(*attr);
                }
            };
        }
    }

    // This can happen if even though initially, one or more points are inside, through clipping,
    // they end up outside.
    if out_vertices.is_empty() {
        return ClipResult::Outside;
    }

    debug_assert_eq!(out_attrs.len(), out_vertices.len());
    debug_assert!(out_vertices.len() >= 3);

    let mut out = Vec::with_capacity(out_vertices.len() - 2);

    for i in 0..out_vertices.len() - 2 {
        out.push(Triangle {
            vertices: [out_vertices[0], out_vertices[i + 1], out_vertices[i + 2]],
            vertex_attributes: [out_attrs[0], out_attrs[i + 1], out_attrs[i + 2]],
        });
    }

    debug_assert_eq!(out_vertices.len() - 2, out.len());

    ClipResult::Clipped(out)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect_0() {
        let clip_plane = vec4::<ClipSpace>(0.0, 1.0, 0.0, 1.0);
        let p0 = Point4D::<ClipSpace>::new(3.93749976, -7.0, 5.06030178, 7.0);
        let p1 = Point4D::<ClipSpace>::new(6.0, 7.0, 5.06030178, 7.0);

        assert!(std::matches!(
            intersect(&clip_plane, &p0, &p1),
            Intersection::BothInside
        ));
    }

    use crate::color::Color;

    const VERTEX_ATTRIBUTES: [VertexAttribute; 3] = [
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

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };

        assert!(std::matches!(try_clip(&tri), ClipResult::Inside));
    }

    fn fully_inside_2() {
        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 1.0, 0.0, -1.0),
            Point4D::<ClipSpace>::new(0.0, 1.5, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.5, 1.0, 0.0, 0.0),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
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

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };

        assert!(std::matches!(try_clip(&tri), ClipResult::Outside));
    }

    fn cull_degenerate_2() {
        let vertices = [
            Point4D::<ClipSpace>::new(-0.5, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.0, 1.0, 0.0, 1.0),
            Point4D::<ClipSpace>::new(0.5, 1.0, 0.0, 1.0),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
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

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
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

        let expected0 = [
            Point4D::<ClipSpace>::new(1.5, 0.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(2.0, 0.5, 0.0, 2.0),
            Point4D::<ClipSpace>::new(2.0, 1.0, 0.0, 2.0),
        ];

        let expected1 = [
            Point4D::<ClipSpace>::new(1.5, 0.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(2.0, 1.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.6, 1.0, 0.0, 2.0),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };

        match try_clip(&tri) {
            ClipResult::Clipped(tris) => {
                assert_eq!(tris.len(), 2);
                assert_eq!(tris[0].vertices, expected0);
                assert_eq!(tris[1].vertices, expected1);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn two_side_overlap() {
        let vertices = [
            Point4D::<ClipSpace>::new(-7.06294441, 5.06294441, 5.06030178, 7.0),
            Point4D::<ClipSpace>::new(-6.06294441, 7.06294441, 5.06030178, 7.0),
            Point4D::<ClipSpace>::new(-5.06294441, 5.06294441, 5.06030178, 7.0),
        ];

        let expected0 = [
            Point4D::<ClipSpace>::new(-7.0, 5.0629444, 5.060302, 7.0),
            Point4D::<ClipSpace>::new(-7.0, 5.188833, 5.060302, 7.0),
            Point4D::<ClipSpace>::new(-6.0944166, 7.0, 5.060302, 7.0),
        ];

        let expected1 = [
            Point4D::<ClipSpace>::new(-7.0, 5.0629444, 5.060302, 7.0),
            Point4D::<ClipSpace>::new(-6.0944166, 7.0, 5.060302, 7.0),
            Point4D::<ClipSpace>::new(-6.031472, 7.0, 5.060302, 7.0),
        ];

        let expected2 = [
            Point4D::<ClipSpace>::new(-7.0, 5.0629444, 5.060302, 7.0),
            Point4D::<ClipSpace>::new(-6.031472, 7.0, 5.060302, 7.0),
            Point4D::<ClipSpace>::new(-5.0629444, 5.0629444, 5.060302, 7.0),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };

        match try_clip(&tri) {
            ClipResult::Clipped(tris) => {
                assert_eq!(tris.len(), 3);
                assert_eq!(tris[0].vertices, expected0);
                assert_eq!(tris[1].vertices, expected1);
                assert_eq!(tris[2].vertices, expected2);
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

        let expected = [
            Point4D::<ClipSpace>::new(2.0, 0.22222227, 0.0, 2.0),
            Point4D::<ClipSpace>::new(2.0, 1.0, 0.0, 2.0),
            Point4D::<ClipSpace>::new(0.6, 1.0, 0.0, 2.0),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };

        match try_clip(&tri) {
            ClipResult::Clipped(tris) => {
                assert_eq!(tris.len(), 1);
                assert_eq!(tris[0].vertices, expected);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn on_right_edge() {
        let vertices = [
            Point4D::<ClipSpace>::new(5.12279034, -1.0, 5.06030178, 7.0),
            Point4D::<ClipSpace>::new(7.0, -0.754419327, 5.06030178, 7.0),
            Point4D::<ClipSpace>::new(7.0, -1.0, 5.06030178, 7.0),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };
        assert!(std::matches!(try_clip(&tri), ClipResult::Inside));
    }

    #[test]
    fn late_outside() {
        // Initially some points are inside each axis so this triangle does not get caught
        // by the early checks. After a few rounds of clipping though, it ends up with 0 inside.
        let vertices = [
            Point4D::<ClipSpace>::new(-4.70005131, -4.70005131, 1.32306385, 3.29994869),
            Point4D::<ClipSpace>::new(-3.70005131, -2.70005131, 1.32306385, 3.29994869),
            Point4D::<ClipSpace>::new(-2.70005131, -4.70005131, 1.32306385, 3.29994869),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };
        assert!(std::matches!(try_clip(&tri), ClipResult::Outside));
    }

    #[test]
    fn complete_coverage() {
        // Initially some points are inside each axis so this triangle does not get caught
        // by the early checks. After a few rounds of clipping though, it ends up with 0 inside.
        let vertices = [
            Point4D::<ClipSpace>::new(-10.70005131, 10.0005131, 1.3, 3.29994869),
            Point4D::<ClipSpace>::new(15.70005131, 0.0, 1.32306385, 1.3),
            Point4D::<ClipSpace>::new(-10.70005131, -10.70005131, 1.3, 3.29994869),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };
        match try_clip(&tri) {
            ClipResult::Clipped(tris) => {
                assert_eq!(tris.len(), 2);
                //assert_eq!(tris[0].vertices, expected);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn small_triangle() {
        let vertices = [
            Point4D::<ClipSpace>::new(1.69605124, -2.40628147, 0.420414984, 2.40628147),
            Point4D::<ClipSpace>::new(1.68917572, -2.40170431, 0.415791571, 2.40170407),
            Point4D::<ClipSpace>::new(1.68629396, -2.36931682, 0.415715098, 2.40162849),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };
        match try_clip(&tri) {
            ClipResult::Clipped(tris) => {
                assert_eq!(tris.len(), 2);
            }
            _ => unreachable!(),
        }
    }
}
