use core::ops::Mul;

use core::marker::PhantomData;

use crate::math::*;

#[derive(Copy, Clone)]
pub struct Vector<CS: CoordinateSystem, const N: usize> {
    arr: [f32; N],
    _coordinate_system: PhantomData<CS>,
}

// TODO: Implement generic operators as well
impl<CS, const N: usize> Vector<CS, { N }>
    where CS: CoordinateSystem
{
    pub fn dot(self, other: Vector<CS, { N }>) -> f32 {
        self.arr
            .iter()
            .zip(other.arr.iter())
            .fold(0.0, |acc, (elem0, elem1)| elem0 * elem1 + acc)
    }

    pub fn x(&self) -> f32 {
        self.arr[0]
    }
    pub fn y(&self) -> f32 {
        self.arr[1]
    }
    pub fn z(&self) -> f32 {
        self.arr[2]
    }
    pub fn w(&self) -> f32 {
        self.arr[3]
    }

    pub fn len(&self) -> f32 {
        self.arr.iter().fold(0.0, |acc, e| acc + e * e).sqrt()
    }
}

impl<CS, const N: usize> std::fmt::Debug for Vector<CS, { N }>
    where CS: PrintableType + CoordinateSystem,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .arr
            .iter()
            .map(|elem| format!("{:?}", elem))
            // How many chars per float? No idea, 32 should be enough
            .fold(String::with_capacity(N * 32), |mut acc, s| {
                acc.push_str(&s);
                acc
            });
        write!(f, "Vector<{}, {}>: {}", N, CS::NAME, s)
    }
}

/*
impl<CS, const N: usize> Mul<f32> for Vector<CS, {N}>
    where CS: CoordinateSystem,
{
    type Output = Vector<CS, {N}>;
    fn mul(self, other: f32) -> Vector<CS, {N}> {
        let arr = self.arr
            .iter()
            .map(|e| e * other)
            .collect::<Vec<_>>().into();

        Self {
            arr,
            ..self
        }
    }
}
*/

pub type Vec2 = Vector<Any2D, {2}>;
pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vector::<Any2D, {2}>{arr: [x, y], _coordinate_system: PhantomData{}}
}

impl<CS> Vector<CS, {2}>
    where CS: CoordinateSystem
{
    pub fn cross(self, other: Vector<CS, {2}>) -> f32 {
        self.x() * other.y() - other.x() * self.y()
    }
}

pub type Vec3<CS> = Vector<CS, {3}>;
pub fn vec3<CS: CoordinateSystem>(x: f32, y: f32, z: f32) -> Vec3<CS> {
    Vector::<CS, {3}>{arr: [x, y, z], _coordinate_system: PhantomData{}}
}

impl<CS: CoordinateSystem> Vec3<CS> {
    pub fn cross(self, other: Self) -> Self {
        let v0 = self.arr;
        let v1 = other.arr;
        let x =
            v0[1] * v1[2] - v0[2] * v1[1];
        let y =
            v0[2] * v1[0] - v0[0] * v1[2];
        let z =
            v0[0] * v1[1] - v0[1] * v1[0];
        vec3(x, y, z)
    }
}

pub type Vec4<CS> = Vector<CS, {4}>;
pub fn vec4<CS: CoordinateSystem>(x: f32, y: f32, z: f32, w: f32) -> Vec4<CS> {
    Vec4::<CS>{arr: [x, y, z, w], _coordinate_system: PhantomData{}}
}

impl<CSF, CST> Mul<Vec4<CSF>> for Mat4<CSF, CST>
    where CSF: CoordinateSystem,
          CST: CoordinateSystem,
{
    type Output = Vec4<CST>;
    fn mul(self, other: Vec4<CSF>) -> Vec4<CST> {
        unimplemented!();
    }
}


