mod model;
mod param_stats;
mod sampler;
mod params;
mod vars;
mod param_eval;

use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::data::{load_training_data, TrainData};
use crate::error::Error;
use crate::options::config::Config;
use crate::train::model::TrainModel;
use crate::train::param_stats::ParamHessianStats;
use crate::train::params::Params;
use crate::train::sampler::Sampler;

pub(crate) fn train_or_check(config: &Config, dry: bool) -> Result<(), Error> {
    let data = load_training_data(config)?;
    println!("Loaded data for {} variants", data.meta.n_data_points());
    println!("{}", data);
    if !dry {
        train(data)?;
    }
    Ok(())
}

fn train(data: TrainData) -> Result<Params, Error> {
    let meta = data.meta.clone();
    let model = TrainModel::new(data);
    let mut params = model.initial_params();
    let mut vars = model.initial_vars(&params);
    let rng = thread_rng();
    let mut sampler = Sampler::<ThreadRng>::new(meta, rng);
    loop {
        let mut stats = ParamHessianStats::new(model.meta().n_traits());
        let stats =
            loop {
                sampler.sample(&model, &params, &mut vars);
                let param_eval = model.param_eval(&params, &vars);
                stats.survey_param_eval(&param_eval);
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