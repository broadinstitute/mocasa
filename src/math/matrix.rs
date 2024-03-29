use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub(crate) struct Matrix {
    pub(crate) n_cols: usize,
    pub(crate) elements: Vec<f64>,
}

impl Matrix {
    pub(crate) fn fill<F: FnMut(usize, usize) -> f64>(n_rows: usize, n_cols: usize, mut f: F)
                                                      -> Matrix {
        let n = n_rows * n_cols;
        let elements = {
            let mut elements: Vec<f64> = Vec::with_capacity(n);
            for i_row in 0..n_rows {
                for i_col in 0..n_cols {
                    elements.push(f(i_row, i_col))
                }
            }
            elements
        };
        Matrix { n_cols, elements }
    }
}

impl Index<usize> for Matrix {
    type Output = [f64];

    fn index(&self, i_row: usize) -> &Self::Output {
        let from = i_row * self.n_cols;
        let to = from + self.n_cols;
        &self.elements[from..to]
    }
}

impl IndexMut<usize> for Matrix {
    fn index_mut(&mut self, i_row: usize) -> &mut Self::Output {
        let from = i_row * self.n_cols;
        let to = from + self.n_cols;
        &mut self.elements[from..to]
    }
}

