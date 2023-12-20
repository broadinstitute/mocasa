use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::error::{Error, ErrorKind};

pub(crate) mod names {
    pub(crate) const TRAIN: &str = "train";
    pub(crate) const CLASSIFY: &str = "classify";
}

#[derive(PartialEq, Clone, Copy)]
pub enum Action { Train, Classify }

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Train => { write!(f, "{}", names::TRAIN) }
            Action::Classify => { write!(f, "{}", names::CLASSIFY) }
        }
    }
}

impl FromStr for Action {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            names::TRAIN => { Ok(Action::Train) }
            names::CLASSIFY => { Ok(Action::Classify) }
            _ => { Err(Error::new(ErrorKind::Mocasa, s.to_string()))}
        }
    }
}