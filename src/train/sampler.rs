use std::sync::Arc;
use rand::Rng;
use crate::data::Meta;
use crate::math::stats::Stats;
use crate::mcmc::metro::MetroHast;
use crate::train::model::TrainModel;
use crate::train::params::Params;
use crate::train::vars::{VarIndex, Vars};

pub(crate) struct Sampler<R: Rng> {
    metro_hast: MetroHast<R>,
    e_stats: Vec<Stats>,
    t_stats: Vec<Vec<Stats>>,
}

impl<R: Rng> Sampler<R> {
    pub(crate) fn new(meta: Arc<Meta>, rng: R) -> Sampler<R> {
        let n_data_points = meta.n_data_points();
        let n_traits = meta.n_traits();
        let e_stats: Vec<Stats> = (0..n_data_points).map(|_| Stats::new()).collect();
        let t_stats: Vec<Vec<Stats>> =
            (0..n_data_points).map(|_| {
                (0..n_traits).map(|_| Stats::new()).collect()
            }).collect();
        let metro_hast = MetroHast::new(rng);
        Sampler { e_stats, t_stats, metro_hast }
    }
    pub(crate) fn sample(&mut self, model: &TrainModel, params: &Params, vars: &mut Vars) {
        for i_var in vars.indices() {
            match i_var {
                VarIndex::E { i_data_point } => {
                    vars.es[i_data_point] = self.sample_e(model, params, vars, &i_data_point);
                }
                VarIndex::T { i_data_point, i_trait } => {
                    vars.ts[i_data_point][i_trait] =
                        self.sample_t(model, params, vars, &i_data_point, &i_trait);
                }
            }
        }
    }
    pub(crate) fn sample_e(&mut self, model: &TrainModel, params: &Params, vars: &Vars,
                           i_data_point: &usize) -> f64 {
        let f_quot = model.f_quot_e(params, vars, i_data_point);
        let e_old = vars.es[*i_data_point];
        let sigma_estimate =
            self.e_stats[*i_data_point].variance().unwrap_or(params.tau);
        let draw = self.metro_hast.draw(f_quot, e_old, sigma_estimate);
        draw.x
    }
    pub(crate) fn sample_t(&mut self, model: &TrainModel, params: &Params, vars: &Vars,
                           i_data_point: &usize, i_trait: &usize) -> f64 {
        let f_quot = model.f_quot_t(params, vars, i_data_point, i_trait);
        let t_old = vars.ts[*i_data_point][*i_trait];
        let sigma_estimate =
            self.t_stats[*i_data_point][*i_trait].variance()
                .unwrap_or(params.sigmas[*i_trait]);
        let draw = self.metro_hast.draw(f_quot, t_old, sigma_estimate);
        draw.x
    }
}