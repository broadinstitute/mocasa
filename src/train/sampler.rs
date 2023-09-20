use std::sync::Arc;
use crate::data::Meta;
use crate::math::stats::Stats;
use crate::train::model::TrainModel;
use crate::train::params::Params;
use crate::train::vars::{VarIndex, Vars};

pub(crate) struct Sampler {
    e_stats: Vec<Stats>,
    t_stats: Vec<Vec<Stats>>,
}

impl Sampler {
    pub(crate) fn new(meta: Arc<Meta>) -> Sampler {
        let n_data_points = meta.n_data_points;
        let n_traits = meta.n_traits();
        let e_stats: Vec<Stats> = (0..n_data_points).map(|_| Stats::new()).collect();
        let t_stats: Vec<Vec<Stats>> =
            (0..n_data_points).map(|_| {
                (0..n_traits).map(|_| Stats::new()).collect()
            }).collect();
        Sampler { e_stats, t_stats }
    }
    pub(crate) fn sample(&mut self, model: &TrainModel, params: &Params, vars: &mut Vars) {
        for i_var in vars.indices() {
            match i_var {
                VarIndex::E { i_data_point } => {
                    vars.es[i_data_point] = self.sample_e(model, params, vars, i_data_point);
                }
                VarIndex::T { i_data_point, i_trait } => {
                    vars.ts[i_data_point][i_trait] =
                        self.sample_t(model, params, vars, i_data_point, i_trait);
                }
            }
        }
    }
    pub(crate) fn sample_e(&mut self, model: &TrainModel, params: &Params, vars: &Vars,
                           i_data_point: usize) -> f64 {

        todo!()
    }
    pub(crate) fn sample_t(&mut self, model: &TrainModel, params: &Params, vars: &Vars,
                           i_data_point: usize, i_trait: usize) -> f64 {
        todo!()
    }
}