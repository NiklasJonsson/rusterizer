use core::ops::{Add, Div, Mul, Neg, Sub};

use core::marker::PhantomData;

use crate::math::*;

#[derive(Copy, Clone)]
pub struct Vector<CS: CoordinateSystem, const N: usize> {
    arr: [f32; N],
    coordinate_system: PhantomData<CS>,
}

impl<CS, const N: usize> Vector<CS, { N }>
where
    CS: CoordinateSystem,
{
    pub fn dot(self, other: Vector<CS, { N }>) -> f32 {
        let mut sum = 0.0;
        for (v0, v1) in self.arr.iter().zip(other.arr.iter()) {
            sum += *v0 * *v1;
        }
        sum
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

    pub fn normalized(self) -> Self {
        self / self.len()
    }
}

impl<CS, const N: usize> std::cmp::PartialEq for Vector<CS, { N }>
where
    CS: CoordinateSystem,
{
    fn eq(&self, other: &Self) -> bool {
        self.arr
            .iter()
            .zip(other.arr.iter())
            .fold(true, |acc, (a, b)| acc && a == b)
    }
}

impl<CS, const N: usize> std::cmp::Eq for Vector<CS, { N }> where CS: CoordinateSystem {}

impl<CS, const N: usize> From<[f32; N]> for Vector<CS, { N }>
where
    CS: CoordinateSystem,
{
    fn from(arr: [f32; N]) -> Self {
        Self {
            arr,
            coordinate_system: PhantomData,
        }
    }
}

impl<CS, const N: usize> From<Vector<CS, { N }>> for [f32; N]
where
    CS: CoordinateSystem,
{
    fn from(val: Vector<CS, { N }>) -> Self {
        val.arr
    }
}

impl<CS, const N: usize> Neg for Vector<CS, { N }>
where
    CS: CoordinateSystem,
{
    type Output = Self;
    fn neg(mut self) -> Self {
        for v in self.arr.iter_mut() {
            *v *= -1.0;
        }

        self
    }
}

impl<CS, const N: usize> Mul<f32> for Vector<CS, { N }>
where
    CS: CoordinateSystem,
{
    type Output = Self;
    fn mul(mut self, other: f32) -> Self::Output {
        for v in self.arr.iter_mut() {
            *v *= other;
        }

        self
    }
}

impl<CS, const N: usize> Div<f32> for Vector<CS, { N }>
where
    CS: CoordinateSystem,
{
    type Output = Self;
    fn div(mut self, other: f32) -> Self::Output {
        for v in self.arr.iter_mut() {
            *v /= other;
        }

        self
    }
}

impl<CS, const N: usize> Add for Vector<CS, { N }>
where
    CS: CoordinateSystem,
{
    type Output = Self;
    fn add(mut self, other: Self) -> Self::Output {
        for (a, b) in self.arr.iter_mut().zip(other.arr.iter()) {
            *a += b;
        }

        self
    }
}

impl<CS, const N: usize> Sub for Vector<CS, { N }>
where
    CS: CoordinateSystem,
{
    type Output = Self;
    fn sub(mut self, other: Self) -> Self::Output {
        for (a, b) in self.arr.iter_mut().zip(other.arr.iter()) {
            *a -= b;
        }

        self
    }
}

impl<CS, const N: usize> std::fmt::Debug for Vector<CS, { N }>
where
    CS: PrintableType + CoordinateSystem,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Avoid trailing comma
        let s = self
            .arr
            .iter()
            .map(|elem| format!("{:?}, ", elem))
            // How many chars per float? No idea, 32 should be enough
            .fold(String::with_capacity(N * 32), |mut acc, s| {
                acc.push_str(&s);
                acc
            });
        write!(f, "Vector<{}, {}>: [{}]", CS::NAME, N, s)
    }
}

pub type Vec2 = Vector<Any2D, 2>;
pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vector::<Any2D, 2> {
        arr: [x, y],
        coordinate_system: PhantomData {},
    }
}

impl<CS> Vector<CS, 2>
where
    CS: CoordinateSystem,
{
    pub fn cross(self, other: Vector<CS, 2>) -> f32 {
        self.x() * other.y() - other.x() * self.y()
    }
}

