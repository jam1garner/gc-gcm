use core::fmt;
use crate::{FsNode, FileSystem};

#[cfg(feature = "no_std")]
use crate::std::vec::Vec;

impl FileSystem {
    /// Iterates over the root of the filesystem
    pub fn iter_root<'a>(&'a self) -> impl Iterator<Item = DirEntry<'a>> + 'a {
        let files = &self.files;
        let mut entries = Vec::new();

        let mut entries_iter = self.files
            .iter()
            .enumerate()
            .map(|(i, node)| DirEntry { node, files, index: i + 1 });

        while let Some(entry) = entries_iter.next() {
            recursive_remove(&entry, &mut entries_iter);

            entries.push(entry);
        }

        entries.into_iter()
    }

    pub fn get_child(&self, child_name: &str) -> Option<DirEntry> {
        self.iter_root()
            .find(|entry| {
                match entry.node {
                    FsNode::Directory { name, .. } | FsNode::File { name, .. }
                        if name == child_name => true,
                    _ => false
                }
            })
    }
}

/// An entry representing a directory within the image's filesystem
#[derive(Clone, Copy)]
pub struct DirEntry<'a> {
    /// The `FsNode` of the current directory
    node: &'a FsNode,

    /// A backreference to all the files within the filesystem
    files: &'a [FsNode],

    /// The index of this directory's FsNode
    index: usize,
}

impl<'a> fmt::Debug for DirEntry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.node, f)
    }
}

fn recursive_remove<'a>(entry: &DirEntry, iter: &mut impl Iterator<Item = DirEntry<'a>>) {
    match entry.node {
        &FsNode::Directory { end_index, .. } => {
            for _ in 1..(end_index as usize - entry.index) {
                iter.next();
            }
        }
        _ => {}
    }
}

impl<'a> DirEntry<'a> {
    pub fn is_dir(&self) -> bool {
        match self.node {
            FsNode::Directory { .. } => true,
            FsNode::File { .. } => false,
        }
    } 

    pub fn is_file(&self) -> bool {
        match self.node {
            FsNode::Directory { .. } => false,
            FsNode::File { .. } => true,
        }
    }

    pub fn entry_name(&self) -> &'a str {
        match self.node {
            FsNode::File { name, .. } | FsNode::Directory { name, .. } => &name,
        }
    }

    pub fn iter_dir(&self) -> Option<impl Iterator<Item = DirEntry<'a>> + 'a> {
        match self.node {
            FsNode::Directory { end_index, .. } => {
                let files = self.files;
                let index = self.index;
                let end_index = *end_index as usize;

                let mut entries = Vec::new();
                let mut entries_iter = 
                    files[index..end_index - 1]
                        .iter()
                        .enumerate()
                        .map(|(i, node)| DirEntry { node, files, index: index + i + 1 });

                while let Some(entry) = entries_iter.next() {
                    recursive_remove(&entry, &mut entries_iter);
                    entries.push(entry);
                }

                Some(entries.into_iter())
            },
            FsNode::File { .. } => None,
        }
    }

    pub fn get_child(&self, child_name: &str) -> Option<DirEntry<'a>> {
        self.iter_dir()?
            .find(|entry| {
                match entry.node {
                    FsNode::Directory { name, .. } | FsNode::File { name, .. }
                        if name == child_name => true,
                    _ => false
                }
            })
    }

    pub fn as_file(&self) -> Option<File> {
        match self.node {
            &FsNode::File { offset, size, .. } => Some(File { offset, size }),
            FsNode::Directory { .. } => None
        }
    }
}

/// A file within the filesystem, representing the range of the data backing within the GCM image
/// itself
#[derive(Debug, Clone, Copy)]
pub struct File {
    pub offset: u32,
    pub size: u32
}
