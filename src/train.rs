mod model;
mod param_stats;
mod sampler;
mod params;
mod vars;
mod param_eval;
mod worker;

use std::cmp;
use std::sync::Arc;
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::thread::{available_parallelism, JoinHandle, spawn};
use std::time::Duration;
use crate::data::{load_training_data, TrainData};
use crate::error::Error;
use crate::options::config::{Config, TrainConfig};
use crate::train::model::TrainModel;
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
    let (join_handles, senders) =
        launch_workers(&model, worker_sender, n_threads, &config.train);
    let mut n_samples: usize = config.train.n_samples_initial;
    let params = loop {
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
                Err(RecvTimeoutError::Timeout) => { /* Just continue */ }
                Err(RecvTimeoutError::Disconnected) => { receive_result?; }
            }
        }

        todo!()
    };
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