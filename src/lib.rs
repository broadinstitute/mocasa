use crate::error::Error;
use crate::options::cli::get_cli_options;
use crate::options::config::load_config;
use crate::options::action::Action;

mod model;
mod options;
mod error;
mod train;
mod classify;
mod data;

pub fn run() -> Result<(), Error> {
    let cli_options = get_cli_options()?;
    let config = load_config(&cli_options.config_file)?;
    match cli_options.action {
        Action::Train => { train::train(&config) }
        Action::Classify => { classify::classify(&config) }
    }
}
