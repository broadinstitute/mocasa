use std::fmt::{Debug, Display, Formatter};

mod names {
    pub const UNKNOWN_ACTION: &str = "unknown action";
}

pub enum ErrorKind {
  UnknownAction
}

pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::UnknownAction => { names::UNKNOWN_ACTION }
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, message: String) -> Error {
        Error { kind, message }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind.as_str(), self.message)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind.as_str(), self.message)
    }
}

impl std::error::Error for Error {
}