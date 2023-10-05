mod model;
mod param_stats;
mod sampler;
mod params;
mod vars;
mod param_eval;
mod worker;

use std::cmp;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use std::thread::{available_parallelism, JoinHandle, spawn};
use crate::data::{load_training_data, TrainData};
use crate::error::Error;
use crate::options::config::Config;
use crate::train::model::TrainModel;
use crate::train::params::Params;
use crate::train::worker::train_chain;

pub(crate) enum MessageToWorker {
    SendParamsAt(usize),
    Shutdown
}

pub(crate) struct MessageToCentral {
    i_thread: usize,
    params: Params
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
        launch_workers(&model, worker_sender, n_threads);
    let params = todo!();
    Ok(params)
}

fn launch_workers(model: &Arc<TrainModel>, worker_sender: Sender<MessageToCentral>,
                  n_threads: usize) -> (Vec<JoinHandle<()>>, Vec<Sender<MessageToWorker>>) {
    let mut join_handles: Vec<JoinHandle<()>> = Vec::with_capacity(n_threads);
    let mut senders: Vec<Sender<MessageToWorker>> = Vec::with_capacity(n_threads);
    for i_thread in 0..n_threads {
        let model = model.clone();
        let worker_sender = worker_sender.clone();
        let (sender, worker_receiver) =
            channel::<MessageToWorker>();
        let join_handle = spawn(move || {
            train_chain(model, worker_sender, worker_receiver, i_thread);
        });
        join_handles.push(join_handle);
        senders.push(sender);
    }
    (join_handles, senders)
}