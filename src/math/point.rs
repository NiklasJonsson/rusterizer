use core::ops::Sub;
use core::ops::Mul;

use crate::math::*;

macro_rules! impl_accessor {
    ($name: ident) => {
        pub fn $name(&self) -> f32 {
            (self.0).$name()
        }
    }
}

macro_rules! impl_accessors {
    ( $( $name: ident),* ) => {
        $(
            impl_accessor!($name);
        )*
    }
}



#[derive(Copy, Clone)]
pub struct Point<CS: CoordinateSystem, const N: usize>(Vector<CS, {N}>);
pub type Point2D = Point<Any2D, 2>;
pub type Point3D<CS: CoordinateSystem> = Point<CS, 3>;
pub type Point4D<CS: CoordinateSystem> = Point<CS, 4>;

impl<CS, const N: usize> Point<CS, {N}>
    where CS: CoordinateSystem,
{
    impl_accessors!(x, y, z, w);
}

impl Point2D
{
    pub fn new(x: f32, y: f32) -> Self {
        Self(vec2(x, y))
    }
}


impl<CS> Point4D<CS>
    where CS: CoordinateSystem,
{
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(vec4::<CS>(x, y, z, w))
    }
}

impl Sub<Point2D> for Point2D
{
    type Output = Vec2;

    fn sub(self, other: Point2D) -> Vec2 {
        let v0 = self.0;
        let v1 = other.0;
        vec2(v0.x() - v1.x(), v0.y() - v1.y())
    }
}

impl<CS> Sub<Point<CS, 4>> for Point<CS, 4>
where CS: CoordinateSystem
{
    type Output = Vec4<CS>;

    fn sub(self, other: Self) -> Self::Output {
        vec4(self.x() - other.x(), self.y() - other.y(), self.z() - other.z(), 0.0)
    }
}

impl<CS, const N: usize> std::fmt::Debug for Point<CS, {N}>
    where CS: PrintableType + CoordinateSystem,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<CSF, CST> Mul<Point4D<CSF>> for Mat4<CSF, CST>
    where CSF: CoordinateSystem,
          CST: CoordinateSystem,
{
    type Output = Point4D<CST>;
    fn mul(self, other: Point4D<CSF>) -> Self::Output {
        Point::<CST, 4>(self * other.0)
    }
}

