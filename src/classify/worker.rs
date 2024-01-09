use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::classify::{MessageToCentral, MessageToWorker};
use crate::data::GwasData;
use crate::options::config::ClassifyConfig;
use crate::train::vars::Vars;
use crate::train::params::Params;
use crate::train::sampler::Sampler;
use crate::classify::exact::calculate_mu;

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
                sampler.sample_n(&data, &params, &mut vars, config.n_steps_burn_in);
                sampler.sample_n(&data, &params, &mut vars, config.n_samples);
                let mu_sampled = sampler.var_stats().calculate_mu();
                let mu_calculated =
                    calculate_mu(&params, &data.betas[0], &data.ses[0]);
                sender.send(MessageToCentral { i_thread, mu_sampled, mu_calculated }).unwrap();
            }
            MessageToWorker::Shutdown => {
                break;
            }
        }
    }
}