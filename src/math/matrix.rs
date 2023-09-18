pub(crate) struct Matrix {
    pub(crate) n_rows: usize,
    pub(crate) n_cols: usize,
    pub(crate) elements: Vec<f64>,
}

impl Matrix {
    pub(crate) fn new<F: Fn(usize, usize) -> f64>(n_rows: usize, n_cols: usize, f: F)
        -> Matrix {
        let mut elements: Vec<f64> = Vec::new();
        for i_row in 0..n_rows {
            for i_col in 0..n_cols {
                elements.push(f(i_row, i_col))
            }
        }
        Matrix { n_rows, n_cols, elements }
    }
}

