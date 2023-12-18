use crate::data::{Meta, GwasData};
use crate::math::matrix::Matrix;
use crate::train::params::Params;
use crate::train::vars::Vars;

pub(crate) struct TrainModel {
    pub(crate) data: GwasData,
}

impl TrainModel {
    pub(crate) fn new(data: GwasData) -> TrainModel {
        TrainModel { data }
    }
    pub(crate) fn meta(&self) -> &Meta { &self.data.meta }
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
}

