use crate::math::stats::Stats;
use crate::train::param_eval::ParamEval;
use crate::train::params::{ParamIndex, Params};
use crate::util::sym_matrix::SymMatrix;

pub(crate) struct ParamHessianStats {
    gradient: Vec<Stats>,
    hessian: SymMatrix<Stats>
}

impl ParamHessianStats {
    pub(crate) fn new(n_traits: usize) -> ParamHessianStats {
        let n_params = ParamIndex::n_params(n_traits);
        let gradient: Vec<Stats> = vec![Stats::new(); n_params];
        let hessian: SymMatrix<Stats> = SymMatrix::new(Stats::new(), n_params);
        ParamHessianStats { gradient, hessian }
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
    pub(crate) fn ready_for_param_estimate(&self) -> bool { todo!() }
    pub(crate) fn estimate_params(&self) -> Params { todo!() }
}