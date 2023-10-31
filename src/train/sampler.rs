use std::sync::Arc;
use rand::Rng;
use crate::data::Meta;
use crate::mcmc::metro::MetroHast;
use crate::train::model::TrainModel;
use crate::train::params;
use crate::train::params::Params;
use crate::train::var_trace::VarTracer;
use crate::train::vars::{VarIndex, Vars};

pub(crate) struct Sampler<R: Rng> {
    meta: Arc<Meta>,
    metro_hast: MetroHast<R>,
    e_stats: Vec<VarTracer>,
    t_stats: Vec<Vec<VarTracer>>,
}

impl<R: Rng> Sampler<R> {
    pub(crate) fn new(meta: Arc<Meta>, rng: R, params: &Params) -> Sampler<R> {
        let n_data_points = meta.n_data_points();
        let n_traits = meta.n_traits();
        let e_stats =
            e_stats_new(n_data_points, params.mu, params::TAU);
        let t_stats =
            t_stats_new(n_data_points, n_traits, &params.betas,
                        &params.sigmas);
        let metro_hast = MetroHast::new(rng);
        Sampler { meta, e_stats, t_stats, metro_hast }
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
        let e_stat = &mut self.e_stats[*i_data_point];
        let sigma_estimate = e_stat.std_dev();
        let draw = self.metro_hast.draw(f_quot, e_old, sigma_estimate);
        let e = draw.x;
        e_stat.add(draw);
        e
    }
    pub(crate) fn sample_t(&mut self, model: &TrainModel, params: &Params, vars: &Vars,
                           i_data_point: &usize, i_trait: &usize) -> f64 {
        let f_quot = model.f_quot_t(params, vars, i_data_point, i_trait);
        let t_old = vars.ts[*i_data_point][*i_trait];
        let t_stat = &mut self.t_stats[*i_data_point][*i_trait];
        let sigma_estimate = t_stat.std_dev();
        let draw = self.metro_hast.draw(f_quot, t_old, sigma_estimate);
        let t = draw.x;
        t_stat.add(draw);
        t
    }
    pub(crate) fn squash_stats(&mut self) {
        self.e_stats.iter_mut().for_each(|e_stat| e_stat.soften_stats());
        self.t_stats.iter_mut().for_each(|t_stats|
            t_stats.iter_mut().for_each(|t_stat| t_stat.soften_stats())
        )
    }
}

fn e_stats_new(n_data_points: usize, mean_estimate: f64, std_dev_estimate: f64) -> Vec<VarTracer> {
    (0..n_data_points).map(|_| VarTracer::new(mean_estimate, std_dev_estimate)).collect()
}

fn t_stats_new(n_data_points: usize, n_traits: usize, mean_estimates: &[f64],
               std_dev_estimates: &[f64])
               -> Vec<Vec<VarTracer>> {
    (0..n_data_points).map(|_| {
        (0..n_traits).map(|i_trait|
            {
                let mean_estimate = mean_estimates[i_trait];
                let std_dev_estimate = std_dev_estimates[i_trait];
                VarTracer::new(mean_estimate, std_dev_estimate)
            }
        ).collect()
    }).collect()
}