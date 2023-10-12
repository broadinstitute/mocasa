mod model;
mod param_stats;
mod sampler;
mod params;
mod vars;
mod param_eval;
mod worker;
mod param_meta_stats;

use std::cmp;
use std::sync::Arc;
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::thread::{available_parallelism, JoinHandle, spawn};
use std::time::{Duration, SystemTime};
use crate::data::{load_training_data, TrainData};
use crate::error::Error;
use crate::options::config::{Config, TrainConfig};
use crate::train::model::TrainModel;
use crate::train::param_meta_stats::ParamMetaStats;
use crate::train::params::Params;
use crate::train::worker::train_chain;

pub(crate) enum MessageToWorker {
    TakeNSamples(usize),
    SetNewParams(Params),
    Shutdown,
}

pub(crate) struct MessageToCentral {
    i_thread: usize,
    params: Result<Params, Error>,
}

impl MessageToCentral {
    pub(crate) fn new(i_thread: usize, params: Result<Params, Error>) -> MessageToCentral {
        MessageToCentral { i_thread, params }
    }
}

pub(crate) fn train_or_check(config: &Config, dry: bool) -> Result<(), Error> {
    let data = load_training_data(config)?;
    println!("Loaded data for {} variants", data.meta.n_data_points());
    println!("{}", data);
    if !dry {
        train(data, config)?;
    }
    Ok(())
}

fn train(data: TrainData, config: &Config) -> Result<Params, Error> {
    let model = Arc::new(TrainModel::new(data));
    let n_threads = cmp::max(available_parallelism()?.get(), 3);
    let (worker_sender, receiver) =
        channel::<MessageToCentral>();
    println!("Launching {} workers and burning in with {} iterations", n_threads,
             config.train.n_steps_burn_in);
    let (join_handles, senders) =
        launch_workers(&model, worker_sender, n_threads, &config.train);
    println!("Workers launched and burned in.");
    let mut n_samples: usize = config.train.n_samples_initial;
    let mut params = model.initial_params();
    loop {
        println!("Asking workers to perform {} iterations.",
                 n_samples * config.train.n_steps_per_sample);
        let start_time = SystemTime::now();
        for sender in senders.iter() {
            sender.send(MessageToWorker::TakeNSamples(n_samples))?;
        }
        let mut param_estimates: Vec<Option<Result<Params, Error>>> =
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
        let n_iterations = config.train.n_steps_per_sample * n_samples;
        let secs_elapsed = (start_time.elapsed()?.as_millis() as f64) / 1000.0;
        let iterations_per_sec = (n_iterations as f64) / secs_elapsed;
        println!("Completed {} iterations over all data points and variables in {} seconds, \
        which is {} iterations per second and thread.",
                 n_iterations, secs_elapsed, iterations_per_sec);
        let mut meta_stats = ParamMetaStats::new(model.meta().clone());
        for param_estimate in param_estimates {
            match param_estimate {
                None => { unreachable!() }
                Some(Err(error)) => { println!("{}", error) }
                Some(Ok(params)) => {
                    let invalid_indices = params.invalid_indices();
                    if invalid_indices.is_empty() {
                        meta_stats.add(params)
                    } else {
                        for invalid_index in invalid_indices {
                            println!("{} is {}, which is invalid", invalid_index,
                                     params[invalid_index])
                        }
                    }
                }
            }
        }
        let summary = meta_stats.summary(params)?;
        println!("{}", summary);
        params =
            if summary.n_chains_used >= 3 {
                params = summary.params;
                if summary.intra_chains_mean < config.train.precision
                    && summary.intra_steps_mean < config.train.precision {
                    println!("Complete!");
                    break;
                }
                if summary.intra_chains_mean < summary.intra_steps_mean {
                    println!("Setting new parameters");
                    for sender in senders.iter() {
                        sender.send(MessageToWorker::SetNewParams(params.clone()))?;
                    }
                } else {
                    n_samples += (n_samples / 10) + 1;
                }
                params
            } else {
                summary.params_old
            };
    };
    shutdown_workers(join_handles, &senders);
    Ok(params)
}

fn launch_workers(model: &Arc<TrainModel>, worker_sender: Sender<MessageToCentral>,
                  n_threads: usize, config: &TrainConfig)
                  -> (Vec<JoinHandle<()>>, Vec<Sender<MessageToWorker>>) {
    let mut join_handles: Vec<JoinHandle<()>> = Vec::with_capacity(n_threads);
    let mut senders: Vec<Sender<MessageToWorker>> = Vec::with_capacity(n_threads);
    for i_thread in 0..n_threads {
        let model = model.clone();
        let worker_sender = worker_sender.clone();
        let (sender, worker_receiver) =
            channel::<MessageToWorker>();
        let config = config.clone();
        let join_handle = spawn(move || {
            train_chain(model, worker_sender, worker_receiver, i_thread, &config);
        });
        join_handles.push(join_handle);
        senders.push(sender);
    }
    (join_handles, senders)
}

fn shutdown_workers(join_handles: Vec<JoinHandle<()>>, senders: &[Sender<MessageToWorker>]) {
    for (i, sender) in senders.iter().enumerate() {
        match sender.send(MessageToWorker::Shutdown) {
            Ok(_) => { println!("Worker {} requested to shut down.", i)}
            Err(_) => { println!("Could not reach worker {}.", i) }
        };
    }
    for (i, join_handle) in join_handles.into_iter().enumerate() {
        match join_handle.join() {
            Ok(_) => { println!("Worker {} shutdown.", i) }
            Err(_) => { println!("Worker {} crashed.", i) }
        }
    }
}