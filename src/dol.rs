use crate::GcmError;
use core::mem::size_of;
use binread::{BinRead, BinReaderExt, io::{self, SeekFrom}, helpers::read_bytes};

#[cfg(feature = "no_std")]
use crate::std::vec::Vec;

const SECTION_COUNT: usize = 18;

/// The header of a dol executable, describing the 19 sections (including the bss) as well as the
/// entrypoint of the executable.
#[derive(BinRead, Debug)]
pub struct DolHeader {
    pub section_offsets: [u32; SECTION_COUNT],
    pub section_addresses: [u32; SECTION_COUNT],
    pub section_lengths: [u32; SECTION_COUNT],

    pub bss_address: u32,
    pub bss_length: u32,

    pub entrypoint: u32,
}

impl DolHeader {
    const SIZE: usize = (SECTION_COUNT * size_of::<u32>() * 3) + (size_of::<u32>() * 3);

    pub fn calculate_file_size(&self) -> usize {
        self.section_offsets
            .iter()
            .zip(self.section_lengths.iter())
            .map(|(start, len)| (start + len) as usize)
            .max()
            .unwrap()
    }
}

/// A dol ("Dolphin") executable file, used as the main executable of the gamecube
///
/// ```
/// use gc_gcm::DolFile;
///
/// let dol = DolFile::open("boot.dol").unwrap();
///
/// println!(".text size: {:#x?}", dol.header.section_lengths[0]);
/// ```
#[derive(BinRead)]
pub struct DolFile {
    pub header: DolHeader,

    #[br(seek_before = SeekFrom::Current(-(DolHeader::SIZE as i64)))]
    #[br(count = header.calculate_file_size())]
    #[br(parse_with = read_bytes)]
    pub raw_data: Vec<u8>,
}

impl DolFile {
    /// Parse a dol from a reader that implements `io::Read` and `io::Seek`
    pub fn from_reader<R>(reader: &mut R) -> Result<Self, GcmError>
        where R: io::Read + io::Seek,
    {
        Ok(reader.read_be()?)
    }
}

#[cfg(not(feature = "no_std"))]
use std::path::Path;

#[cfg(not(feature = "no_std"))]
impl DolFile {
    /// Open a file from a given bath as a DolFile.
    pub fn open<P>(path: P) -> Result<Self, GcmError>
        where P: AsRef<Path>,
    {
        let mut reader = std::io::BufReader::new(std::fs::File::open(path)?);
        Ok(reader.read_be()?)
    }
}

use core::fmt;

impl fmt::Debug for DolFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DolFile")
            .field("header", &self.header)
            .finish()
    }
}
