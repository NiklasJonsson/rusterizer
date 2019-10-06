use core::ops::Add;
use core::ops::Mul;

use crate::math::*;

#[derive(Debug, Copy, Clone, Default)]
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

#[derive(Debug, Default, Clone, Copy)]
pub struct VertexAttribute {
    pub color: Color,
}

impl From<Color> for VertexAttribute {
    fn from(other: Color) -> VertexAttribute {
        VertexAttribute { color: other }
    }
}

impl Mul<f32> for VertexAttribute {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        (self.color * scalar).into()
    }
}

impl Add for VertexAttribute {
    type Output = Self;
    fn add(self, other: VertexAttribute) -> Self::Output {
        (self.color + other.color).into()
    }
}

pub type Vertex<CS: CoordinateSystem> = Point<CS, 4>;

pub struct Triangle<CS>
where
    CS: CoordinateSystem,
{
    pub vertices: [Vertex<CS>; 3],
    pub vertex_attributes: [VertexAttribute; 3],
}

impl<CSF, CST> Mul<Mat3<CSF, CST>> for Triangle<CSF>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    type Output = Triangle<CST>;
    fn mul(self, other: Mat3<CSF, CST>) -> Triangle<CST> {
        unimplemented!();
    }
}

impl<CSF, CST> Mul<Triangle<CSF>> for Mat4<CSF, CST>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    type Output = Triangle<CST>;
    fn mul(self, other: Triangle<CSF>) -> Triangle<CST> {
        unimplemented!();
        /*
        let vertices = other.vertices
            .iter()
            .map(|&p| self * p)
            .collect::<Vec<_>>()[0..3];

        Triangle::<CST> {
            vertices,
            ..self
        }
        */
    }
}

impl<CS> std::fmt::Debug for Triangle<CS>
where
    CS: PrintableType + CoordinateSystem,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Triangle: {:?}\n", self.vertices);
        write!(f, "{:?} ", self.vertex_attributes)
    }
}
