use std::fs::read_to_string;
use crate::error::Error;
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

pub(crate) fn classify(_config: &Config) -> Result<(), Error> {
    todo!()
}

fn read_params(file: &str) -> Result<Params, Error> {
    let params_string = read_to_string(file)?;
    let params = serde_json::from_str(&params_string)?;
    Ok(params)
}