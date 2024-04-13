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
        intersection: Point4D<ClipSpace>,
        line_param: f32,
    },
    SecondInside {
        intersection: Point4D<ClipSpace>,
        line_param: f32,
    },
}

const CULL_DEGENERATE_TRIANGLE_AREA_EPS: f32 = 0.000001;

fn old_intersect(
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
            intersection,
            line_param,
        };
    }

    debug_assert!(p1_signed_dist >= 0.0);
    Intersection::SecondInside {
        intersection,
        line_param,
    }
}

#[derive(Clone, Copy)]
enum ClipPlane {
    LEFT,
    RIGHT,
    BOTTOM,
    TOP,
    NEAR,
    FAR,
}

// Terminology is from ther Sutherland-Hodgman paper. In Blinn, it is called boundary coordinate.
// If this is positive, the point is inside the view volume for this plane, if it is negative, it is outside.
// NOTE: As per the blinn paper, this is only proportional to the distance between the plane and the point
// and should only be used for the signedness or as a term in the intersection calculation.
fn distance_measure(plane: ClipPlane, p: &Point4D<ClipSpace>) -> f32 {
    match plane {
        ClipPlane::LEFT => p.w() + p.x(),
        ClipPlane::RIGHT => p.w() - p.x(),
        ClipPlane::BOTTOM => p.w() + p.y(),
        ClipPlane::TOP => p.w() - p.y(),
        ClipPlane::NEAR => p.w() + p.z(),
        ClipPlane::FAR => p.w() - p.z(),
    }
}

/// Compute the intersection between p0 and p1, using precomputed "distance measures", see the `distance_measure` function.
/// Returns the intersection point and the alpha in the parametric line segment equation intersection_point = (1 - alpha) * p0 + alpha * p1
/// NOTE: This function only works if there is an intersection between the two points.
fn compute_intersection(
    p0: Point4D<ClipSpace>,
    p0_distance_measure: f32,
    p1: Point4D<ClipSpace>,
    p1_distance_measure: f32,
) -> (Point4D<ClipSpace>, f32) {
    let alpha = p0_distance_measure / (p0_distance_measure - p1_distance_measure);
    ((1.0 - alpha) * p0 + alpha * p1, alpha)
}

const CLIP_PLANES: [ClipPlane; 6] = [
    ClipPlane::LEFT,
    ClipPlane::RIGHT,
    ClipPlane::BOTTOM,
    ClipPlane::TOP,
    ClipPlane::NEAR,
    ClipPlane::FAR,
];

