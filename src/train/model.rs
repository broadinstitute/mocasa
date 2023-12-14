use crate::data::{Metaphor, TrainData};
use crate::math::matrix::Matrix;
use crate::train::params::Params;
use crate::train::vars::Vars;

pub(crate) struct TrainModel {
    pub(crate) data: TrainData,
}

impl TrainModel {
    pub(crate) fn new(data: TrainData) -> TrainModel {
        TrainModel { data }
    }
    pub(crate) fn metaphor(&self) -> &Metaphor { &self.data.metaphor }
    pub(crate) fn initial_vars(&self, params: &Params) -> Vars {
        let metaphor = self.data.metaphor.clone();
        let es = vec![params.mu; self.data.n_data_points()];
        let element_gen = |i_data_point: usize, i_trait: usize| {
            es[i_data_point] * params.betas[i_trait]
        };
        let ts =
            Matrix::fill(self.data.n_data_points(), self.data.n_traits(),
                         element_gen);
        Vars { metaphor, es, ts }
    }
}

