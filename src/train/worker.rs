use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::train::{MessageToCentral, MessageToWorker};
use crate::train::model::TrainModel;
use crate::train::param_stats::ParamHessianStats;
use crate::train::sampler::Sampler;

pub(crate) fn train_chain(model: Arc<TrainModel>, sender: Sender<MessageToCentral>,
                          receiver: Receiver<MessageToWorker>, i_thread: usize) {
    let mut params = model.initial_params();
    let mut vars = model.initial_vars(&params);
    let rng = thread_rng();
    let meta = model.meta().clone();
    let mut sampler = Sampler::<ThreadRng>::new(meta, rng);
    loop {
        let mut stats = ParamHessianStats::new(model.meta().n_traits());
        let stats =
            loop {
                sampler.sample(&model, &params, &mut vars);
                let param_eval = model.param_eval(&params, &vars);
                stats.survey_param_eval(&param_eval);
                if stats.ready_for_param_estimate() {
                    break stats;
                }
            };
        let estimate = stats.estimate_params();
        params = estimate.params;
        if estimate.is_done { break }
    }
}