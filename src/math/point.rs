use core::ops::{Mul, Sub, Add};

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

// Deriving these does work atm, so have to do it manually
impl<CS, const N: usize> PartialEq for Point<CS, { N }>
where
    CS: CoordinateSystem,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<CS, const N: usize> Eq for Point<CS, { N }> where CS: CoordinateSystem {}

impl<CS, const N: usize> Point<CS, { N }>
where
    CS: CoordinateSystem,
{
    impl_accessors!(x, y, z, w);

    pub fn xy(&self) -> Point2D {
        Point2D::new(self.x(), self.y())
    }

    pub fn to_vec(self) -> Vector<CS, N> {
        self.0
    }
}

pub fn origin() -> Point3D<WorldSpace> {
    Point3D::<WorldSpace>::new(0.0, 0.0, 0.0)
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

impl<CS> Point3D<CS>
where
    CS: CoordinateSystem,
{
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(vec3::<CS>(x, y, z))
    }

    pub fn extend(&self, w: f32) -> Point4D<CS> {
        Point::<CS, 4>(self.0.extend(w))
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

impl<CS, const N: usize> Mul<f32> for Point<CS, { N }>
where
    CS: CoordinateSystem,
{
    type Output = Point<CS, { N }>;

    fn mul(self, other: f32) -> Self::Output {
        Self(self.0 * other)
    }
}

impl<CS, const N: usize> Add<Vector<CS, {N}>> for Point<CS, { N }>
where
    CS: CoordinateSystem,
{
    type Output = Point<CS, { N }>;

    fn add(self, other: Vector<CS, {N}>) -> Self::Output {
        Self(self.0 + other)
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