pub fn try_clip(triangle: &Triangle<ClipSpace>) -> ClipResult {
    if super::triangle_2x_area(&triangle.vertices).abs() < CULL_DEGENERATE_TRIANGLE_AREA_EPS {
        return ClipResult::Outside;
    }

    // Clip the triangle against the NDC cube but in clip-space, where the NDC cube (in clip-space) is:
    // -w <= x,y,z <= w
    // (per-point, i.e. w is different for every point in the triangle)
    // The following code is using the Sutherland-Hodgman algorithm from this paper:
    // https://dl.acm.org/doi/pdf/10.1145/360767.360802
    // but there is some additional explanation in this paper by Blinn:
    // https://dl.acm.org/doi/pdf/10.1145/800248.807398
    // that I think is a bit easier to understand.
    //
    // A SO answer with some formulas: https://stackoverflow.com/questions/60910464/at-what-stage-is-clipping-performed-in-the-graphics-pipeline
    // Relevant part from "Trip through the graphics pipeline": https://fgiesen.wordpress.com/2011/07/05/a-trip-through-the-graphics-pipeline-2011-part-5/
    // which also talks about guard-band clipping.

    // Fast checks!
    // There are only comparisons and boolean ops which means we can skip the divisions in the clipping.
    // If all x, all y and all z coords are inside w, the triangle is inside the volume, no clipping needed.
    // If all x or all y or all z coords of the triangle are outside 'w', then the triangle is outside and we cull it, no clipping needed.
    let mut inside = [true; 3];
    let mut outside = [true; 3];
    for v in triangle.vertices.iter() {
        inside[0] &= v.x() >= -v.w() && v.x() <= v.w();
        inside[1] &= v.y() >= -v.w() && v.y() <= v.w();
        inside[2] &= v.z() >= -v.w() && v.z() <= v.w();

        outside[0] &= v.x() < -v.w() || v.x() > v.w();
        outside[1] &= v.y() < -v.w() || v.y() > v.w();
        outside[2] &= v.z() < -v.w() || v.z() > v.w();
    }

    if outside.into_iter().any(|x| x) {
        return ClipResult::Outside;
    }

    if inside.into_iter().all(|x| x) {
        return ClipResult::Inside;
    }

    // We now have a triangle that is partially inside the viewing volume, which means it needs to be clipped.
    // There are six planes we want to clip defined as x - w = 0 and x + w = 0 and similarly for y and z.

    // Here, the Sutherland-Hodgman algorithm starts.
    let mut out_vertices: Vec<Point4D<ClipSpace>> = triangle.vertices.to_vec();
    let mut out_attrs: Vec<VertexAttribute> = triangle.vertex_attributes.to_vec();

    for plane in CLIP_PLANES {
        let in_vertices = out_vertices.clone();
        let in_attrs = out_attrs.clone();
        out_attrs.clear();
        out_vertices.clear();

        let mut prev_distance_measure: f32 = distance_measure(plane, in_vertices.last().unwrap());
        for (i, (cur_vert, cur_attr)) in in_vertices.iter().zip(in_attrs.iter()).enumerate() {
            let prev_i = (i + in_vertices.len() - 1) % in_vertices.len();
            let prev_vert = in_vertices[prev_i];
            let prev_attr = in_attrs[prev_i];
            let cur_distance_measure = distance_measure(plane, cur_vert);
            match (prev_distance_measure > 0.0, cur_distance_measure > 0.0) {
                (true, true) => {
                    out_vertices.push(*cur_vert);
                    out_attrs.push(*cur_attr);
                }
                (true, false) => {
                    let (intersection, interpolation_factor) = compute_intersection(
                        prev_vert,
                        prev_distance_measure,
                        *cur_vert,
                        cur_distance_measure,
                    );
                    out_vertices.push(intersection);
                    out_attrs.push((*cur_attr - prev_attr) * interpolation_factor + prev_attr);
                }
                (false, true) => {
                    let (intersection, interpolation_factor) = compute_intersection(
                        prev_vert,
                        prev_distance_measure,
                        *cur_vert,
                        cur_distance_measure,
                    );

                    out_vertices.push(intersection);
                    out_attrs.push((*cur_attr - prev_attr) * interpolation_factor + prev_attr);
                    out_vertices.push(*cur_vert);
                    out_attrs.push(*cur_attr);
                }
                (false, false) => {
                    continue;
                }
            }
            prev_distance_measure = cur_distance_measure;
        }
    }

    // This can happen if even though initially, one or more points are inside, through clipping,
    // they end up outside.
    /*     if out_vertices.is_empty() {
           return ClipResult::Outside;
       }

    */
    debug_assert!(!out_vertices.is_empty());
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

pub fn try_clip_old(triangle: &Triangle<ClipSpace>) -> ClipResult {
    if super::triangle_2x_area(&triangle.vertices).abs() < CULL_DEGENERATE_TRIANGLE_AREA_EPS {
        return ClipResult::Outside;
    }

    // Fast checks:
    // If all x, all y and all z coords are inside w, the triangle is inside the volume.
    // If all x or all y or all z coords of the triangle are outside 'w', then the triangle is outside.
    let mut inside = [true; 3];
    let mut outside = [true; 3];
    for v in triangle.vertices.iter() {
        inside[0] &= v.x() >= -v.w() && v.x() <= v.w();
        inside[1] &= v.y() >= -v.w() && v.y() <= v.w();
        inside[2] &= v.z() >= -v.w() && v.z() <= v.w();

        outside[0] &= v.x() < -v.w() || v.x() > v.w();
        outside[1] &= v.y() < -v.w() || v.y() > v.w();
        outside[2] &= v.z() < -v.w() || v.z() > v.w();
    }

    if outside.into_iter().any(|x| x) {
        return ClipResult::Outside;
    }

    if inside.into_iter().all(|x| x) {
        return ClipResult::Inside;
    }

    // START HERE:
    // This seems to be based on a general plane/line intersection definitions
    // and we don't use the w coordinate comparison optimization
    // 1. Read up on plane/line intersection
    // 2. Verify the below algo
    // 3. Try to combine the "fast checks" with sotherland-hodgeman that actually uses w
    //   Goals: Bounded allocation, caller allocates (preferable on stack but doesn't matter), add sources! Make interpolation look nice.

    // We now have a triangle that is partially inside the viewing volume, which means it needs to be clipped.

    // With these definitions, the positive half space points to inside the bounding box.
    // => A point is inside for dot() > 0.0
    const CLIP_PLANES: [Vec4<ClipSpace>; 6] = [
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
    for clip_plane in CLIP_PLANES.iter() {
        let in_vertices = out_vertices.clone();
        let in_attrs = out_attrs.clone();
        out_attrs.clear();
        out_vertices.clear();
        for (i, (vert, attr)) in in_vertices.iter().zip(in_attrs.iter()).enumerate() {
            let prev_i = (i + in_vertices.len() - 1) % in_vertices.len();
            let prev_vert = in_vertices[prev_i];
            let prev_attr = in_attrs[prev_i];
            match old_intersect(clip_plane, &prev_vert, vert) {
                Intersection::BothOutside => continue,
                Intersection::BothInside => {
                    out_vertices.push(*vert);
                    out_attrs.push(*attr);
                }
                Intersection::FirstInside {
                    intersection,
                    line_param,
                } => {
                    out_vertices.push(intersection);
                    // Interpolate
                    out_attrs.push((*attr - prev_attr) * line_param + prev_attr);
                }
                Intersection::SecondInside {
                    intersection,
                    line_param,
                } => {
                    out_vertices.push(intersection);
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
            old_intersect(&clip_plane, &p0, &p1),
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

    // START HERE:
    // 1. There is a failing test in the unit tests in this file, this should be fixed.
    #[test]
    fn test_clipped_tris_are_inside() {
        // Test that clipping the result of the clipping are not clipped again...
        let vertices = [
            Point4D::<ClipSpace>::new(2.07009363, -3.05162668, 0.985171258, 2.96541810),
            Point4D::<ClipSpace>::new(2.07487488, -2.61568260, 1.03832412, 3.01804209),
            Point4D::<ClipSpace>::new(2.07427669, -2.86637640, 1.15170527, 3.13029504),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };

        let ClipResult::Clipped(clipped) = try_clip(&tri) else {
            unreachable!("Expected the triangle to be clipped");
        };

        for t in clipped {
            assert!(std::matches!(try_clip(&t), ClipResult::Inside));
        }
    }
}
