use crate::data::{GwasData, Meta};
use crate::math::matrix::Matrix;
use crate::params::Params;

pub(crate) struct Vars {
    pub(crate) meta: Meta,
    pub(crate) es: Matrix,
    pub(crate) ts: Matrix,
}

pub(crate) enum VarIndex {
    E { i_data_point: usize, i_endo: usize },
    T { i_data_point: usize, i_trait: usize },
}

impl Vars {
    pub(crate) fn indices(&self) -> impl Iterator<Item=VarIndex> {
        let n_data_points = self.meta.n_data_points();
        let n_traits = self.meta.n_traits();
        let n_endos = self.meta.n_endos();
        (0..n_data_points).flat_map(move |i_data_point| {
            (0..n_endos).map(move |i_endo| VarIndex::E { i_data_point, i_endo })
                .chain((0..n_traits).map(move |i_trait| {
                    VarIndex::T { i_data_point, i_trait }
                }))
        })
    }
    pub(crate) fn initial_vars(data: &GwasData, params: &Params) -> Vars {
        let meta = data.meta.clone();
        let n_data_points = data.n_data_points();
        let n_endos = params.n_endos();
        let es =
            Matrix::fill(n_data_points, n_endos,
                         |_, i_endo| params.mus[i_endo]);
        let ts =
            Matrix::fill(data.n_data_points(), data.n_traits(),
                         |i_data_point: usize, i_trait: usize|
                             {
                                 (0..n_endos).map(|i_endo|
                                     es[i_data_point][i_endo] * params.betas[i_endo][i_trait])
                                     .sum::<f64>()
                             });
        Vars { meta, es, ts }
    }
    pub(crate) fn variable_names(meta: &Meta) -> impl Iterator<Item=String> + '_ {
        let e_iter =
            (0..meta.n_endos()).map(move |i_endo| format!("E_{}", i_endo));
        let t_iter =
            meta.trait_names().iter().map(|trait_name| format!("T_{}", trait_name));
        e_iter.chain(t_iter)
    }
}