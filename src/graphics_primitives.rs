use core::ops::Add;
use core::ops::Div;
use core::ops::Mul;
use core::ops::Sub;

use crate::color::Color;
use crate::math::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct VertexAttribute {
    pub color: Color,
    pub uvs: [f32; 2],
}

impl From<(Color, [f32; 2])> for VertexAttribute {
    fn from((color, uvs): (Color, [f32; 2])) -> Self {
        VertexAttribute { color, uvs }
    }
}

impl Mul<f32> for VertexAttribute {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        let color = self.color * scalar;
        let uvs = [self.uvs[0] * scalar, self.uvs[1] * scalar];

        Self { color, uvs }
    }
}

impl Div<f32> for VertexAttribute {
    type Output = Self;

    fn div(self, scalar: f32) -> Self::Output {
        let color = self.color / scalar;
        let uvs = [self.uvs[0] / scalar, self.uvs[1] / scalar];

        Self { color, uvs }
    }
}

impl Add for VertexAttribute {
    type Output = Self;
    fn add(self, other: VertexAttribute) -> Self::Output {
        let color = self.color + other.color;
        let uvs = [self.uvs[0] + other.uvs[0], self.uvs[1] + other.uvs[1]];

        Self { color, uvs }
    }
}

impl Sub for VertexAttribute {
    type Output = Self;
    fn sub(self, other: VertexAttribute) -> Self::Output {
        let color = self.color - other.color;
        let uvs = [self.uvs[0] - other.uvs[0], self.uvs[1] - other.uvs[1]];

        Self { color, uvs }
    }
}

const N_VERTICES: usize = 3;
#[derive(Clone)]
pub struct Triangle<CS>
where
    CS: CoordinateSystem,
{
    pub vertices: [Point4D<CS>; N_VERTICES],
    pub vertex_attributes: [VertexAttribute; N_VERTICES],
}

impl<CSF, CST> Mul<Triangle<CSF>> for Mat4<CSF, CST>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    type Output = Triangle<CST>;
    fn mul(self, other: Triangle<CSF>) -> Triangle<CST> {
        let Triangle {
            vertices: verts,
            vertex_attributes: attrs,
        } = other;
        let vertices = [self * verts[0], self * verts[1], self * verts[2]];

        Triangle::<CST> {
            vertices,
            vertex_attributes: attrs,
        }
    }
}

impl<CS> std::fmt::Debug for Triangle<CS>
where
    CS: PrintableType + CoordinateSystem,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Triangle:")?;
        for (i, (v, va)) in self
            .vertices
            .iter()
            .zip(self.vertex_attributes.iter())
            .enumerate()
        {
            write!(f, "  {}:\n    {:?}\n    {:?}\n", i, v, va)?;
        }

        Ok(())
    }
}
