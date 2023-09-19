use crate::train::model::{Params, TrainModel, Vars};
use crate::train::param_stats::ParamSample;

pub(crate) struct Sampler {}

impl Sampler {
    pub(crate) fn new() -> Sampler { Sampler {}}
    pub(crate) fn sample(&self, model: &TrainModel, params: &Params, vars: &mut Vars)
        -> ParamSample {
        todo!()
    }
}