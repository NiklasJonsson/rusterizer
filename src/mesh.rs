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

#[allow(unused)]
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

#[allow(unused)]
pub fn triangle<CS>() -> Mesh<CS>
where
    CS: CoordinateSystem,
{
    let vertices = vec![
        Point3D::new(-1.0, -1.0, 2.0),
        Point3D::new(0.0, 1.0, 2.0),
        Point3D::new(8.0, -1.0, 2.0),
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

#[allow(unused)]
pub fn cube<CS>(width: f32) -> Mesh<CS>
where
    CS: CoordinateSystem,
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
        colors.push(match i % 3 {
            0 => Color::red(),
            1 => Color::blue(),
            _ => Color::green(),
        });
    }

    let mut tex_coords = Vec::with_capacity(vertices.len());
    debug_assert_eq!(vertices.len() % 4, 0);
    for _ in 0..vertices.len() / 4 {
        tex_coords.push([0.0, 0.0]);
        tex_coords.push([1.0, 0.0]);
        tex_coords.push([1.0, 1.0]);
        tex_coords.push([0.0, 1.0]);
    }

    debug_assert_eq!(tex_coords.len(), vertices.len());
    debug_assert_eq!(tex_coords.len(), colors.len());

    let attributes = colors
        .into_iter()
        .zip(tex_coords.into_iter())
        .map(|v| v.into())
        .collect::<Vec<_>>();

    Mesh::<CS> {
        vertices,
        indices,
        attributes,
    }
}

#[allow(unused)]
pub fn sphere<CS>(radius: f32) -> Mesh<CS>
where
    CS: CoordinateSystem,
{
    // Left-handed, x right, y up, z forward.
    // phi rotates around y. Theta from (0, 1, 0) to (0, -1, 0)
    // ISO Spherical coordinates
    // Note that phi is sampled once for the beginning and once for the end, to provide proper
    // texture coordinates.
    let n_phi_samples = 17;
    let n_theta_samples = 9;

    let mut vertices = Vec::with_capacity(n_phi_samples * n_theta_samples);
    let mut indices = Vec::new();
    let mut attributes = Vec::new();

    for i in 0..n_theta_samples {
        for j in 0..n_phi_samples {
            let theta_ratio = i as f32 / (n_theta_samples - 1) as f32;
            let phi_ratio = j as f32 / (n_phi_samples - 1) as f32;

            let phi = std::f32::consts::PI * 2.0 * phi_ratio;
            let theta = std::f32::consts::PI * theta_ratio;

            let x = radius * theta.sin() * phi.cos();
            let y = radius * theta.cos();
            let z = radius * theta.sin() * phi.sin();
            vertices.push(Point3D::<CS>::new(x, y, z));

            if i < n_theta_samples - 1 && j < n_phi_samples - 1 {
                indices.push(n_phi_samples * i + j);
                indices.push(n_phi_samples * i + (j + 1));
                indices.push(n_phi_samples * (i + 1) + (j + 1));

                indices.push(n_phi_samples * i + j);
                indices.push(n_phi_samples * (i + 1) + (j + 1));
                indices.push(n_phi_samples * (i + 1) + j);
            }

            let c = Color {
                r: x.abs(),
                g: y.abs(),
                b: z.abs(),
                a: 1.0,
            };

            attributes.push((c, [phi_ratio, theta_ratio]).into());
        }
    }

    Mesh::<CS> {
        vertices,
        indices,
        attributes,
    }
}
