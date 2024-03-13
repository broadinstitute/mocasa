use crate::error::Error;
use crate::params::Params;

pub(crate) fn calculate_mu(params: &Params, betas: &[f64], ses: &[f64]) -> Result<f64, Error> {
    if params.mus.len() == 1 {
        let tau2 = params.taus[0].powi(2);
        let numerator: f64 =
            params.betas[0].iter().zip(params.sigmas.iter()).zip(betas.iter())
                .zip(ses.iter())
                .map(|(((&beta, &sigma), &o), &se)| {
                    beta * o / (sigma.powi(2) + se.powi(2))
                }).sum::<f64>() + params.mus[0] / tau2;
        let denominator: f64 =
            params.betas[0].iter().zip(params.sigmas.iter()).zip(ses.iter())
                .map(|((&beta, &sigma), &se)| {
                    beta.powi(2) / (sigma.powi(2) + se.powi(2))
                }).sum::<f64>() + 1.0 / tau2;
        Ok(numerator / denominator)
    } else {
        Err(Error::from(
            "Exact calculations are currently only supported for a single endophenotype"
        ))
    }
}