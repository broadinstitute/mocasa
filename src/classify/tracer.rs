use std::fmt::Display;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use log::warn;
use crate::data::Meta;
use crate::error::Error;
use crate::sample::sampler::Tracer;


struct Writer {
    buffer: Vec<f64>,
    writer: Result<BufWriter<File>, Error>
}

pub(crate) struct ClassifyTracer {
    e_writers: Vec<Writer>,
    t_writers: Vec<Writer>,
}

impl Writer {
    fn new(file_name: &str, n_chains: usize) -> Writer {
        let buffer = vec![f64::NAN; n_chains];
        let writer = try_writer(file_name);
        Writer { buffer, writer }
    }
    fn prepare(&mut self, i_chain: usize, value: f64) {
        self.buffer[i_chain] = value;
    }
    fn write_values(&mut self) {
        if let Ok(ref mut writer) = self.writer {
            if let Err(error) = writeln!(writer, "{}", self.buffer.join("\t")) {
                warn!("Could not write trace: {}", error)
            }
        }
    }
}

impl ClassifyTracer {
    pub(crate) fn new(meta: &Meta, out_file_name: &str, var_id: &str, n_endos: usize,
                      n_chains: usize) -> ClassifyTracer {
        let e_writers = (0..n_endos).map(|i_endo| {
            let var_name = format!("E_{}", i_endo);
            let file_name = format!("{}_{}_trace_{}", out_file_name, var_id, var_name);
            Writer::new(&file_name, n_chains)
        }).collect::<Vec<_>>();
        let t_writers = (0..meta.n_traits()).map(|i_trait| {
            let var_name = format!("T_{}", i_trait);
            let file_name = format!("{}_{}_trace_{}", out_file_name, var_id, var_name);
            Writer::new(&file_name, n_chains)
        }).collect::<Vec<_>>();
        ClassifyTracer { e_writers, t_writers }
    }
}

fn try_writer(file_name: &str) -> Result<BufWriter<File>, Error> {
    match File::create(file_name) {
        Ok(file) => { Ok(BufWriter::new(file)) }
        Err(error) => { Err(Error::from(error)) }
    }
}

fn try_trace(writer: &mut Writer, name: &str, index: usize,
             item1: &dyn Display, item2: &dyn Display) {
    match writer.writer {
        Ok(ref mut writer) => {
            if let Err(error) = writeln!(writer, "{}\t{}", item1, item2) {
                warn!("Could not write {}_{} trace: {}", name, index, error)
            }
        }
        Err(ref error) => {
            warn!("Could not write {}_{} trace: {}", name, index, error)
        }
    }
}

impl Tracer for ClassifyTracer {
    fn trace_e(&mut self, i_endo: usize, e: f64, i_chain: usize) {
        try_trace(&mut self.e_writers[i_endo], "E", i_endo, &e, &i_chain);
    }

    fn trace_t(&mut self, i_trait: usize, t: f64, i_chain: usize) {
        try_trace(&mut self.t_writers[i_trait], "T", i_trait, &t, &i_chain);
    }
    fn write_values(&mut self) {
        for writer in self.e_writers.iter_mut() {
            writer.write_values()
        }
        for writer in self.t_writers.iter_mut() {
            writer.write_values()
        }
    }
}
