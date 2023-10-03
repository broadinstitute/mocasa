use crate::train::model::TrainModel;
use crate::train::params::{ParamIndex, Params};
use crate::train::vars::Vars;

pub(crate) struct ParamEval {
    pub(crate) n_traits: usize,
    dmu: f64,
    dtau: f64,
    dmu_dmu: f64,
    dmu_dtau: f64,
    dtau_dtau: f64,
    dbetas: Vec<f64>,
    dsigmas: Vec<f64>,
    dbeta_dbetas: Vec<f64>,
    dbeta_dsigmas: Vec<f64>,
    dsigma_dsigmas: Vec<f64>,
}

impl ParamEval {
    pub(crate) fn new(model: &TrainModel, params: &Params, vars: &Vars) -> ParamEval {
        let n_traits = model.meta().n_traits();
        let n_data_points = model.meta().n_data_points();
        let mut dmu: f64 = 0.0;
        let mut dtau: f64 = 0.0;
        let mut dmu_dmu: f64 = 0.0;
        let mut dmu_dtau: f64 = 0.0;
        let mut dtau_dtau: f64 = 0.0;
        let mut dbetas: Vec<f64> = vec![0.0; n_traits];
        let mut dsigmas: Vec<f64> = vec![0.0; n_traits];
        let mut dbeta_dbetas: Vec<f64> = vec![0.0; n_traits];
        let mut dbeta_dsigmas: Vec<f64> = vec![0.0; n_traits];
        let mut dsigma_dsigmas: Vec<f64> = vec![0.0; n_traits];
        for i_data_point in 0..n_data_points {
            let e = vars.es[i_data_point];
            let e_mu = e - params.mu;
            let e_mu2 = e_mu.powi(2);
            let tau = params.tau;
            let tau2 = tau.powi(2);
            dmu += e_mu / tau2;
            dtau += e_mu2 / tau.powi(3);
            dmu_dmu += (e_mu2 - tau2) / tau.powi(4);
            dmu_dtau += e_mu * (e_mu2 - 2.0 * tau2) / tau.powi(5);
            dtau_dtau += e_mu2 * (e_mu2 - 3.0 * tau2) / tau.powi(6);
            for (i_trait, ((((dbeta, dsigma), dbeta_dbeta),
                dbeta_dsigma), dsigma_dsigma))
            in dbetas.iter_mut().zip(dsigmas.iter_mut()).zip(dbeta_dbetas.iter_mut())
                .zip(dbeta_dsigmas.iter_mut()).zip(dsigma_dsigmas.iter_mut())
                .enumerate() {
                let t_be = vars.ts[i_data_point][i_trait] - params.betas[i_trait] * e;
                let t_be2 = t_be.powi(2);
                let s = params.sigmas[i_trait];
                let s2 = s.powi(2);
                *dbeta += t_be * e / s2;
                *dsigma += t_be2 / s.powi(3);
                *dbeta_dbeta += (t_be2 - s2) * e.powi(2) / s.powi(4);
                *dbeta_dsigma += t_be * (t_be2 - 2.0 * s2) * e / s.powi(5);
                *dsigma_dsigma += t_be2 * (t_be2 - 3.0 * s2) / s.powi(6);
            }
        }
        ParamEval {
            n_traits,
            dmu,
            dtau,
            dmu_dmu,
            dmu_dtau,
            dtau_dtau,
            dbetas,
            dsigmas,
            dbeta_dbetas,
            dbeta_dsigmas,
            dsigma_dsigmas,
        }
    }
    pub(crate) fn gradient(&self, index: ParamIndex) -> f64 {
        match index {
            ParamIndex::Mu => { self.dmu }
            ParamIndex::Tau => { self.dtau }
            ParamIndex::Beta(i_trait) => { self.dbetas[i_trait] }
            ParamIndex::Sigma(i_trait) => { self.dsigmas[i_trait] }
        }
    }
    pub(crate) fn hessian(&self, index1: ParamIndex, index2: ParamIndex) -> f64 {
        let (index_min, index_max) =
            if index1.get_ordinal(self.n_traits) <= index2.get_ordinal(self.n_traits) {
                (index1, index2)
            } else {
                (index2, index1)
            };
        match (index_min, index_max) {
            (ParamIndex::Mu, ParamIndex::Mu) => { self.dmu_dmu }
            (ParamIndex::Mu, ParamIndex::Tau) => { self.dmu_dtau }
            (ParamIndex::Tau, ParamIndex::Tau) => { self.dtau_dtau }
            (ParamIndex::Mu, ParamIndex::Beta(i_trait)) => {
                self.dmu * self.dbetas[i_trait]
            }
            (ParamIndex::Tau, ParamIndex::Beta(i_trait)) => {
                self.dtau * self.dbetas[i_trait]
            }
            (ParamIndex::Beta(i_trait1), ParamIndex::Beta(i_trait2)) => {
                if i_trait1 == i_trait2 {
                    self.dbeta_dbetas[i_trait1]
                } else {
                    self.dbetas[i_trait1] * self.dbetas[i_trait2]
                }
            }
            (ParamIndex::Mu, ParamIndex::Sigma(i_trait)) => {
                self.dmu * self.dsigmas[i_trait]
            }
            (ParamIndex::Tau, ParamIndex::Sigma(i_trait)) => {
                self.dtau * self.dsigmas[i_trait]
            }
            (ParamIndex::Beta(i_trait_beta), ParamIndex::Sigma(i_trait_sigma)) => {
                if i_trait_beta == i_trait_sigma {
                    self.dbeta_dsigmas[i_trait_beta]
                } else {
                    self.dbetas[i_trait_beta] * self.dsigmas[i_trait_sigma]
                }
            }
            (ParamIndex::Sigma(i_trait1), ParamIndex::Sigma(i_trait2)) => {
                if i_trait1 == i_trait2 {
                    self.dsigma_dsigmas[i_trait1]
                } else {
                    self.dsigmas[i_trait1] * self.dsigmas[i_trait2]
                }
            }
            (_, _) => { unreachable!() }
        }
    }
}