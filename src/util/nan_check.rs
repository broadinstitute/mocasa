use log::{Level, log_enabled, trace};
use crate::math::matrix::Matrix;

pub(crate) fn trace_nans_vec(name: &str, nums: &[f64]) {
    if log_enabled!(Level::Trace) {
        for (i, num) in nums.iter().enumerate() {
            if num.is_nan() {
                trace!("{}[{}] is NaN.", name, i)
            } else if num.is_infinite() {
                if num.is_sign_positive() {
                    trace!("{}[{}] is Inf.", name, i)
                } else {
                    trace!("{}[{}] is -Inf.", name, i)
                }
            }
        }
    }
}

pub(crate) fn trace_nans_matrix(name: &str, nums: &Matrix) {
    if log_enabled!(Level::Trace) {
        for i_row in 0..nums.n_rows {
            let row = &nums[i_row];
            for (i_col, num) in row.iter().enumerate() {
                if num.is_nan() {
                    trace!("{}[{}][{}] is NaN.", name, i_row, i_col)
                } else if num.is_infinite() {
                    if num.is_sign_positive() {
                        trace!("{}[{}][{}] is Inf.", name, i_row, i_col)
                    } else {
                        trace!("{}[{}][{}] is -Inf.", name, i_row, i_col)
                    }
                }
            }
        }
    }
}