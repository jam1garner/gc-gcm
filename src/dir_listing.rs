use core::fmt;
use crate::{FsNode, FileSystem};

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
}

pub struct DirEntry<'a> {
    node: &'a FsNode,
    files: &'a [FsNode],
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

    pub fn as_file(&self) -> Option<File> {
        match self.node {
            &FsNode::File { offset, size, .. } => Some(File { offset, size }),
            FsNode::Directory { .. } => None
        }
    }
}

pub struct File {
    pub offset: u32,
    pub size: u32
}
