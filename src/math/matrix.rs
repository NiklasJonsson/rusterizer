use core::ops::Mul;

use core::marker::PhantomData;

use crate::math::*;

#[derive(Copy, Clone)]
pub struct Matrix<CSF, CST, const N: usize>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    array: [[f32; N]; N],
    _from_coordinate_space: PhantomData<CSF>,
    _to_coordinate_space: PhantomData<CST>,
}

pub type Mat4<CSF, CST = CSF> = Matrix<CSF, CST, 4>;

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
        let mut result = self.array;
        for row in result.iter_mut() {
            let r: Vector<CSF, { N }> = (*row).into();
            for (j, v) in row.iter_mut().enumerate() {
                let c: Vector<CSF, { N }> = other.col(j).into();
                *v = r.dot(c);
            }
        }

        Self::Output {
            array: result,
            _from_coordinate_space: PhantomData,
            _to_coordinate_space: PhantomData,
        }
    }
}

impl<CSF, CST, const N: usize> std::cmp::PartialEq for Matrix<CSF, CST, { N }>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    fn eq(&self, other: &Self) -> bool {
        self.array
            .iter()
            .zip(other.array.iter())
            .all(|(a, b)| a.iter().zip(b.iter()).all(|(x, y)| x == y))
    }
}

impl<CSF, CST, const N: usize> std::cmp::Eq for Matrix<CSF, CST, { N }>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
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
        let mut result = self.array[0];

        for (i, v) in result.iter_mut().enumerate() {
            *v = self.array[i][j];
        }

        result
    }

    pub fn transpose(&self) -> Self {
        let mut tmp = self.array;

        for (i, row) in tmp.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() {
                *v = self.array[j][i];
            }
        }

        Matrix::<CSF, CST, { N }> {
            array: tmp,
            _from_coordinate_space: PhantomData,
            _to_coordinate_space: PhantomData,
        }
    }
}

impl<CSF, CST> Mat4<CSF, CST>
where
    CSF: CoordinateSystem,
    CST: CoordinateSystem,
{
    pub fn identity() -> Self {
        let mut array = [[0.0f32; 4]; 4];

        for (i, row) in array.iter_mut().enumerate() {
            row[i] = 1.0;
        }

        Self {
            array,
            _to_coordinate_space: PhantomData,
            _from_coordinate_space: PhantomData,
        }
    }

    pub fn from_raw(inp: &[[f32; 4]; 4]) -> Self {
        Self {
            array: *inp,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eq_operator() {
        let mat = mat4::<WorldSpace, WorldSpace>(
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        );
        let mat0 = Mat4::<WorldSpace>::identity();
        let mat1 = Mat4::<WorldSpace>::identity();

        assert_eq!(mat0, mat1);
        assert_eq!(mat1, mat0);
        assert_eq!(mat, mat);
        assert_ne!(mat, mat0);
        assert_ne!(mat0, mat);
        assert_ne!(mat, mat1);
        assert_ne!(mat1, mat);
    }

    #[test]
    fn mul_identity() {
        let mat0 = Mat4::<WorldSpace>::identity();
        let mat1 = Mat4::<WorldSpace>::identity();
        let r = mat0 * mat1;
        assert_eq!(r, mat0);
        assert_eq!(r, mat1);
    }

    #[test]
    fn rows() {
        let mat = mat4::<WorldSpace, WorldSpace>(
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        );
        assert_eq!(mat.row(0), [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(mat.row(1), [5.0, 6.0, 7.0, 8.0]);
        assert_eq!(mat.row(2), [9.0, 10.0, 11.0, 12.0]);
        assert_eq!(mat.row(3), [13.0, 14.0, 15.0, 16.0]);
    }

    #[test]
    fn columns() {
        let mat = mat4::<WorldSpace, WorldSpace>(
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        );
        assert_eq!(mat.col(0), [1.0, 5.0, 9.0, 13.0]);
        assert_eq!(mat.col(1), [2.0, 6.0, 10.0, 14.0]);
        assert_eq!(mat.col(2), [3.0, 7.0, 11.0, 15.0]);
        assert_eq!(mat.col(3), [4.0, 8.0, 12.0, 16.0]);
    }

    #[test]
    fn transpose() {
        let mat = mat4::<WorldSpace, WorldSpace>(
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        );

        let mat_transpose = mat4::<WorldSpace, WorldSpace>(
            1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0, 4.0, 8.0, 12.0, 16.0,
        );

        assert_eq!(mat.transpose(), mat_transpose);
        assert_eq!(mat, mat_transpose.transpose());
    }

    #[test]
    fn mul() {
        let mat = mat4::<WorldSpace, WorldSpace>(
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        );

        let a = [
            [90., 100., 110., 120.],
            [202., 228., 254., 280.],
            [314., 356., 398., 440.],
            [426., 484., 542., 600.],
        ];

        let result = Mat4::<WorldSpace>::from_raw(&a);
        assert_eq!(mat * mat, result);

        let a = [
            [30., 70., 110., 150.],
            [70., 174., 278., 382.],
            [110., 278., 446., 614.],
            [150., 382., 614., 846.],
        ];
        let result = Mat4::<WorldSpace>::from_raw(&a);
        assert_eq!(mat * mat.transpose(), result);
    }
}
