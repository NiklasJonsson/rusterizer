use core::ops::Sub;

use core::marker::PhantomData;

use std::convert::TryInto;

#[derive(Copy, Clone)]
struct Vector<const N: usize>([f32; N]);

// TODO: Can we implement the math operations generically over the size?
impl<const N: usize> Vector<{ N }> {
    pub fn dot(self, other: Vector<{ N }>) -> f32 {
        self.0
            .iter()
            .zip(other.0.iter())
            .fold(0.0, |acc, (elem0, elem1)| elem0 * elem1 + acc)
    }

    fn x(&self) -> f32 {
        self.0[0]
    }
    fn y(&self) -> f32 {
        self.0[1]
    }
    fn z(&self) -> f32 {
        self.0[2]
    }
}

impl<const N: usize> std::fmt::Debug for Vector<{ N }> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .0
            .iter()
            .map(|elem| format!("{:?}", elem))
            // How many chars per float? No idea, 32 should be enough
            .fold(String::with_capacity(N * 32), |mut acc, s| {
                acc.push_str(&s);
                acc
            });
        write!(f, "Vector<{}>: {}", N, s)
    }
}

macro_rules! impl_accessor {
    ($name: ident) => {
        pub fn $name(&self) -> f32 {
            self.backing_vec.$name()
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

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    backing_vec: Vector<2>,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            backing_vec: Vector::<2>([x, y]),
        }
    }

    pub fn dot(self, other: Vec2) -> f32 {
        self.backing_vec.dot(other.backing_vec)
    }

    pub fn cross(self, other: Vec2) -> f32 {
        self.x() * other.y() - other.x() * self.y()
    }

    impl_accessors!(x, y);
}

#[derive(Debug, Copy, Clone)]
pub struct Point2D {
    backing_vec: Vector<2>,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            backing_vec: Vector::<2>([x, y]),
        }
    }
    impl_accessors!(x, y);
}

impl Sub<Point2D> for Point2D {
    type Output = Vec2;

    fn sub(self, other: Point2D) -> Vec2 {
        let v0 = self.backing_vec;
        let v1 = other.backing_vec;
        let backing_vec = Vector::<2>([v0.x() - v1.x(), v0.y() - v1.y()]);
        Vec2 { backing_vec }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Point3D<CS> {
    backing_vec: Vector<3>,
    coordinate_space: PhantomData<CS>,
}

impl<CS> Point3D<CS> {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let backing_vec = Vector::<3>([x, y, z]);
        Self {
            backing_vec,
            coordinate_space: PhantomData,
        }
    }
    impl_accessors!(x, y, z);
}

#[derive(Debug, Copy, Clone)]
pub struct Vec3<CS> {
    backing_vec: Vector<3>,
    coordinate_space: PhantomData<CS>,
}

impl<CS> Vec3<CS> {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let backing_vec = Vector::<3>([x, y, z]);
        Self {
            backing_vec,
            coordinate_space: PhantomData,
        }
    }

    pub fn cross(self, other: Vec3<CS>) -> Vec3<CS> {
        let v0 = self.backing_vec.0;
        let v1 = other.backing_vec.0;
        let array = [
            v0[1] * v1[2] - v0[2] * v1[1],
            v0[2] * v1[0] - v0[0] * v1[2],
            v0[0] * v1[1] - v0[1] * v1[0],
        ];
        Vec3::<CS> {
            backing_vec: Vector::<3>(array),
            coordinate_space: PhantomData,
        }
    }
    impl_accessors!(x, y, z);
}

impl<CS> Sub<Point3D<CS>> for Point3D<CS> {
    type Output = Vec3<CS>;

    fn sub(self, other: Point3D<CS>) -> Vec3<CS> {
        let v0 = self.backing_vec;
        let v1 = other.backing_vec;
        let backing_vec = Vector::<3>([v0.x() - v1.x(), v0.y() - v1.y(), v0.z() - v1.z()]);
        Vec3::<CS> {
            backing_vec,
            coordinate_space: PhantomData,
        }
    }
}

struct WorldSpace;
struct CameraSpace;
struct ProjectionSpace;
struct NDC;

trait CoordinateSpace {}

impl CoordinateSpace for WorldSpace {}
impl CoordinateSpace for CameraSpace {}
impl CoordinateSpace for ProjectionSpace {}
impl CoordinateSpace for NDC {}

struct Matrix<const N: usize> {
    array: [f32; N],
}

struct Matrix4<CSF, CST> {
    backing_matrix: Matrix<16>,
    from_coordinate_space: PhantomData<CSF>,
    to_coordinate_space: PhantomData<CST>,
}

fn project(near: f32, far: f32, aspect_ration: f32) -> Matrix4<CameraSpace, ProjectionSpace> {
    unimplemented!();
}
