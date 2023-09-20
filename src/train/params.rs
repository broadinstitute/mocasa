use std::fmt::{Display, Formatter};
use std::sync::Arc;
use crate::data::Meta;

pub(crate) struct Params {
    pub(crate) meta: Arc<Meta>,
    pub(crate) mu: f64,
    pub(crate) tau: f64,
    pub(crate) betas: Vec<f64>,
    pub(crate) sigmas: Vec<f64>,
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