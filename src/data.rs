mod gwas;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::data::gwas::{GwasReader, GwasRecord};
use crate::error::Error;
use crate::options::config::Config;

pub(crate) struct TrainData {
    pub(crate) names: Vec<String>,
    pub(crate) beta_se_lists: BTreeMap<String, Vec<BetaSe>>,
}

pub(crate) struct BetaSe {
    pub(crate) beta: f64,
    pub(crate) se: f64,
}

impl TrainData {
    pub(crate) fn n_data_points(&self) -> usize { self.beta_se_lists.len() }
    pub(crate) fn n_traits(&self) -> usize { self.names.len() }
}

impl Display for BetaSe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "beta={}, se={}", self.beta, self.se)
    }
}

impl Display for TrainData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", gwas::cols::VAR_ID)?;
        for name in &self.names {
            write!(f, "\tbeta_{}\tse_{}", name, name)?;
        }
        writeln!(f)?;
        for (var_id, beta_se_list) in &self.beta_se_lists {
            write!(f, "{}", var_id)?;
            for beta_se in beta_se_list {
                write!(f, "\t{}\t{}", beta_se.beta, beta_se.se)?
            }
            writeln!(f)?
        }
        Ok(())
    }
}

pub(crate) fn load_training_data(config: &Config) -> Result<TrainData, Error> {
    let mut beta_se_lists = load_ids(&config.train.ids_file)?;
    let mut names: Vec<String> = Vec::new();
    for gwas in &config.gwas {
        names.push(gwas.name.clone());
        load_gaws(&mut beta_se_lists, &gwas.file)?;
        check_n_beta_se(&beta_se_lists, names.len())?;
    }
    Ok(TrainData { names, beta_se_lists })
}

fn load_ids(ids_file: &str) -> Result<BTreeMap<String, Vec<BetaSe>>, Error> {
    let mut ids: BTreeMap<String, Vec<BetaSe>> = BTreeMap::new();
    for line in BufReader::new(File::open(ids_file)?).lines() {
        let line = line?.trim().to_string();
        let values: Vec<BetaSe> = Vec::new();
        ids.insert(line, values);
    }
    Ok(ids)
}

fn load_gaws(beta_se_lists: &mut BTreeMap<String, Vec<BetaSe>>, file: &str) -> Result<(), Error> {
    let gwas_reader =
        GwasReader::new(BufReader::new(File::open(file)?))?;
    for gwas_record in gwas_reader {
        let GwasRecord { var_id, beta, se } = gwas_record?;
        if let Some(beta_se_list) = beta_se_lists.get_mut(&var_id) {
            beta_se_list.push(BetaSe { beta, se })
        }
    }
    Ok(())
}

fn check_n_beta_se(beta_ses: &BTreeMap<String, Vec<BetaSe>>, len_expected: usize)
                   -> Result<(), Error> {
    for (var_id, beta_se_list) in beta_ses {
        match beta_se_list.len().cmp(&len_expected) {
            Ordering::Less => {
                Err(Error::from(format!("Missing value for {}.", var_id)))?
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                Err(Error::from(format!("Duplicate lines for {}.", var_id)))?
            }
        }
    }
    Ok(())
}

