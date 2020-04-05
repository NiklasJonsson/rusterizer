use crate::graphics_primitives::*;
use crate::math::*;

pub struct Mesh<CS>
where
    CS: CoordinateSystem,
{
    pub vertices: Vec<Vertex<CS>>,
    pub indices: Vec<usize>,
    pub attributes: Vec<VertexAttribute>,
}

pub fn centered_quad<CS>(width: f32, color: Color) -> Mesh<CS>
where
    CS: CoordinateSystem,
{
    let vertices = vec![
        vertex(-width / 2.0, width / 2.0, 3.0),
        vertex(width / 2.0, width / 2.0, 3.0),
        vertex(width / 2.0, -width / 2.0, 3.0),
        vertex(-width / 2.0, -width / 2.0, 3.0),
    ];

    let attributes = vec![color.into(); 4];

    let indices = vec![0, 1, 2, 0, 2, 3];

    Mesh::<CS> {
        vertices,
        indices,
        attributes,
    }
}