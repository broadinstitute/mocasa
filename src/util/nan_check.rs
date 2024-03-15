use std::fmt::{Display, Formatter};
use crate::math::matrix::Matrix;

pub(crate) fn find_nans_vec(nums: &[f64]) -> Vec<usize> {
    let mut nans: Vec<usize> = Vec::new();
    for (i, num) in nums.iter().enumerate() {
        if num.is_nan() {
            nans.push(i)
        }
    }
    nans
}

pub(crate) struct Coords {
    i_row: usize,
    i_col: usize,
}

impl Display for Coords {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.i_row, self.i_col)
    }
}

pub(crate) fn find_nans_matrix(nums: &Matrix) -> Vec<Coords> {
    let mut nans: Vec<Coords> = Vec::new();
    for i_row in 0..nums.n_rows {
        let row = &nums[i_row];
        for (i_col, num) in row.iter().enumerate() {
            if num.is_nan() {
                nans.push(Coords { i_row, i_col })
            }
        }
    }
    nans
}