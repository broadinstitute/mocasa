use crate::error::Error;
use crate::options::cli::{Choice, get_choice};
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
mod phenet;
mod params;

pub fn run() -> Result<(), Error> {
    match get_choice()? {
        Choice::Core(core_options) => {
            let config = load_config(&core_options.config_file)?;
            check_prerequisites(&config)?;
            match core_options.action {
                Action::Train => { train::train_or_check(&config, core_options.dry) }
                Action::Classify => { classify::classify_or_check(&config, core_options.dry) }
            }
        }
        Choice::ImportPhenet(options) => { phenet::import_phenet(&options) }
        Choice::ScaleSigmas(options) => {}
    }
}
