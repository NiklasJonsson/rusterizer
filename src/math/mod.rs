pub mod matrix;
pub mod point;
pub mod transform;
pub mod vector;
pub use crate::math::matrix::*;
pub use crate::math::point::*;
pub use crate::math::transform::*;
pub use crate::math::vector::*;

/// This module contains a very basic implementation of math library mainly for learning purposes.
/// A lot of the types are generic over the coordinate space they are defined in, e.g. a point in
/// WorldSpace. This is mostly an experiment to learn about coordinate systems and rust generics,
/// and I can't say that it has prevented a lot of bugs but there have been a few occasions when
/// it has made the progress through the graphics pipeline more clear to me at least.
///
/// The general idea is that primitives such as points, vectors and triangles (defined in graphics_primitives::) are defined
/// in a coordinate space and to transform them to another, you need to transform them with a matrix that defines a transformation
/// between coordinate systems as part of its type signature, e.g. Mat4<WorldSpace, CameraSpace> would transform a Vec4<WorldSpace>
/// to Vec4<CameraSpace>.

pub trait PrintableType {
    const NAME: &'static str;
}

#[derive(Copy, Clone)]
pub struct Any2D;

/// The transformations below oncur in the following order (with transform):
/// World  ->  Camera   ->   Clip        ->         NDC    ->    Screen
///       view      projection    perspective_divide  viewport_transform

/// The coordinate system in which the models/triangle are position relative towards
/// eachother and the camera. X right, Y up, Z towards screen (left-handed)
#[derive(Copy, Clone)]
pub struct WorldSpace;

/// Similar to WorldSpace, except the origin is at the position of the camera.
/// Also known as view space. Also left-handed. Things are in-front of the camera if their z is
/// negative (in camera space)
#[derive(Copy, Clone)]
pub struct CameraSpace;

/// This space ranges from -w, w for all axes and everything that is outside may be clipped.
/// Also known as projection space.
#[derive(Copy, Clone)]
pub struct ClipSpace;

/// Normalized Device Coordinates, x and y have been divided by the the clip space w coordinate.
/// All axes range from -1, 1.
#[derive(Copy, Clone)]
pub struct NDC;

/// x: [0..screen_width] and y: [0..screen_height]
/// z: [zmin, zmax] as defined by the rasterizer
/// Note that (0, 0) is the upper left corner
#[derive(Copy, Clone)]
pub struct ScreenSpace;

pub trait CoordinateSystem: Copy + Clone + PrintableType {}

impl CoordinateSystem for Any2D {}
impl PrintableType for Any2D {
    const NAME: &'static str = "Any2D";
}

impl CoordinateSystem for WorldSpace {}
impl PrintableType for WorldSpace {
    const NAME: &'static str = "WorldSpace";
}

impl CoordinateSystem for CameraSpace {}
impl PrintableType for CameraSpace {
    const NAME: &'static str = "CameraSpace";
}

impl CoordinateSystem for ClipSpace {}
impl PrintableType for ClipSpace {
    const NAME: &'static str = "ClipSpace";
}

impl CoordinateSystem for NDC {}
impl PrintableType for NDC {
    const NAME: &'static str = "NDC";
}

impl CoordinateSystem for ScreenSpace {}
impl PrintableType for ScreenSpace {
    const NAME: &'static str = "ScreenSpace";
}

// See https://www.songho.ca/opengl/gl_projectionmatrix.html for derivation
pub fn project(
    near: f32,
    far: f32,
    aspect_ratio: f32,
    vert_fov: f32,
) -> Mat4<CameraSpace, ClipSpace> {
    assert!(near > 0.0);
    let half_width = (vert_fov / 2.0).tan() * near;
    let half_height = aspect_ratio * half_width;

    // Note that camera space is left-handed here, but as the frustrum is symmetric, it yields the
    // same matrix.
    mat4(
        near / half_width,
        0.0,
        0.0,
        0.0,
        0.0,
        near / half_height,
        0.0,
        0.0,
        0.0,
        0.0,
        -(far + near) / (far - near),
        -2.0 * far * near / (far - near),
        0.0,
        0.0,
        -1.0,
        0.0,
    )
}
