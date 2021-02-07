use memmap::Mmap;
use rayon::prelude::*;
use structopt::StructOpt;
use gc_gcm::{GcmFile, DirEntry, File};

use std::fs;
use std::path::{Path, PathBuf};

#[derive(StructOpt)]
enum Args {
    #[structopt(about = "Extract a GCM ISO to a given folder")]
    Extract {
        #[structopt(help = "Path to ISO to extract")]
        iso: PathBuf,

        #[structopt(help = "Path to folder to extract root of ISO to. Will be created if missing")]
        to: PathBuf,

        #[structopt(long, help = "Run the extraction single threaded. Recommended for HDDs and other physical media.")]
        single_thread: bool
    },

    #[structopt(about = "List the file tree of the given GCM ISO")]
    Tree {
        #[structopt(help = "Path to ISO to list")]
        iso: PathBuf,
    },

    #[structopt(about = "Give a brief explanation about the GCM format")]
    Explain,
}

fn list_recursive<'a>(entry: DirEntry<'a>, depth: usize) {
    for _ in 0..depth {
        print!("    ");
    }

    println!("L {}", entry.entry_name());

    if let Some(entries) = entry.iter_dir() {
        for child in entries {
            list_recursive(child, depth + 1)
        }
    }
}

fn tree(path: PathBuf) {
    let iso = GcmFile::from_reader(&mut std::fs::File::open(&path).unwrap()).unwrap();

    for entry in iso.filesystem.iter_root() {
        list_recursive(entry, 1)
    }
}

fn extract_entry<'a>(entry: DirEntry<'a>, path: &Path, files: &mut Vec<(PathBuf, File)>) {
    if let Some(file_data) = entry.as_file() {
        files.push((path.join(entry.entry_name()), file_data));
    } else if let Some(entries) = entry.iter_dir() {
        let dir_path = path.join(entry.entry_name());
        let _ = fs::create_dir_all(&dir_path);
        for child in entries {
            extract_entry(child, &dir_path, files)
        }
    }
}

fn extract(path: PathBuf, to: &Path, single_thread: bool) {
    let file = std::fs::File::open(&path).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let mut cursor = binread::io::Cursor::new(&mmap[..]);
    let iso = GcmFile::from_reader(&mut cursor).unwrap();

    let mut files = Vec::new();
    for entry in iso.filesystem.iter_root() {
        extract_entry(entry, to, &mut files)
    }

    if let Err(err) = fs::write(
        to.join("boot.dol"),
        &iso.dol.raw_data
    ) {
        println!("Path: boot.dol");
        println!("Error: {:?}", err);
        println!();
    };

    let iso = &mmap[..];

    let extract_file = |(path, file): &(PathBuf, File)| {
        let start = file.offset as usize;
        let end = start + (file.size as usize);
        let file = &iso[start..end];
        
        if let Err(err) = fs::write(path, file) {
            println!("Path: {}", path.display());
            println!("Error: {:?}", err);
            println!();
        }
    };

    if single_thread {
        files.iter().for_each(extract_file);
    } else {
        files.par_iter().for_each(extract_file);
    }
}

fn main() {
    let args = Args::from_args();

    match args {
        Args::Extract { iso, to, single_thread } => extract(iso, &to, single_thread),
        Args::Tree { iso } => tree(iso),
        Args::Explain => println!("{}", include_str!("explain.txt"))
    }
}
