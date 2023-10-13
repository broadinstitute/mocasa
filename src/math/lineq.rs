use crate::error::Error;
use crate::math::matrix::Matrix;

pub(crate) fn solve_lin_eq(mut coeffs: Matrix, mut sums: Vec<f64>) -> Result<Vec<f64>, Error> {
    let n = check_and_get_size(&coeffs, &sums)?;
    let mut row_is: Vec<usize> = (0..n).collect();
    let mut col_is: Vec<usize> = (0..n).collect();
    let mut isolated_is: Vec<usize> = vec![0; n];
    while !row_is.is_empty() {
        let (i_row, i_col) = get_pivot(&coeffs, &row_is, &col_is)?;
        row_is.retain(|i| *i != i_row);
        col_is.retain(|i| *i != i_col);
        eliminate(&mut coeffs, &mut sums, i_row, i_col, n, &col_is);
        isolated_is[i_col] = i_row;
    }
    get_solutions(&coeffs, &sums, &isolated_is)
}

fn check_and_get_size(coeffs: &Matrix, sums: &Vec<f64>) -> Result<usize, Error> {
    if coeffs.n_rows != coeffs.n_cols {
        Err(Error::from(format!(
            "Coefficient matrix has {} rows and {} columns, and needs to be square.",
            coeffs.n_rows, coeffs.n_cols)))
    } else if coeffs.n_rows != sums.len() {
        Err(Error::from(format!(
            "Coefficient matrix has size {} while sums vector as size {}, but these need to be \
            the same.",
            coeffs.n_rows, sums.len())))
    } else {
        Ok(sums.len())
    }
}

fn get_pivot(coeffs: &Matrix, row_is: &[usize], col_is: &[usize])
             -> Result<(usize, usize), Error> {
    let mut i_row_best = row_is[0];
    let mut i_col_best = col_is[0];
    let mut value_abs_best = coeffs[i_row_best][i_col_best].abs();
    for i_row in row_is {
        for i_col in col_is {
            let value_abs = coeffs[*i_row][*i_col].abs();
            if value_abs > value_abs_best {
                i_row_best = *i_row;
                i_col_best = *i_col;
                value_abs_best = value_abs;
            }
        }
    }
    if value_abs_best == 0.0 {
        Err(Error::from("Cannot solve degenerate systems of linear equations."))
    } else {
        Ok((i_row_best, i_col_best))
    }
}

fn eliminate(coeffs: &mut Matrix, sums: &mut [f64], i_row: usize, i_col: usize,
             n: usize, col_is_others: &[usize]) {
    for i_row_other in (0..n).filter(|i_row_other| *i_row_other != i_row) {
        let factor = coeffs[i_row_other][i_col] / coeffs[i_row][i_col];
        for i_col_other in col_is_others {
            coeffs[i_row_other][*i_col_other] -= factor * coeffs[i_row][*i_col_other];
        }
        sums[i_row_other] -= factor * sums[i_row];
    }
}

fn get_solutions(coeffs: &Matrix, sums: &[f64], isolated_is: &[usize])
                 -> Result<Vec<f64>, Error> {
    let solutions: Vec<f64> =
        isolated_is.iter().enumerate().map(|(i_col, i_row)| {
            sums[*i_row] / coeffs[*i_row][i_col]
        }).collect();
    if solutions.iter().any(|solution| !solution.is_finite()) {
        Err(Error::from("Cannot solve (near) degenerate systems of linear equations."))
    } else {
        Ok(solutions)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::math::lineq::solve_lin_eq;
    use crate::math::matrix::Matrix;

    #[test]
    fn try_to_solve() {
        const SIZE: usize = 12;
        let mut rng = rand::thread_rng();
        let coeffs = Matrix::fill(SIZE, SIZE, |_, _| { rng.gen() });
        let sums = (0..SIZE).map(|_| rng.gen()).collect::<Vec<f64>>();
        let xs = solve_lin_eq(coeffs.clone(), sums.clone()).unwrap();
        for i_row in 0..SIZE {
            let mut sum_actual: f64 = 0.0;
            for i_col in 0..SIZE {
                sum_actual += coeffs[i_row][i_col] * xs[i_col];
            }
            let sum = sums[i_row];
            let rel_error = 2.0 * (sum_actual - sum).abs() / (sum_actual.abs() + sum.abs());
            println!("Sum actual {}, sum expected {}, relative error: {}", sum_actual, sum,
                     rel_error);
            const TOLERANCE: f64 = 1e-5;
            assert!(rel_error < TOLERANCE);
        }
    }
}