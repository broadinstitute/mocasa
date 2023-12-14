use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::Error;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Params {
    pub(crate) trait_names: Arc<Vec<String>>,
    pub(crate) mu: f64,
    pub(crate) tau: f64,
    pub(crate) betas: Vec<f64>,
    pub(crate) sigmas: Vec<f64>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum ParamIndex {
    Mu,
    Tau,
    Beta(usize),
    Sigma(usize),
}

impl ParamIndex {
    pub(crate) fn all(n_traits: usize) -> impl Iterator<Item=ParamIndex> {
        vec![ParamIndex::Mu, ParamIndex::Tau].into_iter()
            .chain((0..n_traits).map(ParamIndex::Beta))
            .chain((0..n_traits).map(ParamIndex::Sigma))
    }
    pub(crate) fn n_params(n_traits: usize) -> usize { 2 * n_traits + 2 }
    pub(crate) fn get_ordinal(&self, n_traits: usize) -> usize {
        match self {
            ParamIndex::Mu => { 0 }
            ParamIndex::Tau => { 1 }
            ParamIndex::Beta(i_trait) => { i_trait + 2 }
            ParamIndex::Sigma(i_trait) => { i_trait + n_traits + 2 }
        }
    }
    pub(crate) fn with_trait_name(&self, trait_names: &[String]) -> String {
        match self {
            ParamIndex::Mu => { "mu".to_string() }
            ParamIndex::Tau => { "tau".to_string() }
            ParamIndex::Beta(i_trait) => { format!("beta_{}", trait_names[*i_trait]) }
            ParamIndex::Sigma(i_trait) => { format!("sigma_{}", trait_names[*i_trait]) }
        }
    }
}

impl Params {
    pub(crate) fn from_vec(values: &[f64], trait_names: Arc<Vec<String>>)
                           -> Result<Params, Error> {
        let n_traits = trait_names.len();
        let n_values_needed = ParamIndex::n_params(n_traits);
        if values.len() != n_values_needed {
            Err(Error::from(format!("Need {} values for {} traits, but got {}.",
                                    n_values_needed, n_traits, values.len())))
        } else {
            let mu = values[0];
            let tau = values[1];
            let betas: Vec<f64> = values[2..(2 + n_traits)].to_vec();
            let sigmas: Vec<f64> = values[(2 + n_traits)..(2 + 2 * n_traits)].to_vec();
            Ok(Params { trait_names, mu, tau, betas, sigmas })
        }
    }
    pub(crate) fn n_traits(&self) -> usize { self.trait_names.len() }
}

impl Index<ParamIndex> for Params {
    type Output = f64;

    fn index(&self, index: ParamIndex) -> &Self::Output {
        match index {
            ParamIndex::Mu => { &self.mu }
            ParamIndex::Tau => { &self.tau }
            ParamIndex::Beta(i_trait) => { &self.betas[i_trait] }
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
            ParamIndex::Mu => { write!(f, "mu") }
            ParamIndex::Tau => { write!(f, "tau") }
            ParamIndex::Beta(i_trait) => { write!(f, "beta_{}", i_trait) }
            ParamIndex::Sigma(i_trait) => { write!(f, "sigma_{}", i_trait) }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::train::params::ParamIndex;

    #[test]
    fn indices() {
        const N_PARAMS_TEST: usize = 5;
        for (i_param, index) in ParamIndex::all(N_PARAMS_TEST).enumerate() {
            let i_param2 = index.get_ordinal(N_PARAMS_TEST);
            let index2 =
                ParamIndex::from_ordinal(i_param2, N_PARAMS_TEST).unwrap();
            assert_eq!(i_param, i_param2);
            assert_eq!(index, index2);
        }
    }
}
