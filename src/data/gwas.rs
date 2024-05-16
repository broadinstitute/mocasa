use std::io::{BufRead, Lines};
use serde::{Deserialize, Serialize};
use crate::data;
use crate::error::Error;

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct GwasCols {
    pub(crate) id: String,
    pub(crate) effect: String,
    pub(crate) se: String
}

pub(crate) mod default_cols {
    pub(crate) const VAR_ID: &str = "VAR_ID";
    pub(crate) const BETA: &str = "BETA";
    pub(crate) const SE: &str = "SE";
}

pub(crate) struct GwasReader<R: BufRead> {
    lines: Lines<R>,
    delim: char,
    i_var_id: usize,
    i_beta: usize,
    i_se: usize,
    cols: GwasCols
}

pub(crate) struct GwasRecord {
    pub(crate) var_id: String,
    pub(crate) beta: f64,
    pub(crate) se: f64,
}

impl Default for GwasCols {
    fn default() -> Self {
        let id = default_cols::VAR_ID.to_string();
        let effect =  default_cols::BETA.to_string();
        let se = default_cols::SE.to_string();
        GwasCols { id, effect, se }
    }
}

fn get_delim(header: &str) -> Result<char, Error> {
    let mut delim: Result<char, Error> = Err(Error::from(data::MISSING_DELIM_MSG));
    for c in data::DELIM_LIST {
        if header.contains(*c) {
            delim = Ok(*c);
            break;
        }
    }
    delim
}

impl<R: BufRead> GwasReader<R> {

    pub(crate) fn new(reader: R, cols: GwasCols)
                      -> Result<GwasReader<R>, Error> {
        let mut lines = reader.lines();
        let header =
            lines.next().ok_or_else(|| Error::from("File is empty"))??;
        let mut i_var_id_opt: Option<usize> = None;
        let mut i_beta_opt: Option<usize> = None;
        let mut i_se_opt: Option<usize> = None;
        let delim = get_delim(&header)?;
        for (i, col) in header.split(delim).enumerate() {
            if col == cols.id {
                i_var_id_opt = Some(i)
            } else if col == cols.effect {
                i_beta_opt = Some(i)
            } else if col == cols.se {
                i_se_opt = Some(i)
            }
        }
        let i_var_id =
            i_var_id_opt.ok_or_else(|| Error::from(format!("No {} column", cols.id)))?;
        let i_beta =
            i_beta_opt.ok_or_else(|| Error::from(format!("No {} column", cols.effect)))?;
        let i_se =
            i_se_opt.ok_or_else(|| Error::from(format!("No {} column", cols.se)))?;
        Ok(GwasReader { lines, delim, i_var_id, i_beta, i_se, cols })
    }
    pub(crate) fn parse_line(&mut self, line: &str) -> Result<GwasRecord, Error> {
        let mut var_id_opt: Option<String> = None;
        let mut beta_opt: Option<f64> = None;
        let mut se_opt: Option<f64> = None;
        for (i, part) in line.split(self.delim).enumerate() {
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
        let var_id = var_id_opt.ok_or_else(|| missing_value_error(&self.cols.id))?;
        let beta = beta_opt.ok_or_else(||  missing_value_error(&self.cols.effect))?;
        let se = se_opt.ok_or_else(||  missing_value_error(&self.cols.se))?;
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

fn missing_value_error(col: &str) -> Error {
    Error::from(format!("Missing value for '{}'.", col))
}