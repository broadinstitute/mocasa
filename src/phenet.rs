use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::error::Error;
use crate::options::cli::ImportPhenetOptions;

mod keys {
    pub(crate) const VAR_ID_FILE: &str = "var_id_file";
    pub(crate) const CONFIG_FILE: &str = "config_file";
}

struct PhenetOpts {
    var_id_file: String,
    config_files: Vec<String>,
}

pub(crate) fn import_phenet(options: &ImportPhenetOptions) -> Result<(), Error> {
    read_opts_file(&options.phenet_file)?;
    Ok(())
}

fn missing_option_error(key: &str) -> Error {
    Error::from(format!("Missing option {}", key))
}

fn read_opts_file(opts_file: &str) -> Result<PhenetOpts, Error> {
    let mut var_id_file: Option<String> = None;
    let mut config_files: Vec<String> = Vec::new();
    for line in BufReader::new(File::open(opts_file)?).lines() {
        let line = line?;
        let line = line.trim();
        if !line.starts_with('#') {
            let mut parts_iter = line.split(char::is_whitespace);
            match (parts_iter.next(), parts_iter.next(), parts_iter.next()) {
                (Some(key), Some(value), None) => {
                    match key {
                        keys::VAR_ID_FILE => { var_id_file = Some(value.to_string()) }
                        keys::CONFIG_FILE => { config_files.push(value.to_string()) }
                        _ => { /* do nothing */ }
                    }
                    println!("{} {}", key, value)
                }
                _ => {
                    Err(Error::from(
                        format!("Expected key/value pair, but got '{}'.", line)
                    ))?;
                }
            }
        }
    }
    let var_id_file =
        var_id_file.ok_or_else(|| missing_option_error(keys::CONFIG_FILE))?;
    if config_files.is_empty() {
        Err(missing_option_error(keys::CONFIG_FILE))?;
    }
    Ok(PhenetOpts { var_id_file, config_files })
}