use rand::Rng;
use crate::data::{GwasData, Meta};
use crate::sample::gibbs::GibbsSampler;
use crate::params::Params;
use crate::sample::var_stats::VarStats;
use crate::sample::vars::{VarIndex, Vars};

pub(crate) struct Sampler<R: Rng> {
    gibbs: GibbsSampler<R>,
    var_stats: VarStats,
}

pub(crate) trait Tracer {
    fn trace_e(&mut self, i_endo: usize, e: f64, i_chain: usize);
    fn trace_t(&mut self, i_trait: usize, t: f64, i_chain: usize);
}

pub(crate) struct NoOpTracer;

impl NoOpTracer {
    pub(crate) fn new() -> NoOpTracer { NoOpTracer }
}

impl Tracer for NoOpTracer {
    fn trace_e(&mut self, _i_endo: usize, _e: f64, _i_chain: usize) {}
    fn trace_t(&mut self, _i_trait: usize, _t: f64, _i_chain: usize) {}
}

impl<R: Rng> Sampler<R> {
    pub(crate) fn new(meta: &Meta, rng: R) -> Sampler<R> {
        let gibbs = GibbsSampler::new(rng);
        let var_stats = VarStats::new(meta.clone());
        Sampler { gibbs, var_stats }
    }
    pub(crate) fn sample_n(&mut self, data: &GwasData, params: &Params, vars: &mut [Vars],
                           n_steps: usize, tracer: &mut dyn Tracer) {
        for _ in 0..n_steps {
            self.sample_one(data, params, vars, tracer)
        }
    }
    pub(crate) fn sample_one(&mut self, data: &GwasData, params: &Params, vars: &mut [Vars],
                             tracer: &mut dyn Tracer) {
        for (i_chain, vars) in vars.iter_mut().enumerate() {
            for i_var in vars.indices() {
                match i_var {
                    VarIndex::E { i_data_point, i_endo } => {
                        let e = self.gibbs.draw_e(vars, params, i_data_point, i_endo);
                        tracer.trace_e(i_endo, e, i_chain);
                        vars.es[i_data_point][i_endo] = e;
                    }
                    VarIndex::T { i_data_point, i_trait } => {
                        let t = self.gibbs.draw_t(data, vars, params, i_data_point, i_trait);
                        tracer.trace_t(i_trait, t, i_chain);
                        vars.ts[i_data_point][i_trait] = t;
                    }
                }
            }
            self.var_stats.add(vars);
        }
    }
    pub(crate) fn var_stats(&self) -> &VarStats { &self.var_stats }
}
