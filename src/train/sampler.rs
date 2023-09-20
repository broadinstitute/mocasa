use crate::train::model::TrainModel;
use crate::train::params::Params;
use crate::train::vars::{VarIndex, Vars};

pub(crate) struct Sampler {}

impl Sampler {
    pub(crate) fn new() -> Sampler { Sampler {} }
    pub(crate) fn sample(&self, model: &TrainModel, params: &Params, vars: &mut Vars) {
        for i_var in vars.indices() {
            match i_var {
                VarIndex::Es { i_data_point } => {
                    vars.es[i_data_point] = self.sample_e(model, params, vars, i_data_point);
                }
                VarIndex::Ts { i_data_point, i_trait } => {
                    vars.ts[i_data_point][i_trait] =
                        self.sample_t(model, params, vars, i_data_point, i_trait);
                }
            }
        }
    }
    pub(crate) fn sample_e(&self, model: &TrainModel, params: &Params, vars: &Vars,
                           i_data_point: usize) -> f64 {

        todo!()
    }
    pub(crate) fn sample_t(&self, model: &TrainModel, params: &Params, vars: &Vars,
                           i_data_point: usize, i_trait: usize) -> f64 {
        todo!()
    }
}