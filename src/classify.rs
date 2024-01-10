mod worker;
mod exact;

use std::cmp;
use std::fs::{File, read_to_string};
use std::io::BufWriter;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::available_parallelism;
use crate::data::{GwasData, load_data, Meta};
use crate::error::{Error, for_file};
use crate::options::action::Action;
use crate::options::config::{ClassifyConfig, Config};
use crate::train::params::Params;
use crate::util::threads::{InMessage, OutMessage, TaskQueueObserver, Threads, WorkerLauncher};
use std::io::Write;
use crate::classify::worker::classify_worker;

#[derive(Clone)]
pub(crate) enum MessageToWorker {
    DataPoint(usize),
    Shutdown,
}

impl OutMessage for MessageToWorker {
    const SHUTDOWN: Self = MessageToWorker::Shutdown;
}

pub(crate) struct MessageToCentral {
    i_thread: usize,
    mu_sampled: f64,
    mu_calculated: f64,
}

impl InMessage for MessageToCentral {
    fn i_thread(&self) -> usize { self.i_thread }
}

struct Observer {
    writer: BufWriter<File>
}

impl Observer {
    fn new(file_name: &str) -> Result<Observer, Error> {
        let writer = BufWriter::new(File::create(file_name)?);
        Ok(Observer { writer })
    }
}

impl TaskQueueObserver<MessageToCentral, MessageToWorker> for Observer {
    fn going_to_start_queue(&mut self) {
        println!("Starting to classify data points.");
        if let Err(error) = writeln!(self.writer, "samples\tcalculated") {
            println!("Cannot write temp file: {}", error)
        }
    }

    fn going_to_send(&mut self, out_message: &MessageToWorker, i_task: usize, i_thread: usize) {
        match out_message {
            MessageToWorker::DataPoint(i_data_point) => {
                println!("Sent {} as task {} for thread {}.", i_data_point, i_task, i_thread)
            }
            MessageToWorker::Shutdown => {
                println!("Sent shutdown as task {} to thread {}", i_task, i_thread)
            }
        }
    }

    fn have_received(&mut self, in_message: &MessageToCentral, i_task: usize, i_thread: usize) {
        let mu_sampled = in_message.mu_sampled;
        let mu_calculated = in_message.mu_calculated;
        println!("Got mu_sampled = {} and mu_calculated = {} as task {} from thread {}",
                 mu_sampled, mu_calculated, i_task, i_thread);
        if let Err(error) = writeln!(self.writer, "{}\t{}", mu_sampled, mu_calculated) {
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
    let params = read_params(&config.files.params)?;
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
    let out_messages =
        (0..data.meta.n_data_points()).map(MessageToWorker::DataPoint);
    let temp_out_file = format!("{}_tmp", config.out_file);
    let mut observer = Observer::new(&temp_out_file)?;
    let in_messages = threads.task_queue(out_messages, &mut observer)?;
    let mus_sampled: Vec<f64> =
        in_messages.iter().map(|in_message| in_message.mu_sampled).collect();
    let mus_calculated: Vec<f64> =
        in_messages.iter().map(|in_message| in_message.mu_calculated).collect();
    write_mus_to_file(&config.out_file, &data.meta, &mus_sampled, &mus_calculated)?;
    Ok(())
}

fn read_params(file: &str) -> Result<Params, Error> {
    let params_string = for_file(file, read_to_string(file))?;
    let params = serde_json::from_str(&params_string)?;
    Ok(params)
}

fn write_mus_to_file(file: &str, meta: &Meta, mus_sampled: &[f64], mus_calculated: &[f64])
                     -> Result<(), Error> {
    let mut writer = BufWriter::new(File::create(file)?);
    writeln!(writer, "id\tmu_samp\tmu_calc")?;
    for ((id, &mu_sampled), &mu_calculated)
    in meta.var_ids.iter().zip(mus_sampled.iter()).zip(mus_calculated.iter()) {
        writeln!(writer, "{}\t{}\t{}", id, mu_sampled, mu_calculated)?;
    }
    Ok(())
}