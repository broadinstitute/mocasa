use clap::{Arg, ArgMatches, command, Command};
use crate::error::Error;
use crate::options::action;
use crate::options::action::Action;

mod params {
    pub(crate) const CONFIG_FILE: &str = "conf-file";
    pub(crate) const CONFIG_FILE_SHORT: char = 'f';
    pub(crate) const DRY: &str = "dry";
    pub(crate) const DRY_SHORT: char = 'd';
    pub(crate) const PHENET_FILE: &str = "phenet-file";
    pub(crate) const PHENET_FILE_SHORT: char = 'i';
    pub(crate) const PARAMS_FILE: &str = "params-file";
    pub(crate) const PARAMS_FILE_SHORT: char = 'p';
    pub(crate) const OUT_FILE: &str = "out-file";
    pub(crate) const OUT_FILE_SHORT: char = 'o';
}

mod commands {
    pub(crate) const IMPORT_PHENET: &str = "import-phenet";
}

pub struct CoreOptions {
    pub(crate) action: Action,
    pub(crate) config_file: String,
    pub(crate) dry: bool,
}

pub struct ImportPhenetOptions {
    pub(crate) phenet_file: String,
    pub(crate) params_file: String,
    pub(crate) config_file: String,
    pub(crate) out_file: String,
}

pub(crate) enum Choice {
    Core(CoreOptions),
    ImportPhenet(ImportPhenetOptions),
}

fn new_action_command(name: &'static str) -> Command {
    Command::new(name)
        .arg_required_else_help(true)
        .arg(Arg::new(params::CONFIG_FILE).short(params::CONFIG_FILE_SHORT)
            .long(params::CONFIG_FILE))
        .arg(Arg::new(params::DRY).short(params::DRY_SHORT).long(params::DRY)
            .num_args(0).action(clap::ArgAction::SetTrue))
}

fn new_import_phenet_command() -> Command {
    Command::new(commands::IMPORT_PHENET)
        .arg(Arg::new(params::PHENET_FILE).short(params::PHENET_FILE_SHORT)
            .long(params::PHENET_FILE))
        .arg(Arg::new(params::PARAMS_FILE).short(params::PARAMS_FILE_SHORT)
            .long(params::PARAMS_FILE))
        .arg(Arg::new(params::CONFIG_FILE).short(params::CONFIG_FILE_SHORT)
            .long(params::CONFIG_FILE))
        .arg(Arg::new(params::OUT_FILE).short(params::OUT_FILE_SHORT)
            .long(params::OUT_FILE))
}

fn missing_option_error(name: &str, long: &str, short: char) -> Error {
    Error::from(format!("Missing {} option ('--{}' or '-{}').", name, long, short))
}

fn get_core_options(action: Action, sub_matches: &ArgMatches) -> Result<CoreOptions, Error> {
    let config_file =
        sub_matches.get_one::<String>(params::CONFIG_FILE).cloned().ok_or_else(|| {
            missing_option_error("config file", params::CONFIG_FILE,
                                 params::CONFIG_FILE_SHORT)
        })?;
    let dry = sub_matches.get_flag(params::DRY);
    Ok(CoreOptions { action, config_file, dry })
}

fn get_import_phenet_options(sub_matches: &ArgMatches) -> Result<ImportPhenetOptions, Error> {
    let phenet_file =
        sub_matches.get_one::<String>(params::PHENET_FILE).cloned().ok_or_else(|| {
            Error::from("Missing phenet file option")
        })?;
    let params_file =
        sub_matches.get_one::<String>(params::PARAMS_FILE).cloned().ok_or_else(|| {
            Error::from("Missing params file option")
        })?;
    let config_file =
        sub_matches.get_one::<String>(params::CONFIG_FILE).cloned().ok_or_else(|| {
            Error::from("Missing config file option")
        })?;
    let out_file =
        sub_matches.get_one::<String>(params::CONFIG_FILE).cloned().ok_or_else(|| {
            Error::from("Missing output file option")
        })?;
    Ok(ImportPhenetOptions { phenet_file, params_file, config_file, out_file })
}

fn known_subcommands_message() -> String {
    format!("Known subcommands are {}, {} and {}.", action::names::TRAIN, action::names::CLASSIFY,
            commands::IMPORT_PHENET)
}

pub(crate) fn get_choice() -> Result<Choice, Error> {
    let matches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(new_action_command(action::names::TRAIN))
        .subcommand(new_action_command(action::names::CLASSIFY))
        .subcommand(new_import_phenet_command())
        .get_matches();
    match matches.subcommand() {
        Some((action::names::TRAIN, sub_matches)) => {
            let core_options = get_core_options(Action::Train, sub_matches)?;
            Ok(Choice::Core(core_options))
        }
        Some((action::names::CLASSIFY, sub_matches)) => {
            let core_options = get_core_options(Action::Classify, sub_matches)?;
            Ok(Choice::Core(core_options))
        }
        Some((commands::IMPORT_PHENET, sub_matches)) => {
            let import_phenet_options = get_import_phenet_options(sub_matches)?;
            Ok(Choice::ImportPhenet(import_phenet_options))
        }
        Some((subcommand, _)) => {
            Err(Error::from(
                format!("Unknown subcommand {}. {}", subcommand,
                        known_subcommands_message()),
            ))?
        }
        None => {
            Err(Error::from(
                format!("Need to specify subcommand. {}", known_subcommands_message()),
            ))?
        }
    }
}



