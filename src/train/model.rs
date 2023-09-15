use crate::data::TrainData;

pub(crate) struct TrainModel {
    data: TrainData
}

pub(crate) struct Params {
    mu: f64,
    tau: f64,
    betas: Vec<f64>,
    sigmas: Vec<f64>
}

impl TrainModel {
    pub(crate) fn new(data: TrainData) -> TrainModel {
        TrainModel { data }
    }
    pub(crate) fn initial_params(&self) -> Params {
        let mu = 0.0;
        let tau = 1.0;
        let betas = vec![1.0; self.data.len()];
        let sigmas = vec![1.0; self.data.len()];
        Params { mu, tau, betas, sigmas }
    }
}