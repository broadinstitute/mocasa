use std::sync::Arc;
use crate::data::Meta;
use crate::math::matrix::Matrix;
use crate::train::vars::Vars;

pub(crate) struct VarStats {
    n: usize,
    e_sums: Vec<f64>,
    e2_sums: Vec<f64>,
    e_t_sums: Matrix,
    t2_sums: Matrix,
}

impl VarStats {
    pub(crate) fn new(meta: &Arc<Meta>) -> VarStats {
        let n: usize = 0;
        let n_data_points = meta.n_data_points();
        let n_traits = meta.n_traits();
        let e_sums: Vec<f64> = vec![0.0; n_data_points];
        let e2_sums: Vec<f64> = vec![0.0; n_data_points];
        let e_t_sums: Matrix = Matrix::fill(n_data_points, n_traits, |_, _| 0.0);
        let t2_sums: Matrix = Matrix::fill(n_data_points, n_traits, |_, _| 0.0);
        VarStats { n, e_sums, e2_sums, e_t_sums, t2_sums }
    }
    pub(crate) fn add(&mut self, vars: &Vars) {
        self.n += 1;
        let n_data_points = vars.meta.n_data_points();
        let n_traits = vars.meta.n_traits();
        for j in 0..n_data_points {
            let e_j = vars.es[j];
            self.e_sums[j] += e_j;
            self.e2_sums[j] += e_j.powi(2);
            for i in 0..n_traits {
                let t_j_i = vars.ts[j][i];
                self.e_t_sums[j][i] += e_j * t_j_i;
                self.t2_sums[j][i] += t_j_i.powi(2);
            }
        }
    }
}