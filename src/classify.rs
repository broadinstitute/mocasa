use std::fs::read_to_string;
use crate::data::load_data;
use crate::error::{Error, for_file};
use crate::options::action::Action;
use crate::options::config::Config;
use crate::train::params::Params;

pub(crate) fn classify_or_check(config: &Config, dry: bool) -> Result<(), Error> {
    if dry {
        println!("User picked dry run only, so doing nothing.")
    } else {
        classify(config)?;
    }
    Ok(())
}

pub(crate) fn classify(config: &Config) -> Result<(), Error> {
    let params = read_params(&config.files.params)?;
    let data = load_data(config, Action::Classify)?;
    todo!()
}

fn read_params(file: &str) -> Result<Params, Error> {
    let params_string = for_file(file, read_to_string(file))?;
    let params = serde_json::from_str(&params_string)?;
    Ok(params)
}