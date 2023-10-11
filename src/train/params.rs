use std::fmt::{Display, Formatter};
use std::iter::once;
use std::ops::Index;
use std::sync::Arc;
use crate::data::Meta;
use crate::error::Error;

#[derive(Clone)]
pub(crate) struct Params {
    pub(crate) meta: Arc<Meta>,
    pub(crate) mu: f64,
    pub(crate) tau: f64,
    pub(crate) betas: Vec<f64>,
    pub(crate) sigmas: Vec<f64>,
}

#[derive(Clone, Copy)]
pub(crate) enum ParamIndex {
    Mu,
    Tau,
    Beta(usize),
    Sigma(usize),
}

impl ParamIndex {
    pub(crate) fn all(n_traits: usize) -> impl Iterator<Item=ParamIndex> {
        [ParamIndex::Mu, ParamIndex::Tau].into_iter()
            .chain((0..n_traits).map(ParamIndex::Beta))
            .chain((0..n_traits).map(ParamIndex::Sigma))
    }
    pub(crate) fn to(index_max: ParamIndex, n_traits: usize) -> Box<dyn Iterator<Item=ParamIndex>> {
        match index_max {
            ParamIndex::Mu => { Box::new(once(ParamIndex::Mu)) }
            ParamIndex::Tau => { Box::new([ParamIndex::Mu, ParamIndex::Tau].into_iter()) }
            ParamIndex::Beta(i_trait_max) => {
                let iter =
                    [ParamIndex::Mu, ParamIndex::Tau].into_iter()
                        .chain((0..=i_trait_max).map(ParamIndex::Beta));
                Box::new(iter)
            }
            ParamIndex::Sigma(i_trait_max) => {
                let iter =
                    [ParamIndex::Mu, ParamIndex::Tau].into_iter()
                        .chain((0..n_traits).map(ParamIndex::Beta))
                        .chain((0..=i_trait_max).map(ParamIndex::Sigma));
                Box::new(iter)
            }
        }
    }
    pub(crate) fn n_params(n_traits: usize) -> usize { 2 * n_traits + 2 }
    pub(crate) fn from_ordinal(i_param: usize, n_traits: usize) -> Result<ParamIndex, Error> {
        match i_param {
            0 => { Ok(ParamIndex::Mu) }
            1 => { Ok(ParamIndex::Tau) }
            _ => {
                let i_trait = i_param - 2;
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
            ParamIndex::Tau => { 1 }
            ParamIndex::Beta(i_trait) => { i_trait + 2 }
            ParamIndex::Sigma(i_trait) => { i_trait + n_traits + 2 }
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
            let tau = values[1];
            let betas: Vec<f64> = values[2..(2 + n_traits)].to_vec();
            let sigmas: Vec<f64> = values[(2 + n_traits)..(2 + 2 * n_traits)].to_vec();
            Ok(Params { meta, mu, tau, betas, sigmas })
        }
    }
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
        writeln!(f, "tau = {}", self.tau)?;
        for ((name, beta), sigma) in self.meta.trait_names.iter()
            .zip(self.betas.iter()).zip(self.sigmas.iter()) {
            writeln!(f, "beta_{} = {}", name, beta)?;
            writeln!(f, "sigma_{} = {}", name, sigma)?;
        }
        Ok(())
    }
}
