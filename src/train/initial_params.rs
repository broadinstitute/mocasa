use crate::error::Error;
use crate::math::stats::Stats;
use crate::train::model::TrainModel;
use crate::train::params;
use crate::train::params::Params;

pub(crate) fn estimate_initial_params(model: &TrainModel) -> Result<Params, Error> {
    let meta = model.data.meta.clone();
    let n_data_points = meta.n_data_points();
    let n_traits = meta.n_traits();
    let mut data_stats: Vec<Stats> = (0..n_traits).map(|_| Stats::new()).collect();
    for i_data_point in 0..n_data_points {
        for (i_trait, data_stat) in data_stats.iter_mut().enumerate() {
            println!("{}\t{}\t{}", i_data_point, i_trait, model.data.betas[i_data_point][i_trait]);
            data_stat.add(model.data.betas[i_data_point][i_trait])
        }
    }
    let sigmas: Vec<f64> =
        data_stats.iter().map(|stats|
            stats.variance().map(|var| var.sqrt())
                .ok_or_else(|| { Error::from("Need at least two data points.") })
        ).collect::<Result<Vec<f64>, Error>>()?;
    let means: Vec<f64> =
        data_stats.iter().map(|stats|
            stats.mean().ok_or_else(|| { Error::from("Need at least one data point.") })
        ).collect::<Result<Vec<f64>, Error>>()?;
    let mut e_stats = Stats::new();
    means.iter().for_each(|mean| println!("{}", mean));
    means.iter().for_each(|mean| e_stats.add(*mean));
    let mu =
        e_stats.mean().ok_or_else(|| { Error::from("Need at least one trait.") })?;
    let betas: Vec<f64> =
        means.iter().map(|mean| mean / (mu + params::TAU * mu.signum())).collect();
    Ok(Params { meta, mu, betas, sigmas })
}