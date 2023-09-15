mod model;

use crate::data::{load_training_data, TrainData};
use crate::error::Error;
use crate::options::config::Config;
use crate::train::model::TrainModel;

pub(crate) fn train_or_check(config: &Config, dry: bool) -> Result<(), Error> {
    let data = load_training_data(config)?;
    println!("Loaded data for {} variants", data.beta_se_lists.len());
    println!("{}", data);
    if !dry {
        train(data)?;
    }
    Ok(())
}

fn train(data: TrainData) -> Result<(), Error> {
    let model = TrainModel::new(data);
    let params = model.initial_params();

    todo!()
}