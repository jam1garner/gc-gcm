use binread::{derive_binread, BinRead, BinReaderExt, NullString, io::{self, SeekFrom}};
use binread::file_ptr::{FilePtr, IntoSeekFrom};
use core::fmt;

#[derive(BinRead, Debug)]
#[br(big)]
struct GcmTop (
    // magic value at 0x1c
    #[br(seek_before = SeekFrom::Start(0x1c))]
    pub GcmFile,
);

#[derive(BinRead)]
struct GameId(pub [u8; 6]);

#[derive_binread]
#[derive(Debug)]
#[br(magic = 0xc2339f3d_u32)]
pub struct GcmFile {
    #[br(seek_before = SeekFrom::Start(0))]
    game_id: GameId,
    disc_number: u8,
    revision: u8,
    
    #[br(seek_before = SeekFrom::Start(0x20))]
    #[br(map = NullString::into_string)]
    internal_name: String,

    // just gonna skip debug stuff
    #[br(seek_before = SeekFrom::Start(0x420))]
    dol_offset: u32,
    
    fs_offset: u32,
    fs_size: u32,
    max_fs_size: u32,
    
    #[br(seek_before = SeekFrom::Start(fs_offset as u64))]
    #[br(args(fs_offset, fs_size))]
    filesystem: FileSystem,
}

#[derive(BinRead, Debug)]
#[br(import(offset: u32, size: u32))]
struct FileSystem {
    root: RootNode,

    #[br(args(
        offset as u64, // root offset
        (offset + (root.total_node_count * FsNode::SIZE)) as u64 // name offset (after all entries)
    ))]
    #[br(count = root.total_node_count - 1)]
    files: Vec<FsNode>,
}

#[derive(BinRead, Debug)]
#[br(magic = 1u8)]
struct RootNode {
    #[br(map = U24::into)]
    root_start_index: u32,
    name_offset: u32,
    total_node_count: u32,
}

type FilePtr24<T> = FilePtr<U24, T>;

#[br(import(root_offset: u64, name_offset: u64))]
#[derive(BinRead, Debug)]
enum FsNode {
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
        dir_start_index: u32,
        child_count: u32,
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
        match std::str::from_utf8(&self.0[..]) {
            Ok(id) => fmt::Debug::fmt(id, f),
            Err(_) => write!(f, "GameId({:02x?})", &self.0[..])
        }
    }
}

mod error;
pub use error::GcmError;

impl GcmFile {
    pub fn from_reader<R>(reader: &mut R) -> Result<Self, GcmError>
        where R: io::Read + io::Seek,
    {
        Ok(reader.read_be::<GcmTop>()?.0)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::*;

    #[test]
    fn try_parse() {
        let file = GcmFile::from_reader(&mut File::open("/home/jam/dev/melee/melee.iso").unwrap()).unwrap();
        dbg!(file);
    }
}
