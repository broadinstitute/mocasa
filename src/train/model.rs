use std::fmt::{Display, Formatter};
use std::sync::Arc;
use crate::data::{Meta, TrainData};
use crate::math::matrix::Matrix;

pub(crate) struct TrainModel {
    data: TrainData,
}

pub(crate) struct Params {
    meta: Arc<Meta>,
    mu: f64,
    tau: f64,
    betas: Vec<f64>,
    sigmas: Vec<f64>,
}

pub(crate) struct Vars {
    es: Vec<f64>,
    ts: Matrix,
}

impl TrainModel {
    pub(crate) fn new(data: TrainData) -> TrainModel {
        TrainModel { data }
    }
    pub(crate) fn initial_params(&self) -> Params {
        let meta = self.data.meta.clone();
        let mu = 0.0;
        let tau = 1.0;
        let betas = vec![1.0; self.data.n_traits()];
        let sigmas = vec![1.0; self.data.n_traits()];
        Params { meta, mu, tau, betas, sigmas }
    }
    pub(crate) fn initial_vars(&self, params: &Params) -> Vars {
        let es = vec![params.mu; self.data.n_data_points()];
        let element_gen = |i_data_point: usize, i_trait: usize| {
            es[i_data_point] * params.betas[i_trait]
        };
        let ts =
            Matrix::fill(self.data.n_data_points(), self.data.n_traits(),
                         element_gen);
        Vars { es, ts }
    }
}

impl Display for Params {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}