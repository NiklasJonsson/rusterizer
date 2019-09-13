use core::ops::Sub;

#[derive(Debug, Copy, Clone)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Vec2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dot(self, other: Vec2D) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl Sub<Point2D> for Point2D {
    type Output = Vec2D;

    fn sub(self, other: Point2D) -> Vec2D {
        Vec2D {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
