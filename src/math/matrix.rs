use core::ops::Mul;

use core::marker::PhantomData;

use crate::math::*;

#[derive(Copy, Clone)]
pub struct Matrix<CSF, CST, const N: usize>
    where CSF: CoordinateSystem,
          CST: CoordinateSystem,
{
    array: [f32; N],
    _from_coordinate_space: PhantomData<CSF>,
    _to_coordinate_space: PhantomData<CST>,
}

impl<CSF, CSM, CST, const N: usize> Mul<Matrix<CSF, CSM, {N}>> for Matrix<CSM, CST, {N}>
    where CSF: CoordinateSystem,
          CSM: CoordinateSystem,
          CST: CoordinateSystem,
{
    type Output = Matrix<CSF, CST, {N}>;
    fn mul(self, other: Matrix<CSF, CSM, {N}>) -> Matrix<CSF, CST, {N}> {
        unimplemented!();
    }
}
pub type Mat4<CSF, CST> = Matrix<CSF, CST, 16>;
pub type Mat3<CSF, CST> = Matrix<CSF, CST, 9>;
