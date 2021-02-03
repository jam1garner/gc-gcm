use thiserror::Error;
use std::io;

/// An error resulting from parsing or opening a GcmFile
#[derive(Error, Debug)]
pub enum GcmError {
    #[error("parsing error occurred")]
    ParseError(#[from] binread::Error),
    
    #[error("input/output error occurred")]
    IoError(#[from] io::Error),
}

