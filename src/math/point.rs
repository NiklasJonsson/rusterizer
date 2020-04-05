use core::ops::{Div, Mul, Sub};

use crate::math::*;

macro_rules! impl_accessor {
    ($name: ident) => {
        pub fn $name(&self) -> f32 {
            (self.0).$name()
        }
    };
}

macro_rules! impl_accessors {
    ( $( $name: ident),* ) => {
        $(
            impl_accessor!($name);
        )*
    }
}

#[derive(Copy, Clone)]
pub struct Point<CS: CoordinateSystem, const N: usize>(Vector<CS, { N }>);
pub type Point2D = Point<Any2D, 2>;
pub type Point3D<CS> = Point<CS, 3>;
pub type Point4D<CS> = Point<CS, 4>;

impl<CS, const N: usize> Point<CS, { N }>
where
    CS: CoordinateSystem,
{
    impl_accessors!(x, y, z, w);
}

pub fn origin() -> Point4D<WorldSpace> {
    Point4D::<WorldSpace>::new(0.0, 0.0, 0.0, 0.0)
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self(vec2(x, y))
    }
}

impl<CS> Point4D<CS>
where
    CS: CoordinateSystem,
{
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(vec4::<CS>(x, y, z, w))
    }
}

impl<CS, const N: usize> Sub for Point<CS, { N }>
where
    CS: CoordinateSystem,
{
    type Output = Vector<CS, { N }>;

    fn sub(self, other: Self) -> Self::Output {
        self.0 - other.0
    }
}

impl<CS, const N: usize> Div<f32> for Point<CS, { N }>
where
    CS: CoordinateSystem,
{
    type Output = Point<CS, { N }>;

    fn div(self, other: f32) -> Self::Output {
        Self(self.0 / other)
    }
}

impl<CS, const N: usize> std::fmt::Debug for Point<CS, { N }>
where
    CS: PrintableType + CoordinateSystem,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<CSF, CST, const N: usize> Mul<Point<CSF, { N }>> for Matrix<CSF, CST, { N }>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    type Output = Point<CST, { N }>;
    fn mul(self, other: Point<CSF, { N }>) -> Self::Output {
        Point::<CST, { N }>(self * other.0)
    }
}
