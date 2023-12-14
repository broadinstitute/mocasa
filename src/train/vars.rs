use std::iter;
use crate::data::Metaphor;
use crate::math::matrix::Matrix;

pub(crate) struct Vars {
    pub(crate) metaphor: Metaphor,
    pub(crate) es: Vec<f64>,
    pub(crate) ts: Matrix,
}

pub(crate) enum VarIndex {
    E { i_data_point: usize },
    T { i_data_point: usize, i_trait: usize },
}

impl Vars {
    pub(crate) fn indices(&self) -> impl Iterator<Item=VarIndex> {
        let n_data_points = self.metaphor.n_data_points();
        let n_traits = self.metaphor.n_traits();
        (0..n_data_points).flat_map(move |i_data_point| {
            iter::once(VarIndex::E { i_data_point })
                .chain((0..n_traits).map(move |i_trait| {
                    VarIndex::T { i_data_point, i_trait }
                }))
        })
    }
}