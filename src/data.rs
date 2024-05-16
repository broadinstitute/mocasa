use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

use crate::data::gwas::{GwasReader, GwasRecord};
use crate::error::{Error, ErrorKind, for_context, for_context_kind, for_file};
use crate::math::matrix::Matrix;
use crate::options::action::Action;
use crate::options::config::{Config, GwasConfig};

pub(crate) mod gwas;


const DELIM_LIST: &[char; 4] = &[';', '\t', ',', ' '];
const MISSING_DELIM_MSG: &str =
    "Can only parse data files with semicolon, tab, comma or single blank as delimiter.";

#[derive(Clone)]
pub(crate) struct Meta {
    pub(crate) trait_names: Arc<Vec<String>>,
    pub(crate) var_ids: Arc<Vec<String>>,
}

pub(crate) struct GwasData {
    pub(crate) meta: Meta,
    pub(crate) betas: Matrix,
    pub(crate) ses: Matrix,
}

pub(crate) struct Weights {
    pub(crate) weights: Vec<f64>,
    pub(crate) sum: f64,
}

pub(crate) struct LoadedData {
    pub(crate) gwas_data: GwasData,
    pub(crate) weights: Weights,
}

#[derive(Clone)]
pub(crate) struct BetaSe {
    pub(crate) beta: f64,
    pub(crate) se: f64,
}

pub(crate) struct IdData {
    beta_se_list: Vec<BetaSe>,
    weight: f64
}

impl Meta {
    pub(crate) fn new(trait_names: Arc<Vec<String>>, var_ids: Arc<Vec<String>>) -> Meta {
        Meta { trait_names, var_ids }
    }
    pub(crate) fn var_ids(&self) -> &[String] { &self.var_ids }
    pub(crate) fn trait_names(&self) -> &[String] { &self.trait_names }
    pub(crate) fn n_data_points(&self) -> usize { self.var_ids().len() }
    pub(crate) fn n_traits(&self) -> usize { self.trait_names().len() }
}

impl GwasData {
    pub(crate) fn n_data_points(&self) -> usize { self.meta.n_data_points() }
    pub(crate) fn n_traits(&self) -> usize { self.meta.n_traits() }
    pub(crate) fn only_data_point(&self, i_row: usize) -> (GwasData, Vec<usize>) {
        let var_id = self.meta.var_ids[i_row].clone();
        let var_ids = Arc::new(vec![var_id]);
        let mut is_col: Vec<usize> = Vec::new();
        for i_col in 0..self.n_traits() {
            if self.betas[i_row][i_col].is_finite() && self.ses[i_row][i_col].is_finite() {
                is_col.push(i_col)
            }
        }
        let trait_names: Arc<Vec<String>> =
            Arc::new(is_col.iter().map(|&i_col|
                self.meta.trait_names[i_col].clone())
                .collect());
        let meta = Meta { var_ids, trait_names };
        let n_cols = meta.n_traits();
        let betas =
            Matrix::fill(1, n_cols, |_, i_i_col| self.betas[i_row][is_col[i_i_col]]);
        let ses =
            Matrix::fill(1, n_cols, |_, i_i_col| self.ses[i_row][is_col[i_i_col]]);
        (GwasData { meta, betas, ses }, is_col)
    }
}

impl Weights {
    fn new(n_data_points: usize) -> Weights {
        let weights: Vec<f64> = Vec::with_capacity(n_data_points);
        let sum: f64 = 0.0;
        Weights { weights, sum }
    }
    fn add(&mut self, weight: f64) {
        self.weights.push(weight);
        self.sum += weight;
    }
}

impl Display for BetaSe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "beta={}, se={}", self.beta, self.se)
    }
}

impl Display for GwasData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", gwas::default_cols::VAR_ID)?;
        for trait_name in self.meta.trait_names() {
            write!(f, "\tbeta_{}\tse_{}", trait_name, trait_name)?;
        }
        writeln!(f)?;
        for (i_data_point, var_id) in self.meta.var_ids().iter().enumerate() {
            write!(f, "{}", var_id)?;
            for (i_trait, _) in self.meta.trait_names().iter().enumerate() {
                write!(f, "\t{}\t{}", self.betas[i_data_point][i_trait],
                       self.ses[i_data_point][i_trait])?
            }
            writeln!(f)?
        }
        Ok(())
    }
}

