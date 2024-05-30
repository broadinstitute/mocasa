use std::str::FromStr;
use clap::{Arg, ArgMatches, command, Command};
use log::Level;
use crate::error::Error;
use crate::options::action;
use crate::options::action::Action;
use crate::options::cli::params::ArgDef;

mod params {
    pub(crate) struct ArgDef {
        pub(crate) name: &'static str,
        pub(crate) short: char,
    }

    impl ArgDef {
        const fn new(name: &'static str, short: char) -> ArgDef {
            ArgDef { name, short }
        }
    }

    pub(crate) const CONFIG_FILE: ArgDef = ArgDef::new("conf-file", 'f');
    pub(crate) const DRY: ArgDef = ArgDef::new("dry", 'd');
    pub(crate) const LOG: ArgDef = ArgDef::new("log",'l');
    pub(crate) const PHENET_FILE: ArgDef = ArgDef::new("phenet-file",'i');
    pub(crate) const PARAMS_FILE: ArgDef = ArgDef::new("params-file", 'p');
    pub(crate) const IN_FILE: ArgDef = ArgDef::new("in-file", 'i');
    pub(crate) const SCALE: ArgDef = ArgDef::new("scale", 's');
    pub(crate) const OUT_FILE: ArgDef = ArgDef::new("out-file", 'o');
    pub(crate) const N_CHUNKS: ArgDef = ArgDef::new("n-chunks", 'x');
    pub(crate) const I_CHUNK: ArgDef = ArgDef::new("i-chunk", 'k');
    pub(crate) const VARIANTS: ArgDef = ArgDef::new("variants", 'v');
    pub(crate) const BETAS: ArgDef = ArgDef::new("betas", 'b');
    pub(crate) const SIGMAS: ArgDef = ArgDef::new("sigmas", 's');
    pub(crate) const ENDOS: ArgDef = ArgDef::new("endos", 'e');
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

pub(crate) struct ImportParamsOptions {
    pub(crate) in_file: String,
    pub(crate) variants: Option<String>,
    pub(crate) betas: String,
    pub(crate) sigmas: String,
    pub(crate) endos: String,
    pub(crate) params_file: String,
    pub(crate) config_file: String,
}

pub(crate) enum Choice {
    Core(CoreOptions),
    ImportPhenet(ImportPhenetOptions),
    ScaleSigmas(ScaleSigmasOptions),
    ImportParams(ImportParamsOptions),
}

fn new_arg(arg_def: ArgDef) -> Arg {
    Arg::new(arg_def.name).short(arg_def.short).long(arg_def.name)
}

fn new_action_command(name: &'static str) -> Command {
    Command::new(name)
        .arg_required_else_help(true)
        .arg(new_arg(params::CONFIG_FILE))
        .arg(new_arg(params::DRY).num_args(0).action(clap::ArgAction::SetTrue))
        .arg(new_arg(params::N_CHUNKS).value_parser(clap::value_parser!(usize)))
        .arg(new_arg(params::I_CHUNK).value_parser(clap::value_parser!(usize)))
        .arg(new_arg(params::LOG))
}

fn new_import_phenet_command() -> Command {
    Command::new(commands::IMPORT_PHENET)
        .arg(new_arg(params::PHENET_FILE))
        .arg(new_arg(params::PARAMS_FILE))
        .arg(new_arg(params::CONFIG_FILE))
        .arg(new_arg(params::OUT_FILE))
}

fn new_scale_sigmas_command() -> Command {
    Command::new(commands::SCALE_SIGMAS)
        .arg(new_arg(params::IN_FILE))
        .arg(new_arg(params::SCALE).value_parser(clap::value_parser!(f64)))
        .arg(new_arg(params::OUT_FILE))
}

fn new_import_params_command() -> Command {
    Command::new("import-params")
        .arg(new_arg(params::IN_FILE))
        .arg(new_arg(params::VARIANTS))
        .arg(new_arg(params::BETAS))
        .arg(new_arg(params::SIGMAS))
        .arg(new_arg(params::ENDOS))
        .arg(new_arg(params::PARAMS_FILE))
        .arg(new_arg(params::CONFIG_FILE))
}

fn missing_option_error(name: &str, arg_def: ArgDef) -> Error {
    Error::from(
        format!("Missing {} option ('--{}' or '-{}').", name, arg_def.name, arg_def.short)
    )
}

fn get_core_options(action: Action, sub_matches: &ArgMatches) -> Result<CoreOptions, Error> {
    let config_file =
        sub_matches.get_one::<String>(params::CONFIG_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("config file", params::CONFIG_FILE)
        })?;
    let dry = sub_matches.get_flag(params::DRY.name);
    let log_level =
        sub_matches.get_one::<String>(params::LOG.name)
            .map(|log_str| Level::from_str(log_str)).transpose()?
            .unwrap_or(Level::Warn);
    let n_chunks = sub_matches.get_one::<usize>(params::N_CHUNKS.name).cloned();
    let i_chunk = sub_matches.get_one::<usize>(params::I_CHUNK.name).cloned();
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
        sub_matches.get_one::<String>(params::PHENET_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("phenet opts file", params::PHENET_FILE)
        })?;
    let params_file =
        sub_matches.get_one::<String>(params::PARAMS_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("Mocasa parameters file", params::PARAMS_FILE)
        })?;
    let config_file =
        sub_matches.get_one::<String>(params::CONFIG_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("Mocasa config file", params::CONFIG_FILE)
        })?;
    let out_file =
        sub_matches.get_one::<String>(params::OUT_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("Mocasa classification output file", params::OUT_FILE)
        })?;
    Ok(ImportPhenetOptions { phenet_file, params_file, config_file, out_file })
}

