use std::error;
use std::fmt;
use std::io::Write;

pub enum InitResult {
    GlifOk(String, Box<dyn Write>),
    GlifStdoutOk(Box<dyn Write>),
    UfoOk(std::path::PathBuf),
    InitErr(InitError),
}

impl std::fmt::Debug for InitResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            GlifOk(s, _) => write!(f, "GlifOk({})", &s),
            GlifStdoutOk(..) => write!(f, "GlifStdoutOk(STDOUT)"),
            UfoOk(pb) => write!(f, "UfoOk({:?})", &pb),
            InitErr(e) => write!(f, "InitError({:?})", e),
        }
    }
}

use InitResult::*;

impl From<InitResult> for Result<(), InitError> {
    fn from(ir: InitResult) -> Self {
        match ir {
            GlifOk(..) | GlifStdoutOk(..) | UfoOk(..) => Ok(()),
            InitErr(e) => Err(e),
        }
    }
}

#[derive(Debug)]
pub enum InitError {
    NoCommand,
    FailedUFO,
    FailedGlif,
}

impl InitError {
    fn desc(&self) -> &'static str {
        match self {
            InitError::NoCommand => "No command",
            InitError::FailedUFO => "Failed to write out UFO font",
            InitError::FailedGlif => "Failed to write .glif file",
        }
    }
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ({})", self, self.desc())
    }
}

impl error::Error for InitError {}
