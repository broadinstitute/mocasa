use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use log::{error, warn};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::classify::{Classification, MessageToCentral, MessageToWorker};
use crate::data::{GwasData, Meta};
use crate::options::config::{ClassifyConfig, SharedConfig};
use crate::sample::vars::Vars;
use crate::params::Params;
use crate::sample::sampler::{Tracer, Sampler};
use crate::classify::exact::calculate_mu;
use crate::error::Error;

struct ClassifyTracer {
    e_writers: Vec<Result<BufWriter<File>, Error>>,
    t_writers: Vec<Result<BufWriter<File>, Error>>,
}

impl ClassifyTracer {
    fn new(meta: &Meta, out_file_name: &str, var_id: &str, n_endos: usize) -> ClassifyTracer {
        let e_writers = (0..n_endos).map(|i_endo| {
            let file_name = format!("{}_{}_trace_e_{}", out_file_name, var_id, i_endo);
            try_writer(&file_name)
        }).collect::<Vec<_>>();
        let t_writers = (0..meta.n_traits()).map(|i_trait| {
            let trait_name = &meta.trait_names[i_trait];
            let file_name = format!("{}_{}_trace_t_{}", out_file_name, var_id, trait_name);
            try_writer(&file_name)
        }).collect::<Vec<_>>();
        ClassifyTracer { e_writers, t_writers }
    }
}

fn try_writer(file_name: &str) -> Result<BufWriter<File>, Error> {
    match File::create(file_name) {
        Ok(file) => { Ok(BufWriter::new(file)) }
        Err(error) => { Err(Error::from(error)) }
    }
}

impl Tracer for ClassifyTracer {
    fn trace_e(&mut self, i_endo: usize, e: f64) {
        match self.e_writers[i_endo] {
            Ok(ref mut writer) => {
                if let Err(error) = writeln!(writer, "{}", e) {
                    warn!("Could not write E trace: {}", error)
                }
            }
            Err(ref error) => {
                warn!("Could not write E trace: {}", error)
            }
        }
    }

    fn trace_t(&mut self, i_trait: usize, t: f64) {
        match self.t_writers[i_trait] {
            Ok(ref mut writer) => {
                if let Err(error) = writeln!(writer, "{}", t) {
                    warn!("Could not write T trace: {}", error)
                }
            }
            Err(ref error) => {
                warn!("Could not write T trace: {}", error)
            }
        }
    }
}

pub(crate) fn classify_worker(data: &Arc<GwasData>, params: &Params, config: ClassifyConfig,
                              config_shared: SharedConfig, sender: Sender<MessageToCentral>,
                              receiver: Receiver<MessageToWorker>, i_thread: usize) {
    loop {
        let in_message = receiver.recv().unwrap();
        match in_message {
            MessageToWorker::DataPoint(i_data_point) => {
                let (data, is_col) = data.only_data_point(i_data_point);
                let trait_names = data.meta.trait_names.clone();
                if is_col.len() < trait_names.len() {
                    let id = &data.meta.var_ids()[0];
                    warn!("For {}, we have only data for {} of the {} traits.", id, is_col.len(),
                        trait_names.len())
                } else {
                    for (i_i_col, i_col) in is_col.iter().enumerate() {
                        if i_i_col != *i_col {
                            let id = &data.meta.var_ids()[0];
                            let col_str =
                                is_col.iter().map(|i| i.to_string())
                                    .collect::<Vec<_>>().join(", ");
                            error!("For {id}, column indices are messed up {col_str}.")
                        }
                    }
                }
                let params = params.reduce_to(trait_names, &is_col);
                let mut vars = Vars::initial_vars(&data, &params);
                let rng = thread_rng();
                let meta = data.meta.clone();
                let mut sampler = Sampler::<ThreadRng>::new(&meta, rng);
                let mut tracer =
                    match (&config.trace_ids, data.meta.var_ids.first()) {
                        (Some(trace_ids), Some(var_id))
                        if trace_ids.contains(var_id)
                        => {
                            let tracer =
                                ClassifyTracer::new(&meta, &config.out_file, var_id,
                                                    meta.n_endos());
                            Some(Box::new(tracer) as Box<dyn Tracer>)
                        }
                        _ => { None }
                    };
                sampler.sample_n(&data, &params, &mut vars, config_shared.n_steps_burn_in,
                                 &mut tracer);
                sampler.sample_n(&data, &params, &mut vars, config.n_samples, &mut tracer);
                let sampled_classification =
                    sampler.var_stats().calculate_classification();
                let e_mean_calculated =
                    match calculate_mu(&params, &data.betas[0], &data.ses[0]) {
                        Ok(mu) => {mu}
                        Err(error) => {
                            warn!("{}", error);
                            f64::NAN
                        }
                    };
                let classification =
                    Classification { i_data_point, sampled_classification, e_mean_calculated };
                sender.send(MessageToCentral { i_thread, classification }).unwrap();
            }
            MessageToWorker::Shutdown => {
                break;
            }
        }
    }
}