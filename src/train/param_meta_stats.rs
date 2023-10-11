use std::sync::Arc;
use crate::data::Meta;
use crate::error::Error;
use crate::math::stats::Stats;
use crate::train::params::{ParamIndex, Params};

pub(crate) struct ParamMetaStats {
    meta: Arc<Meta>,
    stats: Vec<Stats>
}

pub(crate) struct Summary {
    pub(crate) params: Params,
    pub(crate) params_old: Params,
    pub(crate) intra_chains: Vec<f64>,
    pub(crate) intra_steps: Vec<f64>,
    pub(crate) intra_chains_mean: f64,
    pub(crate) intra_steps_mean: f64
}

fn unwrap_or_not_enough_data(value: Option<f64>) -> Result<f64, Error> {
    value.ok_or_else(|| Error::from("Not enough data"))
}

impl ParamMetaStats {
    pub(crate) fn new(meta: Arc<Meta>) -> ParamMetaStats {
        let n_params = ParamIndex::n_params(meta.n_traits());
        let stats =  vec![Stats::new(); n_params];
        ParamMetaStats { meta, stats }
    }
    pub(crate) fn add(&mut self, params: Params) {
        let n_traits = self.meta.n_traits();
        for i in ParamIndex::all(n_traits) {
            self.stats[i.get_ordinal(n_traits)].add(params[i])
        }
    }
    pub(crate) fn summary(&self, params_old: Params) -> Result<Summary, Error> {
        let n_traits = self.meta.n_traits();
        let n_params = ParamIndex::n_params(n_traits);
        let mut values: Vec<f64> = Vec::with_capacity(n_params);
        let mut intra_chains: Vec<f64> = Vec::with_capacity(n_params);
        let mut intra_steps: Vec<f64> = Vec::with_capacity(n_params);
        for index in ParamIndex::all(n_traits) {
            let i_param = index.get_ordinal(n_traits);
            let stats = &self.stats[i_param];
            let mean = unwrap_or_not_enough_data(stats.mean())?;
            let var = unwrap_or_not_enough_data(stats.variance())?;
            let std_dev = var.sqrt();
            values.push(mean);
            intra_chains.push(std_dev / mean.abs());
            intra_steps.push((params_old[index] - mean).abs() /  mean.abs());
        }
        let params = Params::from_vec(&values, &self.meta)?;
        let intra_chains_mean = intra_chains.iter().sum::<f64>() / (n_traits as f64);
        let intra_steps_mean = intra_steps.iter().sum::<f64>() / (n_traits as f64);
        Ok(Summary {
            params, params_old, intra_chains, intra_steps, intra_chains_mean, intra_steps_mean
        })
    }
}