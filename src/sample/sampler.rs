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

pub(crate) trait ETracer {
    fn trace_e(&mut self, e: f64);
}

impl<R: Rng> Sampler<R> {
    pub(crate) fn new(meta: &Meta, rng: R) -> Sampler<R> {
        let gibbs = GibbsSampler::new(rng);
        let var_stats = VarStats::new(meta.clone());
        Sampler { gibbs, var_stats }
    }
    pub(crate) fn sample_n(&mut self, data: &GwasData, params: &Params, vars: &mut Vars,
                           n_steps: usize, e_tracer: &mut Option<Box<dyn ETracer>>) {
        for _ in 0..n_steps {
            self.sample_one(data, params, vars, e_tracer)
        }
    }
    pub(crate) fn sample_one(&mut self, data: &GwasData, params: &Params, vars: &mut Vars,
                             e_tracer: &mut Option<Box<dyn ETracer>>) {
        let mut n_endos_2: usize = 0;
        let mut n_traits_2: usize = 0;
        for i_var in vars.indices() {
            match i_var {
                VarIndex::E { i_data_point, i_endo } => {
                    let e = self.gibbs.draw_e(vars, params, i_data_point, i_endo);
                    if let Some(e_tracer) = e_tracer {
                        e_tracer.trace_e(e);
                    }
                    vars.es[i_data_point][i_endo] = e;
                    n_endos_2 += 1;
                }
                VarIndex::T { i_data_point, i_trait } => {
                    vars.ts[i_data_point][i_trait] =
                        self.gibbs.draw_t(data, vars, params, i_data_point, i_trait);
                    n_traits_2 += 1;
                }
            }
        }
        assert_eq!(n_endos_2, data.meta.n_endos());
        assert_eq!(n_traits_2, data.meta.n_traits());
        self.var_stats.add(vars);
    }
    pub(crate) fn var_stats(&self) -> &VarStats { &self.var_stats }
}
