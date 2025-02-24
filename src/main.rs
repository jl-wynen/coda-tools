mod coda;

use anyhow::{bail, Result};
use clap::{Args, Parser};
use colored::Colorize;
use hdf5::{types::VarLenUnicode, Dataset, File, Group};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Arguments {
    /// Concrete files or folder of files to inspect.
    paths: Vec<String>,

    #[command(flatten)]
    command: Command,

    /// Maximum number of files to inspect.
    #[arg(short, default_value_t = 10)]
    n: usize,

    /// Proposal number to scan files for.
    #[arg(long)]
    proposal: Option<String>,

    /// Year of the proposal to scan through, defaults to current.
    #[arg(long)]
    year: Option<i32>,
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
struct Command {
    #[arg(long)]
    list: bool,
    #[arg(long)]
    find: bool,
}

fn parse_arguments() -> Arguments {
    let mut args = Arguments::parse();
    if !args.command.list && !args.command.find {
        args.command.list = true;
    }
    args
}

fn open_dataset_at_path(base: &Group, path: &[&str]) -> hdf5::Result<Dataset> {
    if path.len() == 1 {
        base.dataset(path[0])
    } else {
        base.group(path[0])
            .and_then(|group| open_dataset_at_path(&group, &path[1..]))
    }
}

fn load_instrument_name(file: &File) -> hdf5::Result<String> {
    let name_dataset = open_dataset_at_path(file, &["entry", "instrument", "name"])?;
    Ok(name_dataset.read_scalar::<VarLenUnicode>()?.into())
}

fn try_inspect_file(path: &Path) -> hdf5::Result<()> {
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
    let start = if max_n > files.len() {
        0
    } else {
        files.len() - max_n
    };
    inspect_list_of_files(&files[start..]);
}

fn default_input_paths(args: &Arguments) -> Result<Vec<PathBuf>> {
    let proposal_number = match &args.proposal {
        Some(proposal) => proposal.clone(),
        None => match coda::find_proposal(args.year) {
            Ok(proposal_number) => proposal_number,
            Err(err) => {
                eprintln!("Failed to find a CODA proposal directory: {err}.");
                bail!(err);
            }
        },
    };
    Ok(vec![coda::coda_raw_dir(
        proposal_number.as_str(),
        args.year,
    )])
}

fn list_coda_files(input_paths: &[PathBuf], args: &Arguments) {
    if input_paths.len() > 1 || input_paths[0].is_file() {
        inspect_list_of_files(input_paths);
    } else {
        inspect_files_in_folder(input_paths[0].as_path(), args.n)
    }
}

fn find_coda_files(input_paths: &[PathBuf], args: &Arguments) {}

fn main() {
    let args = parse_arguments();

    let input_paths: Vec<_> = args.paths.iter().map(PathBuf::from).collect();
    let input_paths = if input_paths.is_empty() {
        if let Ok(paths) = default_input_paths(&args) {
            paths
        } else {
            eprintln!(
                "Unable to deduce paths to CODA data. Please provide a path or list of files."
            );
            return;
        }
    } else {
        input_paths
    };

    if args.command.list {
        list_coda_files(&input_paths, &args);
    } else if args.command.find {
        find_coda_files(&input_paths, &args);
    }
}
