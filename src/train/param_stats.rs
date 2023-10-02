use crate::math::stats::Stats;
use crate::train::param_eval::ParamEval;
use crate::train::params::{ParamIndex, Params};

pub(crate) struct ParamHessianStats {
    n_samples: usize,
    gradient: Vec<Stats>,
}

pub(crate) struct ParamEstimate {
    pub(crate) params: Params,
    pub(crate) is_done: bool
}

impl ParamHessianStats {
    pub(crate) fn new(n_traits: usize) -> ParamHessianStats {
        let n_samples: usize = 0;
        let n_params = ParamIndex::n_params(n_traits);
        let gradient: Vec<Stats> = vec![Stats::new(); n_params];
        ParamHessianStats { n_samples, gradient }
    }
    pub(crate) fn survey_param_eval(&mut self, param_eval: &ParamEval) {

        todo!()
    }
    pub(crate) fn ready_for_param_estimate(&self) -> bool { todo!() }
    pub(crate) fn estimate_params(&self) -> ParamEstimate { todo!() }
}