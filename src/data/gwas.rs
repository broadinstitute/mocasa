use std::io::{BufRead, Lines};
use crate::data::GwasCols;
use crate::error::Error;

pub(crate) mod cols_old {
    pub(crate) const VAR_ID: &str = "VAR_ID";
    pub(crate) const BETA: &str = "BETA";
    pub(crate) const SE: &str = "SE";
}

const DELIM: char = ';';

pub(crate) struct GwasReader<R: BufRead> {
    lines: Lines<R>,
    i_var_id: usize,
    i_beta: usize,
    i_se: usize,
}

pub(crate) struct GwasRecord {
    pub(crate) var_id: String,
    pub(crate) beta: f64,
    pub(crate) se: f64,
}

impl<R: BufRead> GwasReader<R> {
    pub(crate) fn new(reader: R, cols: &GwasCols)
                      -> Result<GwasReader<R>, Error> {
        let mut lines = reader.lines();
        let header =
            lines.next().ok_or_else(|| Error::from("File is empty"))??;
        let mut i_var_id_opt: Option<usize> = None;
        let mut i_beta_opt: Option<usize> = None;
        let mut i_se_opt: Option<usize> = None;
        for (i, col) in header.split(DELIM).enumerate() {
            if col == cols.id {
                i_var_id_opt = Some(i)
            } else if col == cols.effect {
                i_beta_opt = Some(i)
            } else if col == cols.se {
                i_se_opt = Some(i)
            }
        }
        let i_var_id =
            i_var_id_opt.ok_or_else(|| Error::from("No VAR_ID column"))?;
        let i_beta =
            i_beta_opt.ok_or_else(|| Error::from("No BETA column"))?;
        let i_se =
            i_se_opt.ok_or_else(|| Error::from("No SE column"))?;
        Ok(GwasReader { lines, i_var_id, i_beta, i_se })
    }
    pub(crate) fn parse_line(&mut self, line: &str) -> Result<GwasRecord, Error> {
        let mut var_id_opt: Option<String> = None;
        let mut beta_opt: Option<f64> = None;
        let mut se_opt: Option<f64> = None;
        for (i, part) in line.split(DELIM).enumerate() {
            if i == self.i_var_id {
                var_id_opt = Some(part.to_string())
            } else if i == self.i_beta {
                beta_opt = Some(part.parse::<f64>()?)
            } else if i == self.i_se {
                se_opt = Some(part.parse::<f64>()?)
            }
            if var_id_opt.is_some() && beta_opt.is_some() && se_opt.is_some() {
                break;
            }
        }
        let var_id =
            var_id_opt.ok_or_else(|| Error::from("Missing value for VAR_ID"))?;
        let beta = beta_opt.ok_or_else(|| Error::from("Missing value for BETA"))?;
        let se = se_opt.ok_or_else(|| Error::from("Missing value for SE"))?;
        Ok(GwasRecord { var_id, beta, se })
    }
}

impl<R: BufRead> Iterator for GwasReader<R> {
    type Item = Result<GwasRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|line|
            line.map_err(Error::from).and_then(|line| self.parse_line(&line)
            ))
    }
}