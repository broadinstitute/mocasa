use crate::params::Params;

pub(crate) fn calculate_mu(params: &Params, betas: &[f64], ses: &[f64]) -> f64 {
    let tau2 = params.tau.powi(2);
    let numerator: f64 =
        params.betas.iter().zip(params.sigmas.iter()).zip(betas.iter())
            .zip(ses.iter())
            .map(|(((&beta, &sigma), &o), &se)| {
                beta * o / (sigma.powi(2) + se.powi(2))
            }).sum::<f64>() + params.mu / tau2;
    let denominator: f64 =
        params.betas.iter().zip(params.sigmas.iter()).zip(ses.iter())
            .map(|((&beta, &sigma), &se)| {
                beta.powi(2) / (sigma.powi(2) + se.powi(2))
            }).sum::<f64>() + 1.0 / tau2;
    numerator / denominator
}