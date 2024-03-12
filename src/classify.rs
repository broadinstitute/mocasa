mod worker;
mod exact;

use std::cmp;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::available_parallelism;
use crate::data::{GwasData, load_data, Meta};
use crate::error::{Error, for_file};
use crate::options::action::Action;
use crate::options::config::{ClassifyConfig, Config};
use crate::params::{Params, read_params_from_file};
use crate::util::threads::{InMessage, OutMessage, TaskQueueObserver, Threads, WorkerLauncher};
use std::io::Write;
use crate::classify::worker::classify_worker;
use crate::sample::var_stats::SampledClassification;

#[derive(Clone)]
pub(crate) enum MessageToWorker {
    DataPoint(usize),
    Shutdown,
}

impl OutMessage for MessageToWorker {
    const SHUTDOWN: Self = MessageToWorker::Shutdown;
}

struct Classification {
    sampled: SampledClassification,
    e_mean_calculated: f64,
}

pub(crate) struct MessageToCentral {
    i_thread: usize,
    classification: Classification,
}

impl InMessage for MessageToCentral {
    fn i_thread(&self) -> usize { self.i_thread }
}

struct Observer {
    meta: Meta,
    var_ids: Arc<Vec<String>>,
    writer: BufWriter<File>,
}

impl Observer {
    fn new(var_ids: &Arc<Vec<String>>, file_name: &str, meta: Meta) -> Result<Observer, Error> {
        let var_ids = var_ids.clone();
        let writer =
            BufWriter::new(for_file(file_name, File::create(file_name))?);
        Ok(Observer { meta, var_ids, writer })
    }
}

impl TaskQueueObserver<MessageToCentral, MessageToWorker> for Observer {
    fn going_to_start_queue(&mut self) {
        println!("Starting to classify data points.");
        if let Err(error) = write_header(&mut self.writer, &self.meta) {
            println!("Cannot write temp file: {}", error)
        }
    }

    fn going_to_send(&mut self, out_message: &MessageToWorker, i_task: usize, i_thread: usize) {
        if matches!(out_message, MessageToWorker::Shutdown) {
            println!("Sent shutdown as task {} to thread {}", i_task, i_thread)
        }
    }

    fn have_received(&mut self, in_message: &MessageToCentral, i_task: usize, _: usize) {
        let var_id = &self.var_ids[i_task];
        let io_result =
            write_entry(&mut self.writer, var_id, &in_message.classification);
        if let Err(error) = io_result {
            println!("Cannot write temp file: {}", error)
        }
    }

    fn nothing_more_to_send(&mut self) {
        println!("No more data points to add to queue.")
    }

    fn completed_queue(&mut self) {
        println!("Completed classification of all data points.")
    }
}

#[derive(Clone)]
struct ClassifyWorkerLauncher {
    data: Arc<GwasData>,
    params: Params,
    config: ClassifyConfig,
}

impl WorkerLauncher<MessageToCentral, MessageToWorker> for ClassifyWorkerLauncher {
    fn launch(self, in_sender: Sender<MessageToCentral>, out_receiver: Receiver<MessageToWorker>,
              i_thread: usize) {
        let ClassifyWorkerLauncher { data, params, config } = self;
        classify_worker(&data, &params, config, in_sender, out_receiver, i_thread);
    }
}

pub(crate) fn classify_or_check(config: &Config, dry: bool) -> Result<(), Error> {
    let params = read_params_from_file(&config.files.params)?;
    println!("Read from file mu = {}, tau = {}", params.mu, params.tau);

    let params =
        match &config.classify.params_override {
            None => { params }
            Some(overwrite) => {
                let params = params.plus_overwrite(overwrite);
                println!("After overwrite, mu = {}, tau = {}", params.mu, params.tau);
                params
            }
        };
    let data = load_data(config, Action::Classify)?;
    if dry {
        println!("User picked dry run only, so doing nothing.")
    } else {
        classify(data, params, config)?;
    }
    Ok(())
}

pub(crate) fn classify(data: GwasData, params: Params, config: &Config) -> Result<(), Error> {
    let data = Arc::new(data);
    let n_threads = cmp::max(available_parallelism()?.get(), 3);
    let config = config.classify.clone();
    let launcher = ClassifyWorkerLauncher { data: data.clone(), params, config: config.clone() };
    let threads = Threads::new(launcher, n_threads);
    let meta = &data.meta;
    let out_messages =
        (0..meta.n_data_points()).map(MessageToWorker::DataPoint);
    let temp_out_file = format!("{}_tmp", config.out_file);
    let mut observer =
        Observer::new(&meta.var_ids, &temp_out_file, meta.clone())?;
    let in_messages = threads.task_queue(out_messages, &mut observer)?;
    let classifications: Vec<Classification> =
        in_messages.into_iter().map(|in_message| in_message.classification).collect();
    write_out_file(&config.out_file, meta, &classifications)?;
    Ok(())
}

fn write_out_file(file: &str, meta: &Meta, classifications: &[Classification])
                  -> Result<(), Error> {
    let mut writer = BufWriter::new(for_file(file, File::create(file))?);
    write_header(&mut writer, meta)?;
    for (id, classification)
    in meta.var_ids.iter().zip(classifications.iter()) {
        write_entry(&mut writer, id, classification)?;
    }
    Ok(())
}

fn write_header(writer: &mut BufWriter<File>, meta: &Meta) -> Result<(), Error> {
    let traits_part = meta.trait_names.join("\t");
    writeln!(writer, "id\te_mean_samp\te_std_samp\te_mean_calc\t{}", traits_part)?;
    Ok(())
}

fn write_entry(writer: &mut BufWriter<File>, id: &str, classification: &Classification)
               -> Result<(), Error> {
    let Classification { sampled, e_mean_calculated } = classification;
    let SampledClassification { e_mean, e_std, t_means } = sampled;
    let t_means_part =
        t_means.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("\t");
    writeln!(writer, "{}\t{}\t{}\t{}\t{}", id, e_mean, e_std, e_mean_calculated, t_means_part)?;
    Ok(())
}