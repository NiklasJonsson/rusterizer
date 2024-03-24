use crate::graphics_primitives::{Triangle, VertexAttribute};
use crate::math::point::*;
use crate::math::ClipSpace;

#[derive(Debug, Clone)]
pub enum ClipResult {
    Outside,
    Inside,
    Clipped(Vec<Triangle<ClipSpace>>),
}

const CULL_DEGENERATE_TRIANGLE_AREA_EPS: f32 = 0.000001;

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
// If this is positive, the point is inside the view volume for this plane, if it is negative, it is outside,
// and if it is zero, it is on the plane.
// NOTE: As per the blinn paper, this is only proportional to the distance between the plane and the point
// and should only be used for the signedness or as a term in the intersection calculation.
fn distance_measure(plane: ClipPlane, p: Point4D<ClipSpace>) -> f32 {
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
    // NOTE: W varies per vertex
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
    // There are only comparisons and boolean ops which means we can skip more expensive calculations in the clipping.
    // If all x, all y and all z coords are inside w, the triangle is inside the volume, no clipping needed.
    // If all x or all y or all z coords of the triangle are outside 'w', then the triangle is outside and we cull it, no clipping needed.
    // let mut inside = [true; 3];
    // let mut outside = [true; 3];
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

        for (i, (cur_vert, cur_attr)) in in_vertices.iter().zip(in_attrs.iter()).enumerate() {
            let prev_i = (i + in_vertices.len() - 1) % in_vertices.len();
            let prev_vert = in_vertices[prev_i];
            let prev_attr = in_attrs[prev_i];
            let prev_distance_measure = distance_measure(plane, prev_vert);
            let cur_distance_measure = distance_measure(plane, *cur_vert);
            // The distance measure is zero if the point is on the plane and positive if it is inside the viewing volume.
            match (prev_distance_measure >= 0.0, cur_distance_measure >= 0.0) {
                // Line is inside, no clipping
                (true, true) => {
                    out_vertices.push(*cur_vert);
                    out_attrs.push(*cur_attr);
                }
                // Prev inside, cur outside => Add only intersection as prev was added last time (or will be added, if it is the last).
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
                // Prev outside, cur inside => Add intersection and current, adding a new edge
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
                // Line is outside, discard
                (false, false) => {
                    continue;
                }
            }
        }
    }

    // This can happen if even though initially, one or more points are inside, through clipping,
    // they end up outside.
    if out_vertices.is_empty() {
        return ClipResult::Outside;
    }

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

#[cfg(test)]
mod test {
    use super::*;

    fn dump(verts: &[Point4D<ClipSpace>]) {
        for (i, v) in verts.iter().enumerate() {
            println!("v{} = [{}, {}, {}, {}]", i, v.x(), v.y(), v.z(), v.w());
        }
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
            Point4D::<ClipSpace>::new(-7.062_944_4, 5.062_944_4, 5.060_302, 7.0),
            Point4D::<ClipSpace>::new(-6.062_944_4, 7.062_944_4, 5.060_302, 7.0),
            Point4D::<ClipSpace>::new(-5.062_944_4, 5.062_944_4, 5.060_302, 7.0),
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
            Point4D::<ClipSpace>::new(5.122_790_3, -1.0, 5.060_302, 7.0),
            Point4D::<ClipSpace>::new(7.0, -0.754_419_3, 5.060_302, 7.0),
            Point4D::<ClipSpace>::new(7.0, -1.0, 5.060_302, 7.0),
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
            Point4D::<ClipSpace>::new(-4.700_051_3, -4.700_051_3, 1.323_063_9, 3.299_948_7),
            Point4D::<ClipSpace>::new(-3.700_051_3, -2.700_051_3, 1.323_063_9, 3.299_948_7),
            Point4D::<ClipSpace>::new(-2.700_051_3, -4.700_051_3, 1.323_063_9, 3.299_948_7),
        ];

        let tri = Triangle {
            vertices,
            vertex_attributes: VERTEX_ATTRIBUTES,
        };
        assert!(std::matches!(try_clip(&tri), ClipResult::Outside));
    }

    #[test]
    fn complete_coverage() {
        let vertices = [
            Point4D::<ClipSpace>::new(-10.700_051, 10.000_513, 1.3, 3.299_948_7),
            Point4D::<ClipSpace>::new(15.700_051, 0.0, 1.323_063_9, 1.3),
            Point4D::<ClipSpace>::new(-10.700_051, -10.700_051, 1.3, 3.299_948_7),
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

    #[test]
    fn small_triangle() {
        let vertices = [
            Point4D::<ClipSpace>::new(1.696_051_2, -2.406_281_5, 0.420_414_98, 2.406_281_5),
            Point4D::<ClipSpace>::new(1.689_175_7, -2.401_704_3, 0.415_791_57, 2.401_704),
            Point4D::<ClipSpace>::new(1.686_294, -2.369_316_8, 0.415_715_1, 2.401_628_5),
        ];
        dump(&vertices);

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

    #[test]
    fn test_clipped_tris_are_inside() {
        // Test that clipping the result of the clipping are not clipped again...
        let vertices = [
            Point4D::<ClipSpace>::new(2.070_093_6, -3.051_626_7, 0.985_171_26, 2.965_418),
            Point4D::<ClipSpace>::new(2.074_874_9, -2.615_682_6, 1.038_324_1, 3.018_042),
            Point4D::<ClipSpace>::new(2.074_276_7, -2.866_376_4, 1.151_705_3, 3.130_295),
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
