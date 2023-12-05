use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use crate::error::Error;
use crate::train::params::{ParamIndex, Params};

pub(crate) struct ParamTraceFileWriter {
    path: PathBuf,
    index: usize
}

impl ParamTraceFileWriter {
    pub(crate) fn new(path: PathBuf, n_traits: usize) -> Result<ParamTraceFileWriter, Error> {
        let index: usize = 0;
        let mut writer = BufWriter::new(File::create(&path)?);
        write!(writer, "index")?;
        for param_index in ParamIndex::all(n_traits) {
            write!(writer, "\t{}", param_index)?;
        }
        writeln!(writer)?;
        Ok(ParamTraceFileWriter { path, index })
    }
    pub(crate) fn write(&mut self, params: &Params) -> Result<(), Error> {
        self.index += 1;
        let n_traits = params.meta.n_traits();
        let mut writer =
            BufWriter::new(File::options().append(true).open(&self.path)?);
        write!(writer, "{}", self.index)?;
        for param_index in ParamIndex::all(n_traits) {
            write!(writer, "\t{}", params[param_index])?;
        }
        writeln!(writer)?;
        Ok(())
    }
}