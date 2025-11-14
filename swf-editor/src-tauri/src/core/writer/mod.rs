// SWF writer module - for saving modified SWF files
// TODO: Implement SWF writing

use crate::core::types::SWFFile;
use std::io;

pub fn write_swf(_swf: &SWFFile, _path: &str) -> Result<(), io::Error> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "SWF writing not yet implemented",
    ))
}