pub(crate) fn load_data(config: &Config, action: Action) -> Result<LoadedData, Error> {
    let n_traits = config.gwas.len();
    let mut beta_se_by_ids: BTreeMap<String, IdData> =
        match action {
            Action::Train => { load_ids(&config.train.ids_file, n_traits)? }
            Action::Classify => { BTreeMap::new() }
        };
    let mut trait_names: Vec<String> = Vec::with_capacity(n_traits);
    for (i_trait, gwas) in config.gwas.iter().enumerate() {
        trait_names.push(gwas.name.clone());
        load_gaws(&mut beta_se_by_ids, gwas, n_traits, i_trait, action)?;
    }
    let n_data_points = beta_se_by_ids.len();
    let mut var_ids: Vec<String> = Vec::with_capacity(n_data_points);
    let mut betas = Matrix::fill(n_data_points, n_traits, |_, _| f64::NAN);
    let mut ses = Matrix::fill(n_data_points, n_traits, |_, _| f64::NAN);
    let mut weights = Weights::new(n_data_points);
    for (i_data_point, (var_id, id_data))
    in beta_se_by_ids.into_iter().enumerate() {
        var_ids.push(var_id);
        weights.add(id_data.weight);
        for (i_trait, beta_se) in id_data.beta_se_list.into_iter().enumerate() {
            betas[i_data_point][i_trait] = beta_se.beta;
            ses[i_data_point][i_trait] = beta_se.se;
        }
    }
    if action == Action::Train {
        for (i_data_point, var_id) in var_ids.iter().enumerate() {
            for (i_trait, trait_name) in trait_names.iter().enumerate() {
                if betas[i_data_point][i_trait].is_nan() {
                    Err(Error::from(
                        format!("Missing beta for trait {} for var id {}",
                                trait_name, var_id)
                    ))?;
                }
                if ses[i_data_point][i_trait].is_nan() {
                    Err(Error::from(
                        format!("Missing se for trait {} for var id {}",
                                trait_name, var_id)
                    ))?;
                }
            }
        }
    }
    let meta = Meta::new(trait_names.into(), var_ids.into());
    let gwas_data = GwasData { meta, betas, ses };
    Ok(LoadedData { gwas_data, weights })
}

fn load_ids(ids_file: &str, n_traits: usize) -> Result<BTreeMap<String, IdData>, Error> {
    let mut beta_se_by_id: BTreeMap<String, IdData> = BTreeMap::new();
    for line
    in BufReader::new(for_file(ids_file, File::open(ids_file))?).lines() {
        let line = line?;
        let mut fields = line.trim().split(DELIM_LIST);
        if let Some(id) = fields.next() {
            if !id.is_empty() {
                let beta_se_list: Vec<BetaSe> = new_beta_se_list(n_traits);
                let weight =
                    for_context_kind(id, ErrorKind::ParseFloat,
                                     fields.next().map(|s| s.parse::<f64>()).transpose())?
                        .unwrap_or(1.0);
                if weight < 0.0 {
                    Err(Error::from(
                        format!("Negative weight ({weight}) for id {id}")
                    ))?;
                }
                let id_data = IdData { beta_se_list, weight };
                beta_se_by_id.insert(id.to_string(), id_data);
            }
        }
    }
    Ok(beta_se_by_id)
}

fn load_gaws(beta_se_by_id: &mut BTreeMap<String, IdData>, gwas_config: &GwasConfig,
             n_traits: usize, i_trait: usize, action: Action)
             -> Result<(), Error> {
    let file = &gwas_config.file;
    let gwas_reader =
        for_context(file, GwasReader::new(BufReader::new(for_file(file, File::open(file))?),
                        gwas_config.cols.clone().unwrap_or_default()))?;
    for gwas_record in gwas_reader {
        let GwasRecord { var_id, beta, se } = for_context(file, gwas_record)?;
        if let Some(id_data) = beta_se_by_id.get_mut(&var_id) {
            id_data.beta_se_list[i_trait] = BetaSe { beta, se };
        } else if action == Action::Classify {
            let mut beta_se_list = new_beta_se_list(n_traits);
            beta_se_list[i_trait] = BetaSe { beta, se };
            let weight = 1.0;
            let id_data = IdData { beta_se_list, weight };
            beta_se_by_id.insert(var_id, id_data);
        }
    }
    Ok(())
}

fn new_beta_se_list(n_traits: usize) -> Vec<BetaSe> {
    vec![BetaSe { beta: f64::NAN, se: f64::NAN }; n_traits]
}