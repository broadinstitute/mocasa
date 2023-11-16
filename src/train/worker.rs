use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::options::config::TrainConfig;
use crate::train::{MessageToCentral, MessageToWorker};
use crate::train::model::TrainModel;
use crate::train::params::Params;
use crate::train::sampler::Sampler;

pub(crate) fn train_chain(model: Arc<TrainModel>, mut params: Params,
                          sender: Sender<MessageToCentral>, receiver: Receiver<MessageToWorker>,
                          i_thread: usize, config: &TrainConfig) {
    let mut vars = model.initial_vars(&params);
    let rng = thread_rng();
    let meta = model.meta().clone();
    let mut sampler = Sampler::<ThreadRng>::new(&meta, rng, &params);
    sampler.sample_n(&model, &params, &mut vars, config.n_steps_burn_in);
    sampler.squash_stats();
    loop {
        let in_message = receiver.recv().unwrap();
        match in_message {
            MessageToWorker::TakeNSamples(n_samples) => {
                sampler.sample_n(&model, &params, &mut vars, n_samples);
                let params_new = sampler.var_stats().compute_new_params();
                sender
                    .send(MessageToCentral::new(i_thread, params_new))
                    .unwrap();
            }
            MessageToWorker::SetNewParams(params_new) => {
                params = params_new;
                sampler.squash_stats();
                sampler.sample_n(&model, &params, &mut vars, config.n_steps_burn_in);
                sampler.squash_stats();
            }
            MessageToWorker::Shutdown => {
                break;
            }
        }
    }
}
