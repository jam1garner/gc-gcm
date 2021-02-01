use thiserror::Error;

#[derive(Error, Debug)]
pub enum GcmError {
    #[error("parsing error occurred")]
    ParseError(#[from] binread::Error),
}

