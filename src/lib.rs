//! A library for working with GCM/ISO files (raw bit-for-bit disk images) for the Nintendo
//! GameCube.
//!
//! Features:
//!
//! * GCM parser
//!     * Disk metadata (game_id, internal name, etc.)
//!     * Offsets to various sections of the GCM
//! * GameCube filesystem parser
//!     * Raw access to filesystem structures
//!     * Iterate over directories in a high-level manner
//!     * Information about the storage of files, allowing extraction
//! * DOL executable parser
//!     * The main executable for the game
//!     * Allows for extraction or loading into memory
//!     * Supports parsing extracted DOL files as well
//!
//! ```
//! use gc_gcm::GcmFile;
//!
//! let iso = GcmFile::open("melee.iso").unwrap();
//!
//! println!("Name of game: {:?}", iso.internal_name);
//! println!("Size of executable: {:x?}", iso.dol.raw_data.len());
//! println!(
//!     "Number of files: {}",
//!     iso.filesystem.files
//!         .iter()
//!         .filter(|entry| matches!(entry, gc_gcm::FsNode::File { .. }))
//!         .count()
//! );
//! ```
//!
//! Output:
//!
//! ```text
//! Name of game: "Super Smash Bros Melee"
//! Size of executable: 4385e0
//! Number of files: 1209
//! ```

#![cfg_attr(feature = "no_std", no_std)]
use binread::{derive_binread, BinRead, BinReaderExt, NullString, io::{self, SeekFrom}};
use binread::file_ptr::{FilePtr, FilePtr32, IntoSeekFrom};
use core::fmt;

#[cfg(feature = "no_std")]
mod std;

#[cfg(feature = "no_std")]
use crate::std::{string::String, vec::Vec};

/// Top-level view of a GCM file
#[derive(BinRead, Debug)]
#[br(big)]
struct GcmTop (
    // magic value at 0x1c
    #[br(seek_before = SeekFrom::Start(0x1c))]
    pub GcmFile,
);

/// A 6-character ID for a game
#[derive(BinRead)]
pub struct GameId(pub [u8; 6]);

/// A parsed GCM/ISO file
#[derive_binread]
#[derive(Debug)]
#[br(magic = 0xc2339f3d_u32)]
pub struct GcmFile {
    #[br(seek_before = SeekFrom::Start(0))]
    pub game_id: GameId,
    pub disc_number: u8,
    pub revision: u8,
    
    #[br(seek_before = SeekFrom::Start(0x20))]
    #[br(map = NullString::into_string)]
    pub internal_name: String,
    
    // just gonna skip debug stuff

    #[br(seek_before = SeekFrom::Start(0x420))]
    pub dol_offset: u32,

    #[br(seek_before = SeekFrom::Start(0x420))]
    #[br(parse_with = FilePtr32::parse)]
    pub dol: DolFile,
    
    fs_offset: u32,
    fs_size: u32,
    max_fs_size: u32,
    
    #[br(seek_before = SeekFrom::Start(fs_offset as u64))]
    #[br(args(fs_offset, fs_size))]
    pub filesystem: FileSystem,
}

/// The parsed GCM filesystem
#[derive(BinRead, Debug)]
#[br(import(offset: u32, size: u32))]
pub struct FileSystem {
    pub root: RootNode,

    #[br(args(
        offset as u64, // root offset
        (offset + (root.total_node_count * FsNode::SIZE)) as u64 // name offset (after all entries)
    ))]
    #[br(count = root.total_node_count - 1)]
    pub files: Vec<FsNode>,
}

/// The root node of the filesystem, under which all the other nodes fall
#[derive(BinRead, Debug)]
#[br(magic = 1u8)]
pub struct RootNode {
    #[br(map = U24::into)]
    pub name_offset: u32,
    pub node_start_index: u32,
    pub total_node_count: u32,
}

type FilePtr24<T> = FilePtr<U24, T>;

/// A given parsed node in the filesystem
#[br(import(root_offset: u64, name_offset: u64))]
#[derive(BinRead, Debug)]
pub enum FsNode {
    #[br(magic = 0u8)]
    File {
        #[br(offset = name_offset)]
        #[br(parse_with = FilePtr24::parse)]
        #[br(map = NullString::into_string)]
        name: String,
        offset: u32,
        size: u32,
    },

    #[br(magic = 1u8)]
    Directory {
        #[br(offset = name_offset)]
        #[br(parse_with = FilePtr24::parse)]
        #[br(map = NullString::into_string)]
        name: String,
        parent_index: u32,
        end_index: u32,
    },
}

impl FsNode {
    const SIZE: u32 = 0xC;
}

#[derive(BinRead, Clone, Copy)]
struct U24([u8; 3]);

impl IntoSeekFrom for U24 {
    fn into_seek_from(self) -> SeekFrom {
        u32::from(self).into_seek_from()
    }
}

impl From<U24> for u32 {
    fn from(U24(x): U24) -> Self {
        u32::from_be_bytes([0, x[0], x[1], x[2]])
    }
}

impl fmt::Debug for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match core::str::from_utf8(&self.0[..]) {
            Ok(id) => fmt::Debug::fmt(id, f),
            Err(_) => write!(f, "GameId({:02x?})", &self.0[..])
        }
    }
}

mod dol;
mod error;
mod dir_listing;
pub use error::GcmError;
pub use dir_listing::*;
pub use dol::*;

impl GcmFile {
    /// Parse a GcmFile from a reader that implements `io::Read` and `io::Seek`
    pub fn from_reader<R>(reader: &mut R) -> Result<Self, GcmError>
        where R: io::Read + io::Seek,
    {
        Ok(reader.read_be::<GcmTop>()?.0)
    }
}

#[cfg(not(feature = "no_std"))]
use ::std::path::Path;

#[cfg(not(feature = "no_std"))]
impl GcmFile {
    /// Open a file from a given bath as a GcmFile.
    pub fn open<P>(path: P) -> Result<Self, GcmError>
        where P: AsRef<Path>,
    {
        let mut reader = ::std::io::BufReader::new(::std::fs::File::open(path)?);
        Ok(reader.read_be::<GcmTop>()?.0)
    }
}
