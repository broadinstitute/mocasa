use rand::Rng;
use crate::data::Metaphor;
use crate::train::gibbs::GibbsSampler;
use crate::train::model::TrainModel;
use crate::train::params::Params;
use crate::train::var_stats::VarStats;
use crate::train::vars::{VarIndex, Vars};

pub(crate) struct Sampler<R: Rng> {
    gibbs: GibbsSampler<R>,
    var_stats: VarStats,
}

impl<R: Rng> Sampler<R> {
    pub(crate) fn new(metaphor: &Metaphor, rng: R) -> Sampler<R> {
        let gibbs = GibbsSampler::new(rng);
        let var_stats = VarStats::new(metaphor.clone());
        Sampler { gibbs, var_stats }
    }
    pub(crate) fn sample_n(&mut self, model: &TrainModel, params: &Params, vars: &mut Vars,
                           n_steps: usize) {
        for _ in 0..n_steps {
            self.sample_one(model, params, vars)
        }
    }
    pub(crate) fn sample_one(&mut self, model: &TrainModel, params: &Params, vars: &mut Vars) {
        for i_var in vars.indices() {
            match i_var {
                VarIndex::E { i_data_point } => {
                    vars.es[i_data_point] = self.gibbs.draw_e(vars, params, i_data_point);
                }
                VarIndex::T { i_data_point, i_trait } => {
                    vars.ts[i_data_point][i_trait] =
                        self.gibbs.draw_t(&model.data, vars, params, i_data_point, i_trait);
                }
            }
        }
        self.var_stats.add(vars);
    }
    pub(crate) fn var_stats(&self) -> &VarStats { &self.var_stats }
}
