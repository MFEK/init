use std::error;
use std::fmt;

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
