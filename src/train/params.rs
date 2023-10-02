use std::fmt::{Display, Formatter};
use std::iter::once;
use std::sync::Arc;
use crate::data::Meta;
use crate::error::Error;

pub(crate) struct Params {
    pub(crate) meta: Arc<Meta>,
    pub(crate) mu: f64,
    pub(crate) tau: f64,
    pub(crate) betas: Vec<f64>,
    pub(crate) sigmas: Vec<f64>,
}

pub(crate) enum ParamIndex {
    Mu,
    Tau,
    Beta(usize),
    Sigma(usize),
}

impl ParamIndex {
    pub(crate) fn all(n_traits: usize) -> impl Iterator<Item=ParamIndex> {
        once(ParamIndex::Mu).chain(once(ParamIndex::Tau))
            .chain((0..n_traits).map(ParamIndex::Beta))
            .chain((0..n_traits).map(ParamIndex::Sigma))
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
