use crate::error::Error;
use crate::options::cli::get_cli_options;
use crate::options::config::load_config;
use crate::options::action::Action;
use crate::options::check_pre::check_prerequisites;

mod options;
mod error;
mod train;
mod classify;
mod data;
mod math;
mod util;
mod report;

pub fn run() -> Result<(), Error> {
    let cli_options = get_cli_options()?;
    let config = load_config(&cli_options.config_file)?;
    check_prerequisites(&config)?;
    match cli_options.action {
        Action::Train => { train::train_or_check(&config, cli_options.dry) }
        Action::Classify => { classify::classify(&config) }
    }
}
