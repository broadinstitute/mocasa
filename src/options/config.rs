use std::fs::read_to_string;
use serde::Deserialize;
use crate::error::Error;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) gwas: Vec<Gwas>,
    pub(crate) train: Train,
}

#[derive(Deserialize)]
pub(crate) struct Train {
    pub(crate) ids_file: String,
    pub(crate) n_steps_record: usize,
    pub(crate) n_steps_burn_in: usize
}

#[derive(Deserialize)]
pub(crate) struct Gwas {
    pub(crate) name: String,
    pub(crate) file: String
}

pub(crate) fn load_config(file: &str) -> Result<Config, Error> {
    let string = read_to_string(file)?;
    let config: Config = toml::from_str(&string)?;
    Ok(config)
}