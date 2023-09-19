use crate::data::TrainData;
use crate::math::matrix::Matrix;

pub(crate) struct TrainModel {
    data: TrainData,
}

pub(crate) struct Params {
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
        let mu = 0.0;
        let tau = 1.0;
        let betas = vec![1.0; self.data.n_traits()];
        let sigmas = vec![1.0; self.data.n_traits()];
        Params { mu, tau, betas, sigmas }
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