fn get_scale_sigmas_options(sub_matches: &ArgMatches) -> Result<ScaleSigmasOptions, Error> {
    let in_file =
        sub_matches.get_one::<String>(params::IN_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("input params file", params::IN_FILE)
        })?;
    let scale =
        sub_matches.get_one::<f64>(params::SCALE.name).cloned().ok_or_else(|| {
            missing_option_error("scale", params::SCALE)
        })?;
    let out_file =
        sub_matches.get_one::<String>(params::OUT_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("output params file", params::OUT_FILE)
        })?;
    Ok(ScaleSigmasOptions { in_file, scale, out_file })
}

fn get_import_params_options(sub_matches: &ArgMatches) -> Result<ImportParamsOptions, Error> {
    let in_file =
        sub_matches.get_one::<String>(params::IN_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("input config file", params::IN_FILE)
        })?;
    let variants = sub_matches.get_one::<String>(params::VARIANTS.name).cloned();
    let betas =
        sub_matches.get_one::<String>(params::BETAS.name).cloned().ok_or_else(|| {
            missing_option_error("betas file", params::BETAS)
        })?;
    let sigmas =
        sub_matches.get_one::<String>(params::SIGMAS.name).cloned().ok_or_else(|| {
            missing_option_error("sigmas file", params::SIGMAS)
        })?;
    let endos =
        sub_matches.get_one::<String>(params::ENDOS.name).cloned().ok_or_else(|| {
            missing_option_error("endos file", params::ENDOS)
        })?;
    let params_file =
        sub_matches.get_one::<String>(params::PARAMS_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("parameters file", params::PARAMS_FILE)
        })?;
    let config_file =
        sub_matches.get_one::<String>(params::CONFIG_FILE.name).cloned().ok_or_else(|| {
            missing_option_error("output config file", params::CONFIG_FILE)
        })?;
    Ok(ImportParamsOptions { in_file, variants, betas, sigmas, endos, params_file, config_file })
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
        .subcommand(new_import_params_command())
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
        Some(("import-params", sub_matches)) => {
            let import_params_options = get_import_params_options(sub_matches)?;
            Ok(Choice::ImportParams(import_params_options))
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



