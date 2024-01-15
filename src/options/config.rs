use std::fs::read_to_string;
use serde::Deserialize;
use crate::error::Error;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) files: FilesConfig,
    pub(crate) gwas: Vec<GwasConfig>,
    pub(crate) train: TrainConfig,
    pub(crate) classify: ClassifyConfig,
}

#[derive(Deserialize)]
pub(crate) struct GwasConfig {
    pub(crate) name: String,
    pub(crate) file: String
}

#[derive(Deserialize)]
pub(crate) struct FilesConfig {
    pub(crate) trace: Option<String>,
    pub(crate) params: String
}

#[derive(Deserialize, Clone)]
pub(crate) struct TrainConfig {
    pub(crate) ids_file: String,
    pub(crate) n_steps_burn_in: usize,
    pub(crate) n_samples_per_iteration: usize,
    pub(crate) n_iterations_per_round: usize,
    pub(crate) n_rounds: usize,
}

#[derive(Deserialize, Clone)]
pub(crate) struct ClassifyConfig {
    pub(crate) n_steps_burn_in: usize,
    pub(crate) n_samples: usize,
    pub(crate) out_file: String,
    pub(crate) trace: Option<bool>
}

pub(crate) fn load_config(file: &str) -> Result<Config, Error> {
    let string = read_to_string(file)?;
    let config: Config = toml::from_str(&string)?;
    Ok(config)
}
