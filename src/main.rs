use std::path::{Path, PathBuf};

use clap::Parser;
use colored::Colorize;
use hdf5::{types::VarLenUnicode, Dataset, File, Group, Result};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// Concrete files or folder of files to inspect.
    paths: Vec<String>,

    /// Maximum number of files to inspect.
    #[arg(short, default_value_t = 10)]
    n: usize,
}

fn open_dataset_at_path(base: &Group, path: &[&str]) -> Result<Dataset> {
    if path.len() == 1 {
        base.dataset(path[0])
    } else {
        base.group(path[0])
            .and_then(|group| open_dataset_at_path(&group, &path[1..]))
    }
}

fn load_instrument_name(file: &File) -> Result<String> {
    let name_dataset = open_dataset_at_path(file, &["entry", "instrument", "name"])?;
    Ok(name_dataset.read_scalar::<VarLenUnicode>()?.into())
}

fn try_inspect_file(path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let name = load_instrument_name(&file)?;
    println!("  Instrument: {}", name.blue().bold());
    Ok(())
}

fn inspect_file(path: &Path) {
    println!("{}:", path.to_str().unwrap().bold());
    match try_inspect_file(&PathBuf::from(path)) {
        Ok(()) => (),
        Err(err) => eprintln!("  Failed: {}", err),
    }
}

fn inspect_list_of_files(paths: &[PathBuf]) {
    for path in paths {
        inspect_file(path)
    }
}

fn inspect_files_in_folder(folder: &Path, max_n: usize) {
    let Ok(dir_iter) = folder.read_dir() else {
        eprintln!("Failed to read directory: {}", folder.display());
        return;
    };
    let mut files = Vec::new();
    for maybe_entry in dir_iter {
        let path = match maybe_entry {
            Ok(entry) => entry.path(),
            Err(err) => {
                eprintln!("Failed to read directory entry: {}", err);
                continue;
            }
        };
        if !path.is_file() {
            continue;
        }
        files.push(path);
    }

    files.sort();
    inspect_list_of_files(&files[max_n.min(files.len())..]);
}

fn main() {
    let args = Args::parse();

    let input_paths: Vec<_> = args.paths.iter().map(PathBuf::from).collect();

    if input_paths.is_empty() {
        inspect_files_in_folder(&std::env::current_dir().unwrap(), args.n)
    } else if input_paths.len() > 1 || input_paths[0].is_file() {
        inspect_list_of_files(&input_paths);
    } else {
        inspect_files_in_folder(input_paths[0].as_path(), args.n)
    }
}
