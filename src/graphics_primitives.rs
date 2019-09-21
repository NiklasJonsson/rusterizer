use core::ops::Add;
use core::ops::Mul;

use crate::math_primitives::*;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn to_rgba(&self) -> u32 {
        (self.r * 255.0) as u32
            | ((self.g * 255.0) as u32) << 8
            | ((self.b * 255.0) as u32) << 16
            | ((self.a * 255.0) as u32) << 24
    }

    pub fn to_bgra(&self) -> u32 {
        (self.b * 255.0) as u32
            | ((self.g * 255.0) as u32) << 8
            | ((self.r * 255.0) as u32) << 16
            | ((self.a * 255.0) as u32) << 24
    }

    pub fn red() -> Color {
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub fn green() -> Color {
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub fn blue() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
            a: self.a * scalar,
        }
    }
}

impl Add<Color> for Color {
    type Output = Color;
    fn add(self, other: Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a,
        }
    }
}

#[derive(Debug)]
pub struct VertexAttribute {
    color: Color,
}

impl From<Color> for VertexAttribute {
    fn from(other: Color) -> VertexAttribute {
        VertexAttribute { color: other }
    }
}

#[derive(Debug)]
pub struct Triangle {
    pub vertices: [Point2D; 3],
    normals: [Vec2; 3],
    pub vertex_attributes: [VertexAttribute; 3],
    area: f32,
}

impl Triangle {
    fn area(vertices: &[Point2D; 3]) -> f32 {
        (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]) * 0.5
    }

    pub fn new(vertices: [Point2D; 3], vertex_attributes: [VertexAttribute; 3]) -> Triangle {
        // Clockwise edge equations
        // To have the normals all pointing towards the inner part of the triangle,
        // they all need to have their positive halfspace to the right of the triangle.
        // If we wanted counter-clockwise, then we switch signs on both x and y of normals
        // (and also switch order for v computations above. Note that coordinate system
        // starts in upper left corner.

        let v0 = vertices[1] - vertices[0];
        let v1 = vertices[2] - vertices[1];
        let v2 = vertices[0] - vertices[2];
        let n0 = Vec2::new(-v0.y(), v0.x());
        let n1 = Vec2::new(-v1.y(), v1.x());
        let n2 = Vec2::new(-v2.y(), v2.x());

        let normals = [n0, n1, n2];
        let area = Triangle::area(&vertices);

        Triangle {
            vertices,
            normals,
            vertex_attributes,
            area,
        }
    }

    pub fn is_point_inside(&self, point: Point2D) -> bool {
        // Based on edge equations
        self.normals
            .iter()
            .zip(self.vertices.iter())
            .fold(true, |acc, (&n, &p)| (n.dot(point - p) >= 0.0) && acc)
    }

    pub fn interpolate_color_for(&self, point: Point2D) -> Color {
        assert!(self.is_point_inside(point));
        let barycentric0 = Triangle::area(&[self.vertices[1], self.vertices[2], point]) / self.area;
        let barycentric1 = Triangle::area(&[self.vertices[2], self.vertices[0], point]) / self.area;
        let barycentric2 = 1.0 - barycentric0 - barycentric1;

        self.vertex_attributes[0].color * barycentric0
            + self.vertex_attributes[1].color * barycentric1
            + self.vertex_attributes[2].color * barycentric2
    }
}
