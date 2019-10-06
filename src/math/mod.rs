pub mod matrix;
pub mod point;
pub mod vector;
pub use crate::math::matrix::*;
pub use crate::math::point::*;
pub use crate::math::vector::*;

pub trait PrintableType {
    const NAME: &'static str;
}

#[derive(Copy, Clone, Debug)]
pub struct Any2D;
#[derive(Copy, Clone)]
pub struct WorldSpace;
#[derive(Copy, Clone)]
pub struct CameraSpace;
#[derive(Copy, Clone)]
pub struct ClipSpace;
#[derive(Copy, Clone)]
pub struct NDC;
#[derive(Copy, Clone)]
pub struct ScreenSpace;

pub trait CoordinateSystem {}

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
    unimplemented!();
}
