use std::str::FromStr;
use clap::{Arg, ArgMatches, command, Command};
use log::Level;
use crate::error::Error;
use crate::options::action;
use crate::options::action::Action;

mod params {
    pub(crate) const CONFIG_FILE: &str = "conf-file";
    pub(crate) const CONFIG_FILE_SHORT: char = 'f';
    pub(crate) const DRY: &str = "dry";
    pub(crate) const DRY_SHORT: char = 'd';
    pub(crate) const LOG: &str = "log";
    pub(crate) const LOG_SHORT: char = 'l';
    pub(crate) const PHENET_FILE: &str = "phenet-file";
    pub(crate) const PHENET_FILE_SHORT: char = 'i';
    pub(crate) const PARAMS_FILE: &str = "params-file";
    pub(crate) const PARAMS_FILE_SHORT: char = 'p';
    pub(crate) const IN_FILE: &str = "in-file";
    pub(crate) const IN_FILE_SHORT: char = 'i';
    pub(crate) const SCALE: &str = "scale";
    pub(crate) const SCALE_SHORT: char = 's';
    pub(crate) const OUT_FILE: &str = "out-file";
    pub(crate) const OUT_FILE_SHORT: char = 'o';
    pub(crate) const N_CHUNKS: &str = "n-chunks";
    pub(crate) const N_CHUNKS_SHORT: char = 'x';
    pub(crate) const I_CHUNK: &str = "i-chunk";
    pub(crate) const I_CHUNK_SHORT: char = 'k';
}

mod commands {
    pub(crate) const IMPORT_PHENET: &str = "import-phenet";
    pub(crate) const SCALE_SIGMAS: &str = "scale-sigmas";
}

pub struct Chunking {
    pub(crate) n_chunks: usize,
    pub(crate) i_chunk: usize,
}

pub struct Flags {
    pub(crate) dry: bool,
    pub(crate) log_level: Level,
    pub(crate) chunking: Option<Chunking>,
}

pub struct CoreOptions {
    pub(crate) action: Action,
    pub(crate) config_file: String,
    pub(crate) flags: Flags,
}

pub struct ImportPhenetOptions {
    pub(crate) phenet_file: String,
    pub(crate) params_file: String,
    pub(crate) config_file: String,
    pub(crate) out_file: String,
}

pub(crate) struct ScaleSigmasOptions {
    pub(crate) in_file: String,
    pub(crate) scale: f64,
    pub(crate) out_file: String,
}

pub(crate) enum Choice {
    Core(CoreOptions),
    ImportPhenet(ImportPhenetOptions),
    ScaleSigmas(ScaleSigmasOptions),
}

fn new_arg(name: &'static str, short: char) -> Arg {
    Arg::new(name).short(short).long(name)
}

fn new_action_command(name: &'static str) -> Command {
    Command::new(name)
        .arg_required_else_help(true)
        .arg(new_arg(params::CONFIG_FILE, params::CONFIG_FILE_SHORT))
        .arg(new_arg(params::DRY, params::DRY_SHORT).num_args(0)
            .action(clap::ArgAction::SetTrue))
        .arg(new_arg(params::N_CHUNKS, params::N_CHUNKS_SHORT)
            .value_parser(clap::value_parser!(usize)))
        .arg(new_arg(params::I_CHUNK, params::I_CHUNK_SHORT)
            .value_parser(clap::value_parser!(usize)))
        .arg(new_arg(params::LOG, params::LOG_SHORT))
}

fn new_import_phenet_command() -> Command {
    Command::new(commands::IMPORT_PHENET)
        .arg(new_arg(params::PHENET_FILE, params::PHENET_FILE_SHORT))
        .arg(new_arg(params::PARAMS_FILE, params::PARAMS_FILE_SHORT))
        .arg(new_arg(params::CONFIG_FILE, params::CONFIG_FILE_SHORT))
        .arg(new_arg(params::OUT_FILE, params::OUT_FILE_SHORT))
}

