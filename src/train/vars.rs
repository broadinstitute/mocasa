use std::iter;
use crate::data::{GwasData, Meta};
use crate::math::matrix::Matrix;
use crate::train::params::Params;

pub(crate) struct Vars {
    pub(crate) meta: Meta,
    pub(crate) es: Vec<f64>,
    pub(crate) ts: Matrix,
}

pub(crate) enum VarIndex {
    E { i_data_point: usize },
    T { i_data_point: usize, i_trait: usize },
}

impl Vars {
    pub(crate) fn indices(&self) -> impl Iterator<Item=VarIndex> {
        let n_data_points = self.meta.n_data_points();
        let n_traits = self.meta.n_traits();
        (0..n_data_points).flat_map(move |i_data_point| {
            iter::once(VarIndex::E { i_data_point })
                .chain((0..n_traits).map(move |i_trait| {
                    VarIndex::T { i_data_point, i_trait }
                }))
        })
    }
    pub(crate) fn initial_vars(data: &GwasData, params: &Params) -> Vars {
        let meta = data.meta.clone();
        let es = vec![params.mu; data.n_data_points()];
        let element_gen = |i_data_point: usize, i_trait: usize| {
            es[i_data_point] * params.betas[i_trait]
        };
        let ts =
            Matrix::fill(data.n_data_points(), data.n_traits(), element_gen);
        Vars { meta, es, ts }
    }
}