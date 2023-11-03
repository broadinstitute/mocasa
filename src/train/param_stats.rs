use std::ops::Index;
use std::sync::Arc;
use crate::data::Meta;
use crate::error::Error;
use crate::math::lineq::solve_lin_eq;
use crate::math::matrix::Matrix;
use crate::math::trident::TridentStats;
use crate::train::param_eval::ParamEval;
use crate::train::params::{ParamIndex, Params};
use crate::util::sym_matrix::SymMatrix;

pub(crate) struct ParamHessianStats {
    meta: Arc<Meta>,
    gradient: Vec<TridentStats>,
    hessian: SymMatrix<TridentStats>,
}

impl ParamHessianStats {
    pub(crate) fn new(meta: Arc<Meta>) -> ParamHessianStats {
        let n_traits = meta.n_traits();
        let n_params = ParamIndex::n_params(n_traits);
        let gradient: Vec<TridentStats> = vec![TridentStats::new(0.0, 0.0); n_params];
        let hessian: SymMatrix<TridentStats> =
            SymMatrix::new(TridentStats::new(0.0, 0.0), n_params);
        ParamHessianStats { meta, gradient, hessian }
    }
    pub(crate) fn survey_param_eval(&mut self, param_eval: &ParamEval) {
        let n_traits = param_eval.n_traits;
        for index1 in ParamIndex::all(n_traits) {
            let i1 = index1.get_ordinal(n_traits);
            self.gradient[i1].add(param_eval.gradient(index1));
            for index2 in ParamIndex::to(index1, n_traits) {
                let i2 = index2.get_ordinal(n_traits);
                self.hessian[(i1, i2)].add(param_eval.hessian(index1, index2))
            }
        }
    }
    pub(crate) fn estimate_params(&self, params: &Params) -> Result<Params, Error> {
        let n_traits = self.meta.n_traits();
        let n_params = ParamIndex::n_params(n_traits);
        let coeffs =
            Matrix::fill(n_params, n_params,
                         |i_row, i_col| {
                             self.hessian.index((i_row, i_col)).mean()
                         });
        let sums: Vec<f64> =
            self.gradient.iter().map(|stats| stats.mean()).collect::<Vec<f64>>();
        let param_changes = solve_lin_eq(coeffs, sums)?;
        let param_values_new =
            ParamIndex::all(n_traits)
                .map(|index| {
                    params[index] - param_changes[index.get_ordinal(n_traits)]
                })
                .collect::<Vec<f64>>();
        Params::from_vec(&param_values_new, &self.meta)
    }
}