use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::data::GwasData;
use crate::options::config::TrainConfig;
use crate::train::{MessageToCentral, MessageToWorker};
use crate::train::params::Params;
use crate::train::sampler::Sampler;
use crate::train::vars::Vars;

pub(crate) fn train_worker(data: &Arc<GwasData>, mut params: Params,
                           sender: Sender<MessageToCentral>, receiver: Receiver<MessageToWorker>,
                           i_thread: usize, config: &TrainConfig) {
    let mut vars = Vars::initial_vars(data, &params);
    let rng = thread_rng();
    let meta = data.meta.clone();
    let mut sampler = Sampler::<ThreadRng>::new(&meta, rng);
    sampler.sample_n(data, &params, &mut vars, config.n_steps_burn_in, &mut None);
    loop {
        let in_message = receiver.recv().unwrap();
        match in_message {
            MessageToWorker::TakeNSamples(n_samples) => {
                sampler.sample_n(data, &params, &mut vars, n_samples, &mut None);
                let params_new = sampler.var_stats().compute_new_params();
                sender
                    .send(MessageToCentral::new(i_thread, params_new))
                    .unwrap();
            }
            MessageToWorker::SetNewParams(params_new) => {
                params = params_new;
                sampler.sample_n(data, &params, &mut vars, config.n_steps_burn_in,
                                 &mut None);
            }
            MessageToWorker::Shutdown => {
                break;
            }
        }
    }
}
