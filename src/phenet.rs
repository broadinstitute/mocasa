use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::error::{Error, for_file};
use crate::options::cli::ImportPhenetOptions;

mod keys {
    pub(crate) const VAR_ID_FILE: &str = "var_id_file";
    pub(crate) const CONFIG_FILE: &str = "config_file";
    pub(crate) const OUTPUT_FILE: &str = "output_file";
    pub(crate) const DECLARE: &str = "declare";
    pub(crate) const TRAIT: &str = "trait";
    pub(crate) const ENDO: &str = "endo";
    pub(crate) const FILE: &str = "file";
}

struct PhenetOpts {
    var_id_file: String,
    config_files: Vec<String>,
    output_file: Option<String>,
}

struct ConfigBuilder {
    trait_names: Vec<String>,
    endo_name: Option<String>,
    files: BTreeMap<String, String>
}

impl ConfigBuilder {
    fn new() -> ConfigBuilder {
        let trait_names: Vec<String> = Vec::new();
        let endo_name: Option<String> = None;
        let files: BTreeMap<String, String> = BTreeMap::new();
        ConfigBuilder { trait_names, endo_name, files }
    }
    fn read_phenet_config(&mut self, file: &str) -> Result<(), Error> {
        let file = for_file(file, File::open(file))?;
        let reader = BufReader::new(file);
        self.parse_phenet_config(reader)?;
        Ok(())
    }
    fn read_phenet_config_optional(&mut self, file: &str) -> Result<(), Error> {
        match for_file(file, File::open(file)) {
            Ok(file) => {
                let reader = BufReader::new(file);
                self.parse_phenet_config(reader)?;
            }
            Err(error) => {
                println!("{}", error);
                println!("Since this file was optional, we proceed.");
            }
        }
        Ok(())
    }
    fn parse_phenet_config(&mut self, reader: BufReader<File>) -> Result<(), Error> {
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if !line.is_empty() {
                let mut parts_iter = line.split(char::is_whitespace);
                match (parts_iter.next(), parts_iter.next(), parts_iter.next(), parts_iter.next()) {
                    (Some(part1), Some(part2), Some(part3), None) => {
                        match (part1, part2, part3) {
                            (trait_name, keys::DECLARE, keys::TRAIT) => {
                                let trait_name = trait_name.to_string();
                                if !self.trait_names.contains(&trait_name) {
                                    self.trait_names.push(trait_name.to_string());
                                }
                            }
                            (endo_name, keys::DECLARE, keys::ENDO) => {
                                self.endo_name = Some(endo_name.to_string());
                            }
                            (trait_name, keys::FILE, file) => {
                                self.files.insert(trait_name.to_string(),
                                                  file.to_string());
                            }
                            _ => {
                                println!("Ignoring line '{}'.", line)
                            }
                        }
                    }
                    _ => {
                        Err(Error::from(format!("Cannot parse line: '{}'.", line)))?;
                    }
                }
            }
        }
        Ok(())
    }
    fn report(&self) {
        println!("Trait names: {}", self.trait_names.join(", "));
        if let Some(endo_name) = &self.endo_name {
            println!("Endo name: {}", endo_name)
        }
        let file_default =  "<no file>".to_string();
        for trait_name in &self.trait_names {
            let file = self.files.get(trait_name).unwrap_or(&file_default);
            println!("{}\t{}", trait_name, file);
        }
    }
}

pub(crate) fn import_phenet(options: &ImportPhenetOptions) -> Result<(), Error> {
    let phenet_opts = read_opts_file(&options.phenet_file)?;
    let mut config_builder = ConfigBuilder::new();
    for config_file in phenet_opts.config_files.iter() {
        config_builder.read_phenet_config(config_file)?;
    }
    if let Some(output_file) = &phenet_opts.output_file {
        config_builder.read_phenet_config_optional(output_file)?;
    }
    config_builder.report();
    Ok(())
}

fn missing_option_error(key: &str) -> Error {
    Error::from(format!("Missing option {}", key))
}

fn read_opts_file(opts_file: &str) -> Result<PhenetOpts, Error> {
    let mut var_id_file: Option<String> = None;
    let mut config_files: Vec<String> = Vec::new();
    let mut output_file: Option<String> = None;
    let opts_file = for_file(opts_file, File::open(opts_file))?;
    for line in BufReader::new(opts_file).lines() {
        let line = line?;
        let line = line.trim();
        if !line.starts_with('#') && !line.is_empty() {
            let mut parts_iter = line.split(char::is_whitespace);
            match (parts_iter.next(), parts_iter.next(), parts_iter.next()) {
                (Some(key), Some(value), None) => {
                    match key {
                        keys::VAR_ID_FILE => { var_id_file = Some(value.to_string()) }
                        keys::CONFIG_FILE => { config_files.push(value.to_string()) }
                        keys::OUTPUT_FILE => { output_file = Some(value.to_string()) }
                        _ => { println!("Ignoring option: {}: {}", key, value) }
                    }
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
        var_id_file.ok_or_else(|| missing_option_error(keys::VAR_ID_FILE))?;
    if config_files.is_empty() {
        Err(missing_option_error(keys::CONFIG_FILE))?;
    }
    Ok(PhenetOpts { var_id_file, config_files, output_file })
}