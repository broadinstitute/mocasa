use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use log::{error, warn};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::classify::{Classification, MessageToCentral, MessageToWorker};
use crate::data::GwasData;
use crate::options::config::{ClassifyConfig, SharedConfig};
use crate::sample::vars::Vars;
use crate::params::Params;
use crate::sample::sampler::{ETracer, Sampler};
use crate::classify::exact::calculate_mu;

struct ClassifyETracer<W: Write> {
    writer: W,
}

impl<W: Write> ETracer for ClassifyETracer<W> {
    fn trace_e(&mut self, e: f64) {
        if let Err(error) = writeln!(self.writer, "{}", e) {
            warn!("Could not write E trace: {}", error)
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
                    let id = &data.meta.var_ids()[i_data_point];
                    warn!("For {}, we have only data for {} of the {} traits.", id, is_col.len(),
                        trait_names.len())
                } else {
                    for (i_i_col, i_col) in is_col.iter().enumerate() {
                        if i_i_col != *i_col {
                            let id = &data.meta.var_ids()[i_data_point];
                            error!("For {id}, column indices are messed up.")
                        }
                    }
                }
                let params = params.reduce_to(trait_names, &is_col);
                let mut vars = Vars::initial_vars(&data, &params);
                let rng = thread_rng();
                let meta = data.meta.clone();
                let mut sampler = Sampler::<ThreadRng>::new(&meta, rng);
                let mut e_tracer =
                    match (&config.trace_ids, data.meta.var_ids.first()) {
                        (Some(trace_ids), Some(var_id))
                        if trace_ids.contains(var_id)
                        => {
                            let trace_file_name = {
                                let mut temp = config.out_file.clone();
                                temp.push('_');
                                temp.push_str(var_id);
                                temp
                            };
                            match File::create(trace_file_name) {
                                Ok(file) => {
                                    let writer = BufWriter::new(file);
                                    let e_tracer = ClassifyETracer { writer };
                                    Some(Box::new(e_tracer) as Box<dyn ETracer>)
                                }
                                Err(error) => {
                                    warn!("Could not create E trace file: {}", error);
                                    None
                                }
                            }
                        }
                        _ => { None }
                    };
                sampler.sample_n(&data, &params, &mut vars, config_shared.n_steps_burn_in,
                                 &mut e_tracer);
                sampler.sample_n(&data, &params, &mut vars, config.n_samples, &mut e_tracer);
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
                let classification = Classification { sampled_classification, e_mean_calculated };
                sender.send(MessageToCentral { i_thread, classification }).unwrap();
            }
            MessageToWorker::Shutdown => {
                break;
            }
        }
    }
}