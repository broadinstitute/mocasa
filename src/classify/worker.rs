use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::classify::{MessageToCentral, MessageToWorker};
use crate::data::GwasData;
use crate::options::config::ClassifyConfig;
use crate::train::vars::Vars;
use crate::params::Params;
use crate::train::sampler::{ETracer, Sampler};
use crate::classify::exact::calculate_mu;
use crate::train::var_stats::MuSig;

struct ClassifyETracer<W: Write> {
    writer: W,
}

impl<W: Write> ETracer for ClassifyETracer<W> {
    fn trace_e(&mut self, e: f64) {
        if let Err(error) = writeln!(self.writer, "{}", e) {
            println!("Could not write E trace: {}", error)
        }
    }
}

pub(crate) fn classify_worker(data: &Arc<GwasData>, params: &Params, config: ClassifyConfig,
                              sender: Sender<MessageToCentral>,
                              receiver: Receiver<MessageToWorker>, i_thread: usize) {
    loop {
        let in_message = receiver.recv().unwrap();
        match in_message {
            MessageToWorker::DataPoint(i_data_point) => {
                let (data, is_col) = data.only_data_point(i_data_point);
                let trait_names = data.meta.trait_names.clone();
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
                                    println!("Could not create E trace file: {}", error);
                                    None
                                }
                            }
                        }
                        _ => { None }
                    };
                sampler.sample_n(&data, &params, &mut vars, config.n_steps_burn_in, &mut
                    e_tracer);
                sampler.sample_n(&data, &params, &mut vars, config.n_samples, &mut e_tracer);
                let MuSig { mu: mu_sampled, sig: sig_sampled }  = sampler.var_stats().calculate_mu_sig();
                let mu_calculated =
                    calculate_mu(&params, &data.betas[0], &data.ses[0]);
                sender.send(MessageToCentral { i_thread, mu_sampled, sig_sampled, mu_calculated }).unwrap();
            }
            MessageToWorker::Shutdown => {
                break;
            }
        }
    }
}