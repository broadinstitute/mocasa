use std::fs::read_to_string;
use serde::Deserialize;
use crate::error::Error;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) gwas: Vec<GWAS>,
    pub(crate) train: Train,
    pub(crate) classify: Classify
}

#[derive(Deserialize)]
pub(crate) struct Train {
    pub(crate) ids_file: String
}

#[derive(Deserialize)]
pub(crate) struct Classify {
    pub(crate) ids_file: String
}

#[derive(Deserialize)]
pub(crate) struct GWAS {
    pub(crate) file: String
}

pub(crate) fn load_config(file: &str) -> Result<Config, Error> {
    let string = read_to_string(file)?;
    let config: Config = toml::from_str(&string)?;
    Ok(config)
}