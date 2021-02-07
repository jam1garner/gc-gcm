#[cfg(not(feature = "no_std"))]
use std::io;

/// An error resulting from parsing or opening a GcmFile
#[derive(Debug)]
pub enum GcmError {
    ParseError(binread::Error),

    #[cfg(not(feature = "no_std"))]
    IoError(std::io::Error),
}

#[cfg(not(feature = "no_std"))]
impl From<io::Error> for GcmError {
    fn from(err: io::Error) -> Self {
        GcmError::IoError(err)
    }
}

impl From<binread::Error> for GcmError {
    fn from(err: binread::Error) -> Self {
        GcmError::ParseError(err)
    }
}
