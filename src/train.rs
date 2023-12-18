mod model;
mod sampler;
pub(crate) mod params;
mod vars;
mod worker;
pub(crate) mod param_meta_stats;
mod initial_params;
pub(crate) mod var_stats;
mod gibbs;
pub(crate) mod trace_file;

use std::cmp;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread::{available_parallelism, JoinHandle, spawn};
use std::time::Duration;
use crate::data::{load_train_data, GwasData};
use crate::error::Error;
use crate::options::config::{Config, TrainConfig};
use crate::report::Reporter;
use crate::train::initial_params::estimate_initial_params;
use crate::train::model::TrainModel;
use crate::train::param_meta_stats::ParamMetaStats;
use crate::train::params::Params;
use crate::train::trace_file::ParamTraceFileWriter;
use crate::train::worker::train_chain;

pub(crate) enum MessageToWorker {
    TakeNSamples(usize),
    SetNewParams(Params),
    Shutdown,
}

pub(crate) struct MessageToCentral {
    i_thread: usize,
    params: Params,
}

impl MessageToCentral {
    pub(crate) fn new(i_thread: usize, params: Params) -> MessageToCentral {
        MessageToCentral { i_thread, params }
    }
}

pub(crate) fn train_or_check(config: &Config, dry: bool) -> Result<(), Error> {
    let data = load_train_data(config)?;
    println!("Loaded data for {} variants", data.meta.n_data_points());
    println!("{}", data);
    if dry {
        println!("User picked dry run only, so doing nothing.")
    } else {
        train(data, config)?;
    }
    Ok(())
}

fn train(data: GwasData, config: &Config) -> Result<(), Error> {
    let model = Arc::new(TrainModel::new(data));
    let n_traits = model.meta().n_traits();
    let mut params_trace_writer =
        if let Some(path) = &config.files.trace {
            let path = PathBuf::from(path);
            Some(ParamTraceFileWriter::new(path, n_traits)?)
        } else {
            None
        };
    let n_threads = cmp::max(available_parallelism()?.get(), 3);
    let (worker_sender, receiver) =
        channel::<MessageToCentral>();
    println!("Launching {} workers and burning in with {} iterations", n_threads,
             config.train.n_steps_burn_in);
    let mut params = estimate_initial_params(&model)?;
    println!("{}", params);
    let (join_handles, senders) =
        launch_workers(&model, &params, worker_sender, n_threads, &config.train);
    println!("Workers launched and burned in.");
    let n_samples: usize = config.train.n_samples_per_iteration;
    let mut reporter = Reporter::new();
    let mut i_round: usize = 0;
    let mut i_iteration: usize = 0;
    loop {
        let params0 = create_param_estimates(&senders, &receiver, n_samples)?;
        let params1 = create_param_estimates(&senders, &receiver, n_samples)?;
        let mut param_meta_stats =
            ParamMetaStats::new(n_threads, params.trait_names.clone(),
                                &params0, &params1);
        let mut reached_precision = false;
        loop {
            i_iteration += 1;
            let params_new = create_param_estimates(&senders, &receiver, n_samples)?;
            param_meta_stats.add(&params_new);
            let summary = param_meta_stats.summary()?;
            if i_iteration >= config.train.n_iterations_per_round {
                params = summary.params.clone();
                if let Some(params_trace_writer) = &mut params_trace_writer {
                    params_trace_writer.write(&params)?;
                }
                if i_round >= config.train.n_rounds {
                    println!("Done!");
                    reached_precision = true;
                } else {
                    i_round += 1;
                    println!("Setting new parameters for round {} after {} iterations", i_round,
                             i_iteration);
                    i_iteration = 0;
                    for sender in senders.iter() {
                        sender.send(MessageToWorker::SetNewParams(params.clone()))?;
                    }
                }
                reporter.report(&summary, i_round, i_iteration, n_samples);
                reporter.reset_round_timer();
                break;
            }
        }
        if reached_precision {
            break;
        }
    };
    write_params_to_file(&params, config.files.params.as_str())?;
    shutdown_workers(join_handles, &senders);
    Ok(())
}

fn create_param_estimates(senders: &[Sender<MessageToWorker>],
                          receiver: &Receiver<MessageToCentral>, n_samples: usize)
                          -> Result<Vec<Params>, Error> {
    let n_threads = senders.len();
    for sender in senders.iter() {
        sender.send(MessageToWorker::TakeNSamples(n_samples))?;
    }
    let mut param_estimates: Vec<Option<Params>> =
        (0..n_threads).map(|_| None).collect();
    while param_estimates.iter().any(|param_opt| param_opt.is_none()) {
        let receive_result =
            receiver.recv_timeout(Duration::from_secs(100));
        match receive_result {
            Ok(message) => {
                let MessageToCentral { i_thread, params } = message;
                param_estimates[i_thread] = Some(params);
            }
            Err(RecvTimeoutError::Timeout) => {
                println!("Still waiting for worker threads.")
            }
            Err(RecvTimeoutError::Disconnected) => { receive_result?; }
        }
    }
    let mut params: Vec<Params> = Vec::with_capacity(n_threads);
    for param_estimate in param_estimates {
        match param_estimate {
            None => {
                Err(Error::from("Did not get parameter estimates from all workers"))?
            }
            Some(param_result) => {
                params.push(param_result)
            }
        }
    }
    Ok(params)
}

fn launch_workers(model: &Arc<TrainModel>, params: &Params,
                  worker_sender: Sender<MessageToCentral>, n_threads: usize, config: &TrainConfig)
                  -> (Vec<JoinHandle<()>>, Vec<Sender<MessageToWorker>>) {
    let mut join_handles: Vec<JoinHandle<()>> = Vec::with_capacity(n_threads);
    let mut senders: Vec<Sender<MessageToWorker>> = Vec::with_capacity(n_threads);
    for i_thread in 0..n_threads {
        let model = model.clone();
        let worker_sender = worker_sender.clone();
        let (sender, worker_receiver) =
            channel::<MessageToWorker>();
        let config = config.clone();
        let params = params.clone();
        let join_handle = spawn(move || {
            train_chain(model, params, worker_sender, worker_receiver, i_thread,
                        &config);
        });
        join_handles.push(join_handle);
        senders.push(sender);
    }
    (join_handles, senders)
}

fn shutdown_workers(join_handles: Vec<JoinHandle<()>>, senders: &[Sender<MessageToWorker>]) {
    for (i, sender) in senders.iter().enumerate() {
        match sender.send(MessageToWorker::Shutdown) {
            Ok(_) => { println!("Sent to worker {} request to shut down.", i) }
            Err(_) => { println!("Could not reach worker {}.", i) }
        };
    }
    for (i, join_handle) in join_handles.into_iter().enumerate() {
        match join_handle.join() {
            Ok(_) => { println!("Worker {} has shut down.", i) }
            Err(_) => { println!("Worker {} has crashed.", i) }
        }
    }
}

fn write_params_to_file(params: &Params, output_file: &str) -> Result<(), Error> {
    let mut writer = BufWriter::new(File::create(output_file)?);
    let json = serde_json::to_string(params)?;
    writeln!(writer, "{}", json)?;
    Ok(())
}
