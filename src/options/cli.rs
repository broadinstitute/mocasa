use clap::{Arg, command, Command};
use crate::error::Error;
use crate::options::action;
use crate::options::action::Action;

mod params {
    pub(crate) const CONFIG_FILE: &str = "conf-file";
    pub(crate) const CONFIG_FILE_SHORT: char = 'f';
    pub(crate) const DRY: &str = "dry";
    pub(crate) const DRY_SHORT: char = 'd';
}

pub struct CliOptions {
    pub(crate) action: Action,
    pub(crate) config_file: String,
    dry: bool,
}

fn new_action_command(name: &'static str) -> Command {
    Command::new(name)
        .arg_required_else_help(true)
        .arg(Arg::new(params::CONFIG_FILE).short(params::CONFIG_FILE_SHORT)
            .long(params::CONFIG_FILE))
        .arg(Arg::new(params::DRY).short(params::DRY_SHORT).long(params::DRY)
            .num_args(0).action(clap::ArgAction::SetTrue))
}

pub(crate) fn get_cli_options() -> Result<CliOptions, Error> {
    let matches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(new_action_command(action::names::TRAIN))
        .subcommand(new_action_command(action::names::CLASSIFY))
        .get_matches();
    let (action, sub_matches) =
        match matches.subcommand() {
            Some((action::names::TRAIN, sub_matches)) => {
                (Action::Train, sub_matches)
            }
            Some((action::names::CLASSIFY, sub_matches)) => {
                (Action::Classify, sub_matches)
            }
            Some((subcommand, _)) => {
                Err(Error::from(format!("Unknown action {}. Known actions are {} and {}.",
                                        subcommand, action::names::TRAIN, action::names::CLASSIFY),
                ))?
            }
            None => {
                Err(Error::from(format!("Need to specify action {} or {}.",
                                        action::names::TRAIN, action::names::CLASSIFY),
                ))?
            }
        };
    let config_file =
        sub_matches.get_one::<String>(params::CONFIG_FILE).cloned().ok_or_else(|| {
            Error::from("Missing config file option")
        })?;
    let dry = sub_matches.get_flag(params::DRY);
    Ok(CliOptions { action, config_file, dry })
}



