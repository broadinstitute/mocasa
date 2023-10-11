use std::ops::Index;
use std::sync::Arc;
use crate::data::Meta;
use crate::error::Error;
use crate::math::lineq::solve_lin_eq;
use crate::math::matrix::Matrix;
use crate::math::stats::Stats;
use crate::train::param_eval::ParamEval;
use crate::train::params::{ParamIndex, Params};
use crate::util::sym_matrix::SymMatrix;

pub(crate) struct ParamHessianStats {
    meta: Arc<Meta>,
    gradient: Vec<Stats>,
    hessian: SymMatrix<Stats>,
}

impl ParamHessianStats {
    pub(crate) fn new(meta: Arc<Meta>) -> ParamHessianStats {
        let n_traits = meta.n_traits();
        let n_params = ParamIndex::n_params(n_traits);
        let gradient: Vec<Stats> = vec![Stats::new(); n_params];
        let hessian: SymMatrix<Stats> = SymMatrix::new(Stats::new(), n_params);
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
    pub(crate) fn estimate_params(&self) -> Result<Params, Error> {
        let n_traits = self.meta.n_traits();
        let n_params = ParamIndex::n_params(n_traits);
        let coeffs =
            Matrix::try_fill(n_params, n_params,
                             |i_row, i_col| {
                                 self.hessian.index((i_row, i_col)).mean()
                                     .ok_or_else(|| Error::from("No sufficient stats"))
                             })?;
        let sums: Vec<f64> =
            self.gradient.iter().map(
                |stats|
                    stats.mean().ok_or_else(|| Error::from("No sufficient stats"))
            ).collect::<Result<Vec<f64>, Error>>()?;
        let solutions = solve_lin_eq(coeffs, sums)?;
        Params::from_vec(&solutions, &self.meta)
    }
}