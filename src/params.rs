pub(crate) mod transform;

use std::fmt::{Display, Formatter};
use std::fs::{File, read_to_string};
use std::io::BufWriter;
use std::ops::Index;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::{Error, for_file};
use std::io::Write;
use crate::math::matrix::Matrix;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Params {
    pub(crate) trait_names: Arc<Vec<String>>,
    pub(crate) mus: Vec<f64>,
    pub(crate) taus: Vec<f64>,
    pub(crate) betas: Matrix,
    pub(crate) sigmas: Vec<f64>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum ParamIndex {
    Mu(usize),
    Tau(usize),
    Beta(usize, usize),
    Sigma(usize),
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ParamsOverride {
    mu: Option<f64>,
    tau: Option<f64>,
}

impl ParamIndex {
    pub(crate) fn all(n_traits: usize) -> impl Iterator<Item=ParamIndex> {
        vec![ParamIndex::Mu, ParamIndex::Tau].into_iter()
            .chain((0..n_traits).map(ParamIndex::Beta))
            .chain((0..n_traits).map(ParamIndex::Sigma))
    }
    pub(crate) fn n_params(n_traits: usize) -> usize { 2 * n_traits + 2 }
    pub(crate) fn get_ordinal(&self, n_endos: usize, n_traits: usize) -> usize {
        match self {
            ParamIndex::Mu(&i_endo) => { i_endo }
            ParamIndex::Tau(&i_endo) => { n_endos + i_endo }
            ParamIndex::Beta(&i_endo, &i_trait) => { 2 * n_endos + i_endo * n_traits + i_trait }
            ParamIndex::Sigma(&i_trait) => { i_trait + n_endos * (2 + n_traits) }
        }
    }
    pub(crate) fn with_trait_name(&self, trait_names: &[String]) -> String {
        match self {
            ParamIndex::Mu(&i_endo) => { format!("mu_{}", i_endo) }
            ParamIndex::Tau(&i_endo) => { format!("mu_{}", i_endo) }
            ParamIndex::Beta(i_trait) => { format!("beta_{}", trait_names[*i_trait]) }
            ParamIndex::Sigma(i_trait) => { format!("sigma_{}", trait_names[*i_trait]) }
        }
    }
}

impl Params {
    pub(crate) fn from_vec(values: &[f64], trait_names: Arc<Vec<String>>, n_endos: usize)
                           -> Result<Params, Error> {
        let n_traits = trait_names.len();
        let n_values_needed = ParamIndex::n_params(n_traits);
        if values.len() != n_values_needed {
            Err(Error::from(format!("Need {} values for {} traits, but got {}.",
                                    n_values_needed, n_traits, values.len())))
        } else {
            let i_mu0: usize = 0;
            let i_tau0: usize = i_mu0 + n_endos;
            let i_beta00: usize = i_tau0 + n_endos;
            let i_sigma0: usize = i_beta00 + n_endos * n_traits;
            let mus = values[i_mu0..i_tau0].to_vec();
            let taus = values[i_tau0..i_beta00].to_vec();
            let betas_elements: Vec<f64> = values[i_beta00..i_sigma0].to_vec();
            let betas = Matrix::from_vec(n_endos, n_traits, betas_elements)?;
            let sigmas: Vec<f64> = values[i_sigma0..n_values_needed].to_vec();
            Ok(Params { trait_names, mus, taus, betas, sigmas })
        }
    }
    pub(crate) fn n_endos(&self) -> usize { self.betas.n_rows }
    pub(crate) fn n_traits(&self) -> usize { self.trait_names.len() }
    pub(crate) fn reduce_to(&self, trait_names: Arc<Vec<String>>, is_cols: &[usize]) -> Params {
        let mus = self.mus.clone();
        let taus = self.taus.clone();
        let betas = self.betas.only_cols(is_cols);
        let sigmas: Vec<f64> = is_cols.iter().map(|i_col| self.sigmas[*i_col]).collect();
        Params { trait_names, mus, taus, betas, sigmas }
    }
    pub(crate) fn plus_overwrite(self, overwrite: &ParamsOverride) -> Params {
        let Params {
            trait_names, mut mus, mut taus, betas,
            sigmas
        } = self;
        if let Some(mu_overwrite) = overwrite.mu {
            mus.fill(mu_overwrite)
        }
        if let Some(tau_overwrite) = overwrite.mu {
            taus.fill(tau_overwrite)
        }
        Params { trait_names, mus, taus, betas, sigmas }
    }
    pub(crate) fn normalized_with_mu_one(&self) -> Params {
        let trait_names = self.trait_names.clone();
        let n_endo = self.n_endos();
        let n_traits = self.n_traits();
        let mus: Vec<f64> = vec![1.0; n_endo];
        let taus: Vec<f64> =
            self.taus.iter().enumerate().map(|(i_endo, tau)| tau / self.mus[i_endo])
                .collect();
        let betas =
            Matrix::fill(n_endo, n_traits,
                         |i_endo, i_trait|
                             self.betas[i_endo][i_trait] * self.mus[i_endo]);
        let sigmas = self.sigmas.clone();
        Params { trait_names, mus, taus, betas, sigmas }
    }
}

impl Index<ParamIndex> for Params {
    type Output = f64;

    fn index(&self, index: ParamIndex) -> &Self::Output {
        match index {
            ParamIndex::Mu(&i_endo) => { &self.mu[i_endo] }
            ParamIndex::Tau(&i_endo) => { &self.tau[i_endo] }
            ParamIndex::Beta(&i_endo, &i_trait) => { &self.betas[i_endo][i_trait] }
            ParamIndex::Sigma(i_trait) => { &self.sigmas[i_trait] }
        }
    }
}

impl Display for Params {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "mu = {}", self.mu)?;
        for ((name, beta), sigma) in self.trait_names.iter()
            .zip(self.betas.iter()).zip(self.sigmas.iter()) {
            writeln!(f, "beta_{} = {}", name, beta)?;
            writeln!(f, "sigma_{} = {}", name, sigma)?;
        }
        Ok(())
    }
}

impl Display for ParamIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamIndex::Mu(i_endo) => { write!(f, "mu_{}", i_endo) }
            ParamIndex::Tau(i_endo) => { write!(f, "tau_{}", i_endo) }
            ParamIndex::Beta(i_endo, i_trait) => {
                write!(f, "beta_{}_{}", i_endo, i_trait)
            }
            ParamIndex::Sigma(i_trait) => { write!(f, "sigma_{}", i_trait) }
        }
    }
}

pub(crate) fn read_params_from_file(file: &str) -> Result<Params, Error> {
    let params_string = for_file(file, read_to_string(file))?;
    let params = serde_json::from_str(&params_string)?;
    Ok(params)
}

pub(crate) fn write_params_to_file(params: &Params, output_file: &str) -> Result<(), Error> {
    let mut writer =
        BufWriter::new(for_file(output_file, File::create(output_file))?);
    let json = serde_json::to_string(params)?;
    writeln!(writer, "{}", json)?;
    Ok(())
}
