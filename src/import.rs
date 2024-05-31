use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::num::ParseFloatError;
use std::path::Path;
use log::warn;
use crate::error::{Error, for_file};
use crate::options::cli::ImportParamsOptions;
use crate::options::config::{load_config, write_config};

pub(crate) fn import(options: &ImportParamsOptions) -> Result<(), Error> {
    let mut config = load_config(&options.in_file)?;
    if let Some(variants_file) = &options.variants {
        if !Path::new(variants_file).exists() {
            warn!("Variants file ({}) does not exist.", variants_file);
        }
        config.classify.only_ids = None;
        config.classify.only_ids_file = Some(variants_file.clone());
    }
    write_config(&config, &options.config_file)?;
    Ok(())
}

struct Endos {
    endos: Vec<String>,
    mus: Vec<f64>,
    taus: Vec<f64>,
}

fn next_line_prefix<T>(prefix: &str, lines: &mut Lines<BufReader<File>>, f: fn(&str) -> T)
                       -> Result<Vec<T>, Error> {
    match lines.next() {
        None => Err(Error::from("Unexpected end of file")),
        Some(line) => {
            let line = line?;
            let mut parts = line.split('\t');
            match parts.next() {
                None => { Err(Error::from("Unexpected end of line")) }
                Some(part) => {
                    if part == prefix {
                        Ok(parts.map(f).collect::<Vec<T>>())
                    } else {
                        Err(Error::from(
                            format!("Expected line starting with '{}', got '{}'",
                                    prefix, part))
                        )
                    }
                }
            }
        }
    }
}

fn next_line_prefix_strings(prefix: &str, lines: &mut Lines<BufReader<File>>)
                            -> Result<Vec<String>, Error> {
    next_line_prefix(prefix, lines, |part| part.to_string())
}

fn next_line_prefix_numbers(prefix: &str, lines: &mut Lines<BufReader<File>>)
                            -> Result<Vec<f64>, Error> {
    next_line_prefix(prefix, lines, |part| part.parse::<f64>())?.into_iter()
        .collect::<Result<Vec<f64>, ParseFloatError>>().map_err(Error::from)
}



fn load_endos(file: &str) -> Result<Endos, Error> {
    let mut lines =
        BufReader::new(for_file(file, File::open(file))?).lines();
    let endos = next_line_prefix_strings("endos", &mut lines)?;
    let mus = next_line_prefix_numbers("mus", &mut lines)?;
    let taus = next_line_prefix_numbers("taus", &mut lines)?;
    Ok(Endos { endos, mus, taus })
}