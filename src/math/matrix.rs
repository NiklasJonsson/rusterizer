use core::ops::Mul;

use core::marker::PhantomData;

use crate::math::*;

#[derive(Copy, Clone)]
pub struct Matrix<CSF, CST, const N: usize>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    array: [[f32; { N }]; { N }],
    _from_coordinate_space: PhantomData<CSF>,
    _to_coordinate_space: PhantomData<CST>,
}

pub type Mat4<CSF, CST> = Matrix<CSF, CST, 4>;
pub type Mat3<CSF, CST> = Matrix<CSF, CST, 3>;

impl<CSF, CSM, CST, const N: usize> Mul<Matrix<CSF, CSM, { N }>> for Matrix<CSM, CST, { N }>
where
    CSF: CoordinateSystem,
    CSM: CoordinateSystem,
    CST: CoordinateSystem,
{
    type Output = Matrix<CSF, CST, { N }>;
    fn mul(self, other: Matrix<CSF, CSM, { N }>) -> Matrix<CSF, CST, { N }> {
        unimplemented!();
    }
}

impl<CSF, CST, const N: usize> Matrix<CSF, CST, { N }>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    pub fn row(&self, i: usize) -> [f32; { N }] {
        self.array[i]
    }
}