pub type Vec3<CS> = Vector<CS, 3>;
pub fn vec3<CS: CoordinateSystem>(x: f32, y: f32, z: f32) -> Vec3<CS> {
    Vector::<CS, 3> {
        arr: [x, y, z],
        coordinate_system: PhantomData {},
    }
}

impl<CS: CoordinateSystem> Vec3<CS> {
    pub fn cross(self, other: Self) -> Self {
        let v0 = self.arr;
        let v1 = other.arr;
        let x = v0[1] * v1[2] - v0[2] * v1[1];
        let y = v0[2] * v1[0] - v0[0] * v1[2];
        let z = v0[0] * v1[1] - v0[1] * v1[0];
        vec3(x, y, z)
    }

    pub fn extend(&self, w: f32) -> Vec4<CS> {
        vec4(self.arr[0], self.arr[1], self.arr[2], w)
    }
}

pub type Vec4<CS> = Vector<CS, 4>;
pub const fn vec4<CS: CoordinateSystem>(x: f32, y: f32, z: f32, w: f32) -> Vec4<CS> {
    Vec4::<CS> {
        arr: [x, y, z, w],
        coordinate_system: PhantomData {},
    }
}

impl<CSF, CST, const N: usize> Mul<Vector<CSF, { N }>> for Matrix<CSF, CST, { N }>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    type Output = Vector<CST, { N }>;
    fn mul(self, other: Vector<CSF, { N }>) -> Self::Output {
        let Vector {
            arr,
            coordinate_system: _,
        } = other;
        let mut result = arr;
        for (i, r) in result.iter_mut().enumerate() {
            let row: Vector<CSF, { N }> = self.row(i).into();
            *r = row.dot(other);
        }
        Self::Output {
            arr: result,
            coordinate_system: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality() {
        let v = vec3::<WorldSpace>(0.0, 0.0, 0.0).len();
        let v1 = vec3::<WorldSpace>(1.0, 2.0, 3.0).len();
        let v2 = vec3::<WorldSpace>(1.0, 2.0, 3.0).len();
        assert_eq!(v, v);
        assert_eq!(v1, v1);
        assert_eq!(v1, v2);
        assert_ne!(v, v1);
        assert_ne!(v, v2);
        let v3 = vec3::<WorldSpace>(3.0, 2.0, 3.0).len();

        assert_ne!(v3, v);
        assert_ne!(v3, v1);
        assert_ne!(v3, v2);
    }

    #[test]
    fn length() {
        assert_eq!(vec3::<WorldSpace>(0.0, 0.0, 0.0).len(), 0.0);
        assert_eq!(vec3::<WorldSpace>(1.0, 0.0, 0.0).len(), 1.0);
        assert_eq!(vec3::<WorldSpace>(0.0, 1.0, 0.0).len(), 1.0);
        assert_eq!(vec3::<WorldSpace>(0.0, 0.0, 1.0).len(), 1.0);
        assert_eq!(vec3::<WorldSpace>(-1.0, 0.0, 0.0).len(), 1.0);
        assert_eq!(vec3::<WorldSpace>(0.0, -1.0, 0.0).len(), 1.0);
        assert_eq!(vec3::<WorldSpace>(0.0, 0.0, -1.0).len(), 1.0);
        assert_eq!(vec3::<WorldSpace>(3.0, 10.0, 1.0).len(), 10.488089);
    }

    #[test]
    fn normalized() {
        assert_eq!(vec3::<WorldSpace>(3.0, 10.760, 1.0).normalized().len(), 1.0);
        assert_eq!(vec3::<WorldSpace>(8.0, 10.0, 1.0).normalized().len(), 1.0);
        assert_eq!(vec3::<WorldSpace>(3.0, 143.5, 1.0).normalized().len(), 1.0);
        assert_eq!(
            vec3::<WorldSpace>(63.0, 2234.5, -1.0).normalized().len(),
            1.0
        );
        assert_eq!(
            vec3::<WorldSpace>(23.0, -1546.1, 1324.0).normalized().len(),
            1.0
        );
        assert_eq!(
            vec3::<WorldSpace>(99.0, 14.0, -123.0).normalized().len(),
            1.0
        );
    }

    #[test]
    fn neg() {
        assert_eq!(
            -vec3::<WorldSpace>(3.0, 10.760, 1.0),
            vec3::<WorldSpace>(-3.0, -10.760, -1.0)
        );
        assert_eq!(
            -vec3::<WorldSpace>(8.0, 10.0, 1.0),
            vec3::<WorldSpace>(-8.0, -10.0, -1.0)
        );
        assert_eq!(
            -vec3::<WorldSpace>(3.0, 143.5, 1.0),
            vec3::<WorldSpace>(-3.0, -143.5, -1.0)
        );
        assert_eq!(
            -vec3::<WorldSpace>(63.0, 2234.5, -1.0),
            vec3::<WorldSpace>(-63.0, -2234.5, 1.0)
        );
        assert_eq!(
            -vec3::<WorldSpace>(23.0, -1546.1, 1324.0),
            vec3::<WorldSpace>(-23.0, 1546.1, -1324.0)
        );
        assert_eq!(
            -vec3::<WorldSpace>(99.0, 14.0, -123.0),
            vec3::<WorldSpace>(-99.0, -14.0, 123.0)
        );
    }

    #[test]
    fn mul() {
        assert_eq!(
            vec3::<WorldSpace>(3.0, 10.90, 1.0) * 10.0,
            vec3::<WorldSpace>(30.0, 109.0, 10.0)
        );
        assert_eq!(
            vec3::<WorldSpace>(13.0, 10.90, -15.0) * 2.0,
            vec3::<WorldSpace>(26.0, 21.8, -30.0)
        );
        assert_eq!(
            vec3::<WorldSpace>(13.0, 10.90, -15.0) * -3.0,
            vec3::<WorldSpace>(-39.0, -32.699997, 45.0)
        );
    }

    #[test]
    fn div() {
        assert_eq!(
            vec3::<WorldSpace>(3.0, 10.90, 1.0) / 10.0,
            vec3::<WorldSpace>(0.3, 1.0899999, 0.1)
        );
        assert_eq!(
            vec3::<WorldSpace>(13.0, 10.90, -15.0) / 2.0,
            vec3::<WorldSpace>(6.5, 5.45, -7.5)
        );
        assert_eq!(
            vec3::<WorldSpace>(13.0, 10.90, -15.0) / -3.0,
            vec3::<WorldSpace>(-4.333_333_5, -3.6333332, 5.0)
        );
    }

    #[test]
    fn add() {
        let v = [
            vec3::<WorldSpace>(3.0, 10.34, 1.0),
            vec3::<WorldSpace>(13.0, 10.90, -15.0),
            vec3::<WorldSpace>(-10_345.124, 0.9123, -15.0),
        ];

        let expected = [
            vec3::<WorldSpace>(6.0, 20.68, 2.0),
            vec3::<WorldSpace>(16.0, 21.24, -14.0),
            vec3::<WorldSpace>(-10_342.124, 11.2523, -14.0),
            vec3::<WorldSpace>(16.0, 21.24, -14.0),
            vec3::<WorldSpace>(26.0, 21.80, -30.0),
            vec3::<WorldSpace>(-10_332.124, 11.8123, -30.0),
            vec3::<WorldSpace>(-10_342.124, 11.2523, -14.0),
            vec3::<WorldSpace>(-10_332.124, 11.8123, -30.0),
            vec3::<WorldSpace>(-20690.248, 1.8246, -30.0),
        ];
        for i in 0..3 {
            for j in 0..3 {
                assert_eq!(v[i] + v[j], expected[i * 3 + j]);
            }
        }
    }

    #[test]
    fn mat4_mul_identity() {
        let v = [
            vec4::<WorldSpace>(3.0, 10.34, 1.0, 0.0),
            vec4::<WorldSpace>(13.0, 10.90, -15.0, 0.0),
            vec4::<WorldSpace>(-10_345.124, 0.9123, -15.0, 0.0),
            vec4::<WorldSpace>(3.0, 10.34, 1.0, 1.0),
            vec4::<WorldSpace>(13.0, 10.90, -15.0, 1.0),
            vec4::<WorldSpace>(-10_345.124, 0.9123, -15.0, 1.0),
        ];

        let expected = [
            vec4::<WorldSpace>(3.0, 10.34, 1.0, 0.0),
            vec4::<WorldSpace>(13.0, 10.90, -15.0, 0.0),
            vec4::<WorldSpace>(-10_345.124, 0.9123, -15.0, 0.0),
            vec4::<WorldSpace>(3.0, 10.34, 1.0, 1.0),
            vec4::<WorldSpace>(13.0, 10.90, -15.0, 1.0),
            vec4::<WorldSpace>(-10_345.124, 0.9123, -15.0, 1.0),
        ];

        for j in 0..v.len() {
            assert_eq!(Mat4::<WorldSpace>::identity() * v[j], expected[j]);
        }
    }

    #[test]
    fn mat4_mul_translate() {
        let v = [
            vec4::<WorldSpace>(3.0, 10.34, 1.0, 0.0),
            vec4::<WorldSpace>(13.0, 10.90, -15.0, 0.0),
            vec4::<WorldSpace>(-10_345.124, 0.9123, -15.0, 0.0),
            vec4::<WorldSpace>(3.0, 10.34, 1.0, 1.0),
            vec4::<WorldSpace>(13.0, 10.90, -15.0, 1.0),
            vec4::<WorldSpace>(-10_345.124, 0.9123, -15.0, 1.0),
        ];

        let expected = [
            vec4::<WorldSpace>(3.0, 10.34, 1.0, 0.0),
            vec4::<WorldSpace>(13.0, 10.90, -15.0, 0.0),
            vec4::<WorldSpace>(-10_345.124, 0.9123, -15.0, 0.0),
            vec4::<WorldSpace>(7.0, 8.34, 5.5, 1.0),
            vec4::<WorldSpace>(17.0, 8.90, -10.5, 1.0),
            vec4::<WorldSpace>(-10_341.124, -1.0877, -10.5, 1.0),
        ];

        let mat4s = [transform::translate::<WorldSpace>(4.0, -2.0, 4.5)];

        for i in 0..1 {
            for j in 0..v.len() {
                assert_eq!(mat4s[i] * v[j], expected[i * 3 + j]);
            }
        }
    }

    #[test]
    fn mat4_mul_rotate_lh() {
        let v = [
            vec3::<WorldSpace>(1.0, 0.0, 0.0),
            vec3::<WorldSpace>(0.0, 1.0, 0.0),
            vec3::<WorldSpace>(0.0, 0.0, 1.0),
        ];

        let expected = [
            vec3::<WorldSpace>(1.0, 0.0, 0.0),
            vec3::<WorldSpace>(0.0, -0.00000004371139, 1.0),
            vec3::<WorldSpace>(0.0, -1.0, -0.00000004371139),
            vec3::<WorldSpace>(-0.00000004371139, 0.0, -1.0),
            vec3::<WorldSpace>(0.0, 1.0, 0.0),
            vec3::<WorldSpace>(1.0, 0.0, -0.00000004371139),
            vec3::<WorldSpace>(-0.00000004371139, 1.0, 0.0),
            vec3::<WorldSpace>(-1.0, -0.00000004371139, 0.0),
            vec3::<WorldSpace>(0.0, 0.0, 1.0),
        ];

        let mat4s = [
            transform::rotate_x::<WorldSpace>(std::f32::consts::FRAC_PI_2),
            transform::rotate_y::<WorldSpace>(std::f32::consts::FRAC_PI_2),
            transform::rotate_z::<WorldSpace>(std::f32::consts::FRAC_PI_2),
        ];

        for i in 0..mat4s.len() {
            for j in 0..v.len() {
                assert_eq!(mat4s[i] * v[j].extend(0.0), expected[i * 3 + j].extend(0.0));
                assert_eq!(mat4s[i] * v[j].extend(1.0), expected[i * 3 + j].extend(1.0));
            }
        }
    }
}
