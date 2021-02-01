use structopt::StructOpt;
use std::path::PathBuf;

use gc_gcm::{GcmFile, DirEntry};

#[derive(StructOpt)]
enum Args {
    Extract {
        iso: PathBuf,
    },

    List {
        iso: PathBuf,
    },
}

fn list_entry<'a>(entry: DirEntry<'a>, depth: usize) {
    for _ in 0..depth {
        print!("    ");
    }

    println!("L {}", entry.entry_name());

    if let Some(entries) = entry.iter_dir() {
        for child in entries {
            list_entry(child, depth + 1)
        }
    }
}

fn list(path: PathBuf) {
    let iso = GcmFile::from_reader(&mut std::fs::File::open(&path).unwrap()).unwrap();

    for entry in iso.filesystem.iter_root() {
        list_entry(entry, 1)
    }
}

fn main() {
    let args = Args::from_args();

    match args {
        Args::Extract { iso } => todo!(),
        Args::List { iso } => list(iso),
    }
}
