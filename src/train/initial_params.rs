use crate::data::GwasData;
use crate::error::Error;
use crate::math::stats::Stats;
use crate::params::Params;

pub(crate) fn estimate_initial_params(data: &GwasData) -> Result<Params, Error> {
    let meta = &data.meta;
    let n_data_points = meta.n_data_points();
    let n_traits = meta.n_traits();
    let mut beta_stats: Vec<Stats> = (0..n_traits).map(|_| Stats::new()).collect();
    let mut se_stats: Vec<Stats> = (0..n_traits).map(|_| Stats::new()).collect();
    for i_data_point in 0..n_data_points {
        for i_trait in 0..n_traits {
            beta_stats[i_trait].add(data.betas[i_data_point][i_trait]);
            se_stats[i_trait].add(data.betas[i_data_point][i_trait])
        }
    }
    let sigmas: Vec<f64> =
        beta_stats.iter().map(|stats|
            stats.variance().map(|var| var.sqrt())
                .ok_or_else(|| { Error::from("Need at least two data points.") })
        ).collect::<Result<Vec<f64>, Error>>()?;
    let beta_means: Vec<f64> =
        beta_stats.iter().map(|stats|
            stats.mean().ok_or_else(|| { Error::from("Need at least one data point.") })
        ).collect::<Result<Vec<f64>, Error>>()?;
    let se_means: Vec<f64> =
        se_stats.iter().map(|stats|
            stats.mean().ok_or_else(|| { Error::from("Need at least one data point.") })
        ).collect::<Result<Vec<f64>, Error>>()?;
    let mut means_stats = Stats::new();
    beta_means.iter().for_each(|mean| means_stats.add(*mean));
    let mu =
        means_stats.mean().ok_or_else(|| { Error::from("Need at least one trait.") })?;
    let mut precision_stats = Stats::new();
    se_means.iter().for_each(|se| precision_stats.add(se.powi(-2)));
    let tau =
        1.0 / precision_stats.variance()
            .ok_or_else(|| { Error::from("Need at least one trait.") })?
            .sqrt();
    let betas: Vec<f64> = beta_means.iter().map(|mean| mean / (mu + tau * mu.signum())).collect();
    let trait_names = meta.trait_names.clone();
    Ok(Params { trait_names, mu, tau, betas, sigmas })
}