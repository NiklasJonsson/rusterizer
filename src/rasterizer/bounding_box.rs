use std::f32;

use crate::math::Point2D;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelBoundingBox {
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