fn new_scale_sigmas_command() -> Command {
    Command::new(commands::SCALE_SIGMAS)
        .arg(new_arg(params::IN_FILE, params::IN_FILE_SHORT))
        .arg(new_arg(params::SCALE, params::SCALE_SHORT)
            .value_parser(clap::value_parser!(f64)))
        .arg(new_arg(params::OUT_FILE, params::OUT_FILE_SHORT))
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
    let log_level =
        sub_matches.get_one::<String>(params::LOG)
            .map(|log_str| Level::from_str(log_str)).transpose()?
            .unwrap_or(Level::Warn);
    let n_chunks = sub_matches.get_one::<usize>(params::N_CHUNKS).cloned();
    let i_chunk = sub_matches.get_one::<usize>(params::I_CHUNK).cloned();
    let chunking =
        match (n_chunks, i_chunk) {
            (Some(n_chunks), Some(i_chunk)) =>
                if i_chunk > n_chunks {
                    Err(Error::from("i_chunk cannot be larger than n_chunks."))
                } else if i_chunk == 0 {
                    Err(Error::from("i_chunk must be larger than zero."))
                } else {
                    Ok(Some(Chunking { n_chunks, i_chunk }))
                },
            (None, None) => Ok(None),
            _ => Err(Error::from(
                "Both or neither of n-chunks and i-chunk must be specified."
            ))
        }?;
    let flags = Flags { dry, log_level, chunking };
    Ok(CoreOptions { action, config_file, flags })
}

fn get_import_phenet_options(sub_matches: &ArgMatches) -> Result<ImportPhenetOptions, Error> {
    let phenet_file =
        sub_matches.get_one::<String>(params::PHENET_FILE).cloned().ok_or_else(|| {
            missing_option_error("phenet opts file", params::PHENET_FILE,
                                 params::PHENET_FILE_SHORT)
        })?;
    let params_file =
        sub_matches.get_one::<String>(params::PARAMS_FILE).cloned().ok_or_else(|| {
            missing_option_error("Mocasa parameters file", params::PARAMS_FILE,
                                 params::PARAMS_FILE_SHORT)
        })?;
    let config_file =
        sub_matches.get_one::<String>(params::CONFIG_FILE).cloned().ok_or_else(|| {
            missing_option_error("Mocasa config file", params::CONFIG_FILE,
                                 params::CONFIG_FILE_SHORT)
        })?;
    let out_file =
        sub_matches.get_one::<String>(params::OUT_FILE).cloned().ok_or_else(|| {
            missing_option_error("Mocasa classification output file", params::OUT_FILE,
                                 params::OUT_FILE_SHORT)
        })?;
    Ok(ImportPhenetOptions { phenet_file, params_file, config_file, out_file })
}

fn get_scale_sigmas_options(sub_matches: &ArgMatches) -> Result<ScaleSigmasOptions, Error> {
    let in_file =
        sub_matches.get_one::<String>(params::IN_FILE).cloned().ok_or_else(|| {
            missing_option_error("input params file", params::IN_FILE,
                                 params::IN_FILE_SHORT)
        })?;
    let scale =
        sub_matches.get_one::<f64>(params::SCALE).cloned().ok_or_else(|| {
            missing_option_error("scale", params::SCALE,
                                 params::SCALE_SHORT)
        })?;
    let out_file =
        sub_matches.get_one::<String>(params::OUT_FILE).cloned().ok_or_else(|| {
            missing_option_error("output params file", params::OUT_FILE,
                                 params::OUT_FILE_SHORT)
        })?;
    Ok(ScaleSigmasOptions { in_file, scale, out_file })
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
        .subcommand(new_scale_sigmas_command())
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
        Some((commands::SCALE_SIGMAS, sub_matches)) => {
            let scale_sigmas_options = get_scale_sigmas_options(sub_matches)?;
            Ok(Choice::ScaleSigmas(scale_sigmas_options))
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



