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
use crate::options::config::{ClassifyConfig, Config, SharedConfig};
use crate::params::{Params, read_params_from_file};
use crate::util::threads::{InMessage, OutMessage, TaskQueueObserver, Threads, WorkerLauncher};
use std::io::Write;
use log::{info, trace, warn};
use crate::classify::worker::classify_worker;
use crate::options::cli::{Chunking, Flags};
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
    i_data_point: usize,
    sampled_classification: SampledClassification,
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
    config_shared: SharedConfig,
}

impl WorkerLauncher<MessageToCentral, MessageToWorker> for ClassifyWorkerLauncher {
    fn launch(self, in_sender: Sender<MessageToCentral>, out_receiver: Receiver<MessageToWorker>,
              i_thread: usize) {
        let ClassifyWorkerLauncher {
            data, params, config, config_shared
        } = self;
        classify_worker(&data, &params, config, config_shared, in_sender, out_receiver, i_thread);
    }
}

pub(crate) fn classify_or_check(config: &Config, flags: Flags) -> Result<(), Error> {
    info!("Loading params");
    let params = read_params_from_file(&config.files.params)?;
    trace!("Loaded params: {}", params);

    let params =
        match &config.classify.params_override {
            None => { params }
            Some(overwrite) => {
                let params = params.plus_overwrite(overwrite);
                trace!("Overwritten params: {}", params);
                params
            }
        };
    trace!("Loaded params for {} endophenotypes and {} traits.", params.n_endos(),
        params.n_traits());
    trace!("Loading data");
    let data = load_data(config, Action::Classify)?;
    trace!("Loaded data with {} data points and {} traits.", data.n_data_points(), data.n_traits());
    if params.n_endos() != config.shared.n_endos {
        Err(Error::from(format!(
            "Number of endophenotypes is {} in configuration, but {} in parameters.",
            config.shared.n_endos, params.n_endos()
        )))?;
    }
    if flags.dry {
        info!("User picked dry run only, so doing nothing.")
    } else {
        classify(data, params, config, &flags.chunking)?;
    }
    Ok(())
}

pub(crate) fn classify(data: GwasData, params: Params, config: &Config,
                       chunking: &Option<Chunking>)
                       -> Result<(), Error> {
    let data = Arc::new(data);
    let n_threads = cmp::max(available_parallelism()?.get(), 3);
    let config_shared = config.shared.clone();
    let config = config.classify.clone();
    match chunking {
        None => { warn!("No chunking") }
        Some(Chunking { n_chunks, i_chunk }) => {
            warn!("Chunking into {} chunks, this is chunk {}", n_chunks, i_chunk)
        }
    }
    let out_file =
        match chunking {
            None => config.out_file.clone(),
            Some(Chunking { i_chunk, .. }) => { format!("{}_{}", config.out_file, i_chunk) }
        };
    let temp_out_file = format!("{}_tmp", out_file);
    let launcher = ClassifyWorkerLauncher { data: data.clone(), params, config, config_shared };
    let threads = Threads::new(launcher, n_threads);
    let meta = &data.meta;
    let n_data_points = meta.n_data_points();
    let out_messages =
        match chunking {
            None => { (0..n_data_points).map(MessageToWorker::DataPoint) }
            Some(Chunking { n_chunks, i_chunk }) => {
                let start = (i_chunk - 1) * n_data_points / n_chunks;
                let end = i_chunk * n_data_points / n_chunks;
                (start..end).map(MessageToWorker::DataPoint)
            }
        };
    let mut observer =
        Observer::new(&meta.var_ids, &temp_out_file, meta.clone())?;
    let in_messages = threads.task_queue(out_messages, &mut observer)?;
    let classifications: Vec<Classification> =
        in_messages.into_iter().map(|in_message| in_message.classification).collect();
    write_out_file(&out_file, meta, &classifications)?;
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
    let n_endos = meta.n_endos();
    let e_part =
        (0..meta.n_endos())
            .map(|i_endo| format!("e_{}_mean\te_{}_std\te_{}_z", i_endo, i_endo, i_endo))
            .collect::<Vec<String>>().join("\t");
    let traits_part = meta.trait_names.join("\t");
    if n_endos == 1 {
        writeln!(writer, "id\t{}\te_mean_calc\t{}", e_part, traits_part)?;
    } else {
        writeln!(writer, "id\t{}\t{}", e_part, traits_part)?;
    }
    Ok(())
}

fn write_entry(writer: &mut BufWriter<File>, id: &str, classification: &Classification)
               -> Result<(), Error> {
    let Classification {
        i_data_point, sampled_classification: sampled,
        e_mean_calculated
    } = classification;
    let SampledClassification { e_mean, e_std, t_means } = sampled;
    let e_part =
        e_mean.iter().zip(e_std.iter())
            .map(|(e_mean, e_std)| {
                let e_z = e_mean / e_std;
                format!("{}\t{}\t{}", e_mean, e_std, e_z)
            })
            .collect::<Vec<String>>().join("\t");
    let t_means_part =
        t_means.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("\t");
    if e_mean.len() == 1 {
        writeln!(writer, "{}\t{}\t{}\t{}", id, e_part, e_mean_calculated, t_means_part)?;
    } else {
        writeln!(writer, "{}\t{}\t{}", id, e_part, t_means_part)?;
    }
    Ok(())
}