use crate::train::params::Params;

pub(crate) struct ParamDiffStats {
    n_samples: usize
}

pub(crate) struct ParamDiffs {}

pub(crate) struct ParamEstimate {
    pub(crate) params: Params,
    pub(crate) is_done: bool
}

impl ParamDiffStats {
    pub(crate) fn new() -> ParamDiffStats {
        let n_samples: usize = 0;
        ParamDiffStats { n_samples }
    }
    pub(crate) fn add_diffs(&mut self, param_diffs: ParamDiffs) { todo!() }
    pub(crate) fn ready_for_param_estimate(&self) -> bool { todo!() }
    pub(crate) fn estimate_params(&self) -> ParamEstimate { todo!() }
}