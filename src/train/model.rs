use crate::data::TrainData;
use crate::math::matrix::Matrix;
use crate::train::param_stats::ParamDiffs;
use crate::train::params::Params;
use crate::train::vars::Vars;

pub(crate) struct TrainModel {
    data: TrainData,
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
        let meta = self.data.meta.clone();
        let es = vec![params.mu; self.data.n_data_points()];
        let element_gen = |i_data_point: usize, i_trait: usize| {
            es[i_data_point] * params.betas[i_trait]
        };
        let ts =
            Matrix::fill(self.data.n_data_points(), self.data.n_traits(),
                         element_gen);
        Vars { meta, es, ts }
    }
    pub(crate) fn evaluate_params(&self, params: &Params, vars: &Vars) -> ParamDiffs {
        todo!()
    }
}
