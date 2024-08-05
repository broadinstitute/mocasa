use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use log::{info, warn};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::classify::{Classification, MessageToCentral, MessageToWorker};
use crate::data::{GwasData, Meta};
use crate::options::config::{ClassifyConfig, SharedConfig};
use crate::sample::vars::Vars;
use crate::params::Params;
use crate::sample::sampler::{Tracer, Sampler, NoOpTracer};
use crate::classify::exact::calculate_mu;
use crate::error::Error;

struct ClassifyTracer {
    e_writers: Vec<Result<BufWriter<File>, Error>>,
    t_writers: Vec<Result<BufWriter<File>, Error>>,
}

impl ClassifyTracer {
    fn new(meta: &Meta, out_file_name: &str, var_id: &str, n_endos: usize) -> ClassifyTracer {
        let e_writers = (0..n_endos).map(|i_endo| {
            let var_name = format!("E_{}", i_endo);
            let file_name = format!("{}_{}_trace_{}", out_file_name, var_id, var_name);
            let mut writer = try_writer(&file_name);
            try_trace(&mut writer, "E", i_endo, &var_name, &"chain");
            writer
        }).collect::<Vec<_>>();
        let t_writers = (0..meta.n_traits()).map(|i_trait| {
            let var_name = format!("T_{}", i_trait);
            let file_name = format!("{}_{}_trace_{}", out_file_name, var_id, var_name);
            let mut writer = try_writer(&file_name);
            try_trace(&mut writer, "T", i_trait, &var_name, &"chain");
            writer
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

fn try_trace(writer: &mut Result<BufWriter<File>, Error>, name: &str, index: usize,
             item1: &dyn Display, item2: &dyn Display) {
    match writer {
        Ok(ref mut writer) => {
            if let Err(error) = writeln!(writer, "{}\t{}", item1, item2) {
                warn!("Could not write {}_{} trace: {}", name, index, error)
            }
        }
        Err(ref error) => {
            warn!("Could not write {}_{} trace: {}", name, index, error)
        }
    }
}

impl Tracer for ClassifyTracer {
    fn trace_e(&mut self, i_endo: usize, e: f64, i_chain: usize) {
        try_trace(&mut self.e_writers[i_endo], "E", i_endo, &e, &i_chain);
    }

    fn trace_t(&mut self, i_trait: usize, t: f64, i_chain: usize) {
        try_trace(&mut self.t_writers[i_trait], "T", i_trait, &t, &i_chain);
    }
}

pub(crate) fn classify_worker(data: &Arc<GwasData>, params: &Params, config: ClassifyConfig,
                              config_shared: SharedConfig, sender: Sender<MessageToCentral>,
                              receiver: Receiver<MessageToWorker>, i_thread: usize) {
    loop {
        let in_message = receiver.recv().unwrap();
        match in_message {
            MessageToWorker::DataPoint(i_data_point) => {
                let n_traits_total = data.meta.n_traits();
                let (data, is_col) = data.only_data_point(i_data_point);
                let trait_names = data.meta.trait_names.clone();
                if data.meta.n_traits() < n_traits_total {
                    let id = &data.meta.var_ids()[0];
                    let trait_list = trait_names.join(", ");
                    warn!("For {}, of the {} traits, we only have data for {} traits ({}).", id,
                        n_traits_total, data.meta.n_traits(), trait_list)
                }
                let params = params.reduce_to(trait_names, &is_col);
                let mut vars: Vec<Vars> =
                    (0..config.n_parallel())
                        .map(|_| Vars::initial_vars(&data, &params))
                        .collect();
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
                            Box::new(tracer) as Box<dyn Tracer>
                        }
                        _ => { Box::new(NoOpTracer::new()) }
                    };
                sampler.sample_n(&data, &params, &mut vars, config_shared.n_steps_burn_in,
                                 tracer.as_mut());
                sampler.sample_n(&data, &params, &mut vars, config.n_samples, tracer.as_mut());
                let sampled_classification =
                    sampler.var_stats().calculate_classification();
                let e_mean_calculated =
                    match calculate_mu(&params, &data.betas[0], &data.ses[0]) {
                        Ok(mu) => { mu }
                        Err(error) => {
                            info!("{}", error);
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