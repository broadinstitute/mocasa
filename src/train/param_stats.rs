use crate::train::params::Params;

pub(crate) struct ParamHessianStats {
    n_samples: usize
}

pub(crate) struct ParamHessian {}

pub(crate) struct ParamEstimate {
    pub(crate) params: Params,
    pub(crate) is_done: bool
}

impl ParamHessianStats {
    pub(crate) fn new() -> ParamHessianStats {
        let n_samples: usize = 0;
        ParamHessianStats { n_samples }
    }
    pub(crate) fn add_hessian(&mut self, param_hessian: ParamHessian) { todo!() }
    pub(crate) fn ready_for_param_estimate(&self) -> bool { todo!() }
    pub(crate) fn estimate_params(&self) -> ParamEstimate { todo!() }
}