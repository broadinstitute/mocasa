use std::fmt::{Display, Formatter};
use std::iter::once;
use std::ops::Index;
use std::sync::Arc;
use crate::data::Meta;
use crate::error::Error;

pub(crate) const TAU: f64 = 1.0;

#[derive(Clone)]
pub(crate) struct Params {
    pub(crate) meta: Arc<Meta>,
    pub(crate) mu: f64,
    pub(crate) betas: Vec<f64>,
    pub(crate) sigmas: Vec<f64>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum ParamIndex {
    Mu,
    Beta(usize),
    Sigma(usize),
}

impl ParamIndex {
    pub(crate) fn all(n_traits: usize) -> impl Iterator<Item=ParamIndex> {
        once(ParamIndex::Mu)
            .chain((0..n_traits).map(ParamIndex::Beta))
            .chain((0..n_traits).map(ParamIndex::Sigma))
    }
    pub(crate) fn to(index_max: ParamIndex, n_traits: usize) -> Box<dyn Iterator<Item=ParamIndex>> {
        match index_max {
            ParamIndex::Mu => { Box::new(once(ParamIndex::Mu)) }
            ParamIndex::Beta(i_trait_max) => {
                let iter =
                    once(ParamIndex::Mu)
                        .chain((0..=i_trait_max).map(ParamIndex::Beta));
                Box::new(iter)
            }
            ParamIndex::Sigma(i_trait_max) => {
                let iter =
                    once(ParamIndex::Mu)
                        .chain((0..n_traits).map(ParamIndex::Beta))
                        .chain((0..=i_trait_max).map(ParamIndex::Sigma));
                Box::new(iter)
            }
        }
    }
    pub(crate) fn n_params(n_traits: usize) -> usize { 2 * n_traits + 1 }
    pub(crate) fn from_ordinal(i_param: usize, n_traits: usize) -> Result<ParamIndex, Error> {
        match i_param {
            0 => { Ok(ParamIndex::Mu) }
            _ => {
                let i_trait = i_param - 1;
                if i_trait < n_traits {
                    Ok(ParamIndex::Beta(i_trait))
                } else {
                    let i_trait = i_trait - n_traits;
                    if i_trait < n_traits {
                        Ok(ParamIndex::Sigma(i_trait))
                    } else {
                        Err(Error::from(
                            format!(
                                "Found index {}, but for {} traits, there are only {} params.",
                                i_param, n_traits, ParamIndex::n_params(n_traits))))
                    }
                }
            }
        }
    }
    pub(crate) fn get_ordinal(&self, n_traits: usize) -> usize {
        match self {
            ParamIndex::Mu => { 0 }
            ParamIndex::Beta(i_trait) => { i_trait + 1 }
            ParamIndex::Sigma(i_trait) => { i_trait + n_traits + 1 }
        }
    }
    pub(crate) fn with_trait_name(&self, trait_names: &[String]) -> String {
        match self {
            ParamIndex::Mu => { "mu".to_string() }
            ParamIndex::Beta(i_trait) => { format!("beta_{}", trait_names[*i_trait]) }
            ParamIndex::Sigma(i_trait) => { format!("sigma_{}", trait_names[*i_trait]) }
        }
    }
}

impl Params {
    pub(crate) fn from_vec(values: &[f64], meta: &Arc<Meta>) -> Result<Params, Error> {
        let n_traits = meta.n_traits();
        let n_values_needed = ParamIndex::n_params(n_traits);
        if values.len() != n_values_needed {
            Err(Error::from(format!("Need {} values for {} traits, but got {}.",
                                    n_values_needed, n_traits, values.len())))
        } else {
            let meta = meta.clone();
            let mu = values[0];
            let betas: Vec<f64> = values[1..(2 + n_traits)].to_vec();
            let sigmas: Vec<f64> = values[(1 + n_traits)..(1 + 2 * n_traits)].to_vec();
            Ok(Params { meta, mu, betas, sigmas })
        }
    }
    pub(crate) fn invalid_indices(&self) -> Vec<ParamIndex> {
        let mut invalid_indices: Vec<ParamIndex> = Vec::new();
        for (i_trait, sigma) in self.sigmas.iter().enumerate() {
            if *sigma <= 0.0 {
                invalid_indices.push(ParamIndex::Sigma(i_trait))
            }
        }
        invalid_indices
    }
}

impl Index<ParamIndex> for Params {
    type Output = f64;

    fn index(&self, index: ParamIndex) -> &Self::Output {
        match index {
            ParamIndex::Mu => { &self.mu }
            ParamIndex::Beta(i_trait) => { &self.betas[i_trait] }
            ParamIndex::Sigma(i_trait) => { &self.sigmas[i_trait] }
        }
    }
}

impl Display for Params {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "mu = {}", self.mu)?;
        for ((name, beta), sigma) in self.meta.trait_names.iter()
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
