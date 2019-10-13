pub mod matrix;
pub mod point;
pub mod vector;
pub use crate::math::matrix::*;
pub use crate::math::point::*;
pub use crate::math::vector::*;

pub trait PrintableType {
    const NAME: &'static str;
}

#[derive(Copy, Clone)]
pub struct Any2D;

/// The transformations below oncur in the following order(with transform):
/// World  ->  Camera   ->   Clip        ->         NDC    ->    Screen
///       view      projection    perspective_divide  mul_w_viewport

/// The coordinate system in which the models/triangle are position relative towards
/// eachother and the camera.
#[derive(Copy, Clone)]
pub struct WorldSpace;

/// Similar to WorldSpace, except the origin is at the position of the camera.
#[derive(Copy, Clone)]
pub struct CameraSpace;

/// This space ranges from -1, 1 and everything that is outside may be clipped
#[derive(Copy, Clone)]
pub struct ClipSpace;

/// Normalized Device Coordinates
#[derive(Copy, Clone)]
pub struct NDC;

/// x: [0..screen_width] and y: [0..screen_height]
#[derive(Copy, Clone)]
pub struct ScreenSpace;

pub trait CoordinateSystem: Copy + Clone {}

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

pub fn project(near: f32, far: f32, aspect_ration: f32) -> Mat4<CameraSpace, ClipSpace> {
    Mat4::identity()
}
