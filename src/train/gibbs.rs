use rand::Rng;
use rand_distr::Normal;
use crate::train::params::Params;
use crate::train::vars::Vars;
use rand_distr::Distribution;

pub(crate) fn draw_e<R: Rng>(rng: &mut R, vars: &Vars, params: &Params, i_data_point: usize) -> f64 {
    let n_traits = params.meta.n_traits();
    let tau = params.tau;
    let inv_sig_sum: f64 = 1.0 / tau.powi(2) + (0..n_traits).map(|i_trait| {
        (params.betas[i_trait] / params.sigmas[i_trait]).powi(2)
    }).sum::<f64>();
    let variance = 1.0 / inv_sig_sum;
    let std_dev = variance.sqrt();
    let frac_sum = params.mu / tau.powi(2) + (0..n_traits).map(|i_trait| {
        params.betas[i_trait]*vars.ts[i_data_point][i_trait] / params.sigmas[i_trait].powi(2)
    }).sum::<f64>();
    let mean = variance * frac_sum;
    Normal::new(mean, std_dev).unwrap().sample(rng)
}

pub(crate) fn make_t_gibbs(vars: &Vars, params: &Params, i_trait: usize) -> Normal<f64> {
    todo!()
}