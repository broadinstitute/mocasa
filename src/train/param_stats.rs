use crate::train::model::Params;

pub(crate) struct ParamStats {
    n_samples: usize
}

pub(crate) struct ParamSample {}

pub(crate) struct ParamEstimate {
    pub(crate) params: Params,
    pub(crate) is_done: bool
}

impl ParamStats {
    pub(crate) fn new() -> ParamStats {
        let n_samples: usize = 0;
        ParamStats { n_samples }
    }
    pub(crate) fn add_sample(&mut self, sample: ParamSample) { todo!() }
    pub(crate) fn ready_for_param_estimate(&self) -> bool { todo!() }
    pub(crate) fn estimate_params(&self) -> ParamEstimate { todo!() }
}