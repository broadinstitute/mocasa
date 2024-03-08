use std::fmt::{Display, Formatter};
use std::sync::Arc;
use crate::error::Error;
use crate::math::stats::Stats;
use crate::math::trident::TridentStats;
use crate::params::{ParamIndex, Params};

pub(crate) struct ParamMetaStats {
    trait_names: Arc<Vec<String>>,
    stats: Vec<Vec<TridentStats>>,
}

pub(crate) struct Summary {
    pub(crate) n_chains_used: usize,
    pub(crate) params: Params,
    pub(crate) intra_chain_vars: Vec<f64>,
    pub(crate) inter_chain_vars: Vec<f64>,
    pub(crate) inter_intra_ratios: Vec<f64>,
    pub(crate) relative_errors: Vec<f64>,
    pub(crate) inter_intra_ratios_mean: f64,
    pub(crate) relative_errors_mean: f64,
}

impl ParamMetaStats {
    pub(crate) fn new(n_chains_used: usize, trait_names: Arc<Vec<String>>, params0: &[Params],
                      params1: &[Params])
                      -> ParamMetaStats {
        let n_traits = trait_names.len();
        let stats = (0..n_chains_used).map(|i_chain| {
            ParamIndex::all(n_traits).map(|index| {
                let param0 = params0[i_chain][index];
                let param1 = params1[i_chain][index];
                TridentStats::new(param0, param1)
            }).collect::<Vec<TridentStats>>()
        }).collect::<Vec<Vec<TridentStats>>>();
        ParamMetaStats { trait_names, stats }
    }
    pub(crate) fn n_traits(&self) -> usize { self.trait_names.len() }
    pub(crate) fn n_chains_used(&self) -> usize { self.stats.len() }
    pub(crate) fn add(&mut self, params: &[Params]) {
        let n_traits = self.n_traits();
        for (i_chain, param) in params.iter().enumerate() {
            for index in ParamIndex::all(n_traits) {
                let i_param = index.get_ordinal(n_traits);
                self.stats[i_chain][i_param].add(param[index])
            }
        }
    }
    pub(crate) fn summary(&self) -> Result<Summary, Error> {
        let n_traits = self.n_traits();
        let n_chains_used = self.n_chains_used();
        let n_params = ParamIndex::n_params(n_traits);
        let mut param_values: Vec<f64> = Vec::with_capacity(n_params);
        let mut intra_chain_vars: Vec<f64> = Vec::with_capacity(n_params);
        let mut inter_chain_vars: Vec<f64> = Vec::with_capacity(n_params);
        let mut inter_intra_ratios: Vec<f64> = Vec::with_capacity(n_params);
        let mut relative_errors: Vec<f64> = Vec::with_capacity(n_params);
        for index in ParamIndex::all(n_traits) {
            let i_param = index.get_ordinal(n_traits);
            let mut inter_mean_stats = Stats::new();
            let mut inter_var_stats = Stats::new();
            for i_chain in 0..n_chains_used {
                let stats = &self.stats[i_chain][i_param];
                inter_mean_stats.add(stats.mean());
                inter_var_stats.add(stats.variance());
            }
            let param_value = unwrap_or_not_enough_data(inter_mean_stats.mean())?;
            let intra_chain_var = unwrap_or_not_enough_data(inter_var_stats.mean())?;
            let inter_chain_var = unwrap_or_not_enough_data(inter_mean_stats.variance())?;
            let inter_intra_ratio = inter_chain_var / intra_chain_var;
            let relative_error =
                (intra_chain_var.sqrt() / param_value.abs()) / (n_chains_used as f64).sqrt();
            param_values.push(param_value);
            intra_chain_vars.push(intra_chain_var);
            inter_chain_vars.push(inter_chain_var);
            inter_intra_ratios.push(inter_intra_ratio);
            relative_errors.push(relative_error);
        }
        let params = Params::from_vec(&param_values, self.trait_names.clone())?;
        let inter_intra_ratios_mean =
            inter_intra_ratios.iter().sum::<f64>() / (n_params as f64);
        let relative_errors_mean =
            relative_errors.iter().sum::<f64>() / (n_params as f64);
        Ok(Summary {
            n_chains_used,
            params,
            intra_chain_vars,
            inter_chain_vars,
            inter_intra_ratios,
            relative_errors,
            inter_intra_ratios_mean,
            relative_errors_mean,
        })
    }
}

fn unwrap_or_not_enough_data(value: Option<f64>) -> Result<f64, Error> {
    value.ok_or_else(|| Error::from("Not enough data"))
}

fn str18<T: Display>(item: T) -> String {
    format!("{}                  ", item)[0..18].to_string()
}

impl Display for Summary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let n_traits = self.params.n_traits();
        writeln!(f, "Chains used: {}", self.n_chains_used)?;
        writeln!(f, "Relative errors mean: {}", self.relative_errors_mean)?;
        writeln!(f, "Inter/intra ratios mean: {}", self.inter_intra_ratios_mean.sqrt())?;
        writeln!(f, "{} {} {} {} {} {}",
                 str18("param"), str18("value"), str18("rel.err."),
                 str18("inter_chains"), str18("intra_chains"), str18("ratio"))?;
        for (i, index) in ParamIndex::all(n_traits).enumerate() {
            let param = self.params[index];
            let rel_err = self.relative_errors[i];
            let inter_chain_std_dev = self.inter_chain_vars[i].sqrt();
            let intra_chain_std_dev = self.intra_chain_vars[i].sqrt();
            let ratio = self.inter_intra_ratios[i];
            writeln!(f, "{} {} {} {} {} {}",
                     str18(index.with_trait_name(&self.params.trait_names)),
                     str18(param), str18(rel_err), str18(inter_chain_std_dev),
                     str18(intra_chain_std_dev), str18(ratio))?
        }
        Ok(())
    }
}