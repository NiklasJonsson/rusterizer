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

pub fn mat4<CSF, CST>(
    x00: f32,
    x01: f32,
    x02: f32,
    x03: f32,
    x10: f32,
    x11: f32,
    x12: f32,
    x13: f32,
    x20: f32,
    x21: f32,
    x22: f32,
    x23: f32,
    x30: f32,
    x31: f32,
    x32: f32,
    x33: f32,
) -> Mat4<CSF, CST>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    let array = [
        [x00, x01, x02, x03],
        [x10, x11, x12, x13],
        [x20, x21, x22, x23],
        [x30, x31, x32, x33],
    ];

    Mat4::<CSF, CST> {
        array,
        _from_coordinate_space: PhantomData,
        _to_coordinate_space: PhantomData,
    }
}

impl<CSF, CSM, CST, const N: usize> Mul<Matrix<CSF, CSM, { N }>> for Matrix<CSM, CST, { N }>
where
    CSF: CoordinateSystem,
    CSM: CoordinateSystem,
    CST: CoordinateSystem,
{
    type Output = Matrix<CSF, CST, { N }>;
    fn mul(self, other: Matrix<CSF, CSM, { N }>) -> Self::Output {
        let mut result = self.array.clone();
        for i in 0..N {
            for j in 0..N {
                let r: Vector<CSF, { N }> = self.row(i).into();
                let c: Vector<CSF, { N }> = other.col(j).into();
                result[i][j] = r.dot(c).into();
            }
        }

        Self::Output {
            array: result,
            _from_coordinate_space: PhantomData,
            _to_coordinate_space: PhantomData,
        }
    }
}

impl<CSF, CST, const N: usize> Matrix<CSF, CST, { N }>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    pub fn row(&self, i: usize) -> [f32; N] {
        self.array[i]
    }

    pub fn col(&self, j: usize) -> [f32; N] {
        let mut result = self.array[0].clone();

        for i in 0..N {
            result[i] = self.array[i][j];
        }

        result
    }
}

impl<CSF, CST> Mat4<CSF, CST>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    pub fn identity() -> Self {
        let mut array = [[0.0f32; { 4 }]; { 4 }];

        for i in 0..4 {
            array[i][i] = 1.0;
        }

        Self {
            array,
            _to_coordinate_space: PhantomData,
            _from_coordinate_space: PhantomData,
        }
    }
}

impl<CSF, CST, const N: usize> std::fmt::Debug for Matrix<CSF, CST, { N }>
where
    CSF: PrintableType + CoordinateSystem,
    CST: PrintableType + CoordinateSystem,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .array
            .iter()
            .map(|row| {
                format!(
                    "[{}]",
                    row.iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n  ");
        write!(
            f,
            "Matrix<{}, {}, {}>:\n[\n  {}\n]",
            CSF::NAME,
            CST::NAME,
            N,
            s
        )
    }
}
