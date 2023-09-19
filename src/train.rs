mod model;
mod param_stats;
mod sampler;

use crate::data::{load_training_data, TrainData};
use crate::error::Error;
use crate::options::config::Config;
use crate::train::model::{Params, TrainModel};
use crate::train::param_stats::ParamStats;
use crate::train::sampler::Sampler;

pub(crate) fn train_or_check(config: &Config, dry: bool) -> Result<(), Error> {
    let data = load_training_data(config)?;
    println!("Loaded data for {} variants", data.beta_se_lists.len());
    println!("{}", data);
    if !dry {
        train(data)?;
    }
    Ok(())
}

fn train(data: TrainData) -> Result<Params, Error> {
    let model = TrainModel::new(data);
    let mut params = model.initial_params();
    let mut vars = model.initial_vars(&params);
    let sampler = Sampler::new();
    loop {
        let mut stats = ParamStats::new();
        let stats =
            loop {
                let sample = sampler.sample(&model, &params, &mut vars);
                stats.add_sample(sample);
                if stats.ready_for_param_estimate() {
                    break stats;
                }
            };
        let estimate = stats.estimate_params();
        params = estimate.params;
        if estimate.is_done { break }
    }
    Ok(params)
}