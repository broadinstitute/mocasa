use crate::train::model::TrainModel;
use crate::train::params::{ParamIndex, Params};
use crate::train::vars::Vars;

pub(crate) struct ParamEval {

}

impl ParamEval {
    pub(crate) fn new(model: &TrainModel, params: &Params, vars: &Vars) -> ParamEval {
        ParamEval {}
    }
    pub(crate) fn gradient(&self, index: ParamIndex) -> f64 {
        todo!()
    }
    pub(crate) fn hessian(&self, index1: ParamIndex, index2: ParamIndex) -> f64 {
        todo!()
    }
}