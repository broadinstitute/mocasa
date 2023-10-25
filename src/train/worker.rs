use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::options::config::TrainConfig;
use crate::train::{MessageToCentral, MessageToWorker};
use crate::train::model::TrainModel;
use crate::train::param_stats::ParamHessianStats;
use crate::train::params::Params;
use crate::train::sampler::Sampler;

pub(crate) fn train_chain(model: Arc<TrainModel>, mut params: Params,
                          sender: Sender<MessageToCentral>, receiver: Receiver<MessageToWorker>,
                          i_thread: usize, config: &TrainConfig) {
    let mut vars = model.initial_vars(&params);
    let rng = thread_rng();
    let meta = model.meta().clone();
    let mut sampler = Sampler::<ThreadRng>::new(meta, rng, &params);
    let mut stats = ParamHessianStats::new(model.meta().clone());
    sampler.sample_n(&model, &params, &mut vars, config.n_steps_burn_in);
    sampler.squash_stats();
    loop {
        let in_message = receiver.recv().unwrap();
        match in_message {
            MessageToWorker::TakeNSamples(n_samples) => {
                for _ in 0..n_samples {
                    sampler.sample_n(&model, &params, &mut vars, config.n_steps_per_sample);
                    let param_eval = model.param_eval(&params, &vars);
                    stats.survey_param_eval(&param_eval);
                }
                sender
                    .send(MessageToCentral::new(i_thread, stats.estimate_params(&params)))
                    .unwrap();
            }
            MessageToWorker::SetNewParams(params_new) => {
                params = params_new;
                stats.squash();
                sampler.squash_stats();
                sampler.sample_n(&model, &params, &mut vars, config.n_steps_burn_in);
                sampler.squash_stats();
            }
            MessageToWorker::Shutdown => {
                break
            }
        }
    }
}
