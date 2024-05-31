use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::num::ParseFloatError;
use std::path::Path;
use log::warn;
use crate::error::{Error, for_file};
use crate::math::matrix::Matrix;
use crate::options::cli::ImportParamsOptions;
use crate::options::config::{load_config, write_config};
use crate::params::{Params, write_params_to_file};

pub(crate) fn import(options: &ImportParamsOptions) -> Result<(), Error> {
    let mut config = load_config(&options.in_file)?;
    if let Some(variants_file) = &options.variants {
        if !Path::new(variants_file).exists() {
            warn!("Variants file ({}) does not exist.", variants_file);
        }
        config.classify.only_ids = None;
        config.classify.only_ids_file = Some(variants_file.clone());
    }
    config.files.params.clone_from(&options.params_file);
    let endos = load_endos(&options.endos)?;
    config.shared.n_endos = endos.endos.len();
    let betas = load_betas(&options.betas, &endos.endos)?;
    let trait_names =
        config.gwas.iter().map(|gwas_config| gwas_config.name.clone())
            .collect::<Vec<String>>();
    check_same_strings("traits from config", &trait_names, "traits from betas",
                       &betas.traits)?;
    let sigmas = load_sigmas(&options.sigmas)?;
    check_same_strings("traits from config", &trait_names, "traits from sigmas",
                       &sigmas.traits)?;
    let trait_names = trait_names.into();
    let mus = endos.mus;
    let taus = endos.taus;
    let betas = betas.values;
    let sigmas = sigmas.values;
    let params = Params { trait_names, mus, taus, betas, sigmas };
    write_params_to_file(&params, &options.params_file)?;
    write_config(&config, &options.config_file)?;
    Ok(())
}

fn get_lines(file: &str) -> Result<impl Iterator<Item=Result<String, io::Error>>, Error> {
    let lines =
        BufReader::new(for_file(file, File::open(file))?).lines()
            .map(|line| line.map(|line| line.trim().to_string()))
            .filter(|line| {
                if let Ok(line) = line {
                    !(line.is_empty() || line.starts_with('#'))
                } else {
                    true
                }
            });
    Ok(lines)
}

struct Endos {
    endos: Vec<String>,
    mus: Vec<f64>,
    taus: Vec<f64>,
}

struct Betas {
    traits: Vec<String>,
    values: Matrix,
}

struct Sigmas {
    traits: Vec<String>,
    values: Vec<f64>,
}
fn next_line_prefix<L, T>(prefix: &str, lines: &mut L, f: fn(&str) -> T)
                          -> Result<Vec<T>, Error>
    where L: Iterator<Item=Result<String, io::Error>> {
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

fn next_line_prefix_strings<L>(prefix: &str, lines: &mut L)
                            -> Result<Vec<String>, Error>
    where L: Iterator<Item=Result<String, io::Error>> {
    next_line_prefix(prefix, lines, |part| part.to_string())
}

fn next_line_prefix_numbers<L>(prefix: &str, lines: &mut L)
                            -> Result<Vec<f64>, Error>
    where L: Iterator<Item=Result<String, io::Error>> {
    next_line_prefix(prefix, lines, |part| part.parse::<f64>())?.into_iter()
        .collect::<Result<Vec<f64>, ParseFloatError>>().map_err(Error::from)
}

fn assert_no_more_lines<L>(lines: &mut L, file: &str) -> Result<(), Error>
    where L: Iterator<Item=Result<String, io::Error>> {
    match lines.next() {
        None => Ok(()),
        Some(Ok(line)) => {
            Err(Error::from(
                format!("Unexpected line in {}: '{}'", file, line))
            )
        }
        Some(Err(error)) => {
            for_file(file, Err(error))
        }
    }
}
fn check_same_length<T1, T2>(name1: &str, values1: &[T1], name2: &str, values2: &[T2])
                             -> Result<(), Error> {
    if values1.len() != values2.len() {
        Err(Error::from(
            format!("{} has {} values, but {} has {} values.",
                    name1, values1.len(), name2, values2.len()))
        )
    } else {
        Ok(())
    }
}

fn check_same_strings(name1: &str, strings1: &[String], name2: &str, strings2: &[String])
                      -> Result<(), Error> {
    check_same_length(name1, strings1, name2, strings2)?;
    for (i, (string1, string2))
    in strings1.iter().zip(strings2).enumerate() {
        if string1 != string2 {
            return Err(Error::from(
                format!("In position {}, {} has value '{}', but {} has value '{}'.",
                        i, name1, string1, name2, string2))
            );
        }
    }
    Ok(())
}

fn load_endos(file: &str) -> Result<Endos, Error> {
    let mut lines = get_lines(file)?;
    let endos = next_line_prefix_strings("endos", &mut lines)?;
    let mus = next_line_prefix_numbers("mus", &mut lines)?;
    check_same_length("endos", &endos, "mus", &mus)?;
    let taus = next_line_prefix_numbers("taus", &mut lines)?;
    check_same_length("endos", &endos, "taus", &taus)?;
    assert_no_more_lines(&mut lines, file)?;
    Ok(Endos { endos, mus, taus })
}

fn load_betas(file: &str, endos: &[String]) -> Result<Betas, Error> {
    let mut lines = get_lines(file)?;
    let traits = next_line_prefix_strings("traits", &mut lines)?;
    let mut rows : Vec<Vec<f64>> = Vec::new();
    for endo in endos {
        let row = next_line_prefix_numbers(endo, &mut lines)?;
        check_same_length("traits", &traits, endo, &row)?;
        rows.push(row);
    }
    assert_no_more_lines(&mut lines, file)?;
    let values =
        Matrix::fill(endos.len(), traits.len(), |i_row, i_col| rows[i_row][i_col]);
    Ok(Betas { traits, values })
}

fn load_sigmas(file: &str) -> Result<Sigmas, Error> {
    let mut lines = get_lines(file)?;
    let traits = next_line_prefix_strings("traits", &mut lines)?;
    let values = next_line_prefix_numbers("sigmas", &mut lines)?;
    check_same_length("traits", &traits, "sigmas", &values)?;
    assert_no_more_lines(&mut lines, file)?;
    Ok(Sigmas { traits, values })
}
