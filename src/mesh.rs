use crate::color::Color;
use crate::graphics_primitives::*;
use crate::math::*;

pub struct Mesh<CS>
where
    CS: CoordinateSystem,
{
    pub vertices: Vec<Point3D<CS>>,
    pub indices: Vec<usize>,
    pub attributes: Vec<VertexAttribute>,
}

pub fn centered_quad<CS>(width: f32) -> Mesh<CS>
where
    CS: CoordinateSystem,
{
    let vertices = vec![
        Point3D::new(-width / 2.0, width / 2.0, 2.0),
        Point3D::new(width / 2.0, width / 2.0, 2.0),
        Point3D::new(width / 2.0, -width / 2.0, 2.0),
        Point3D::new(-width / 2.0, -width / 2.0, 2.0),
    ];

    let attributes = vec![
        (Color::red(), [0.0, 0.0]).into(),
        (Color::blue(), [1.0, 0.0]).into(),
        (Color::green(), [1.0, 1.0]).into(),
        (Color::white(), [0.0, 1.0]).into(),
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    Mesh::<CS> {
        vertices,
        indices,
        attributes,
    }
}

pub fn triangle<CS>() -> Mesh<CS>
where
    CS: CoordinateSystem,
{
    let vertices = vec![
        Point3D::new(-1.0, -1.0, 2.0),
        Point3D::new(0.0, 1.0, 2.0),
        Point3D::new(1.0, -1.0, 2.0),
    ];

    let attributes = vec![
        (Color::red(), [0.0, 1.0]).into(),
        (Color::blue(), [1.0, 0.0]).into(),
        (Color::green(), [1.0, 1.0]).into(),
    ];

    let indices = vec![0, 1, 2];
    Mesh::<CS> {
        vertices,
        indices,
        attributes,
    }
}

pub fn cube<CS>(width: f32) -> Mesh<CS>
where
    CS: CoordinateSystem
{
    let vertices = vec![
        // Front
        Point3D::new(-0.5, 0.5, -0.5) * width,
        Point3D::new(0.5, 0.5, -0.5) * width,
        Point3D::new(0.5, -0.5, -0.5) * width,
        Point3D::new(-0.5, -0.5, -0.5) * width,

        // Back
        Point3D::new(0.5, 0.5, 0.5) * width,
        Point3D::new(-0.5, 0.5, 0.5) * width,
        Point3D::new(-0.5, -0.5, 0.5) * width,
        Point3D::new(0.5, -0.5, 0.5) * width,

        // Left
        Point3D::new(-0.5, 0.5, 0.5) * width,
        Point3D::new(-0.5, 0.5, -0.5) * width,
        Point3D::new(-0.5, -0.5, -0.5) * width,
        Point3D::new(-0.5, -0.5, 0.5) * width,

        // Right
        Point3D::new(0.5, 0.5, -0.5) * width,
        Point3D::new(0.5, 0.5, 0.5) * width,
        Point3D::new(0.5, -0.5, 0.5) * width,
        Point3D::new(0.5, -0.5, -0.5) * width,

        // Top
        Point3D::new(-0.5, 0.5, -0.5) * width,
        Point3D::new(-0.5, 0.5, 0.5) * width,
        Point3D::new(0.5, 0.5, 0.5) * width,
        Point3D::new(0.5, 0.5, -0.5) * width,

        // Bottom
        Point3D::new(-0.5, -0.5, 0.5) * width,
        Point3D::new(-0.5, -0.5, -0.5) * width,
        Point3D::new(0.5, -0.5, -0.5) * width,
        Point3D::new(0.5, -0.5, 0.5) * width,
    ];

    let mut indices = Vec::new();
    for i in 0..6 {
        let offset = i * 4;
        let mut add_triangle = |i: usize, j: usize, k: usize| {
            indices.push(offset + i);
            indices.push(offset + j);
            indices.push(offset + k);
        };
        add_triangle(0, 1, 2);
        add_triangle(0, 2, 3);
    }

    let mut colors = Vec::new();
    for i in 0..vertices.len() {
        colors.push(
            match i % 3 {
            0 => Color::red(),
            1 => Color::blue(),
            _ => Color::green(),
            },
        );
    }

    let mut tex_coords = Vec::with_capacity(vertices.len());
    assert_eq!(vertices.len() % 4, 0);
    for _ in 0..vertices.len() / 4 {
        tex_coords.push([0.0, 0.0]);
        tex_coords.push([1.0, 0.0]);
        tex_coords.push([1.0, 1.0]);
        tex_coords.push([0.0, 1.0]);
    }

    assert_eq!(tex_coords.len(), vertices.len());
    assert_eq!(tex_coords.len(), colors.len());

    let attributes = colors.into_iter().zip(tex_coords.into_iter()).map(|v| v.into()).collect::<Vec<_>>();


    Mesh::<CS> {
        vertices,
        indices,
        attributes,
    }
}
