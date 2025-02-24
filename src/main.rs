mod coda;

use anyhow::{bail, Result};
use clap::Parser;
use colored::Colorize;
use hdf5::{types::VarLenUnicode, Dataset, File, Group};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Arguments {
    /// Files or folder of files to inspect.
    paths: Vec<String>,

    #[arg(short, long)]
    instrument: Option<String>,

    /// Maximum number of results to display.
    #[arg(short, default_value_t = 10)]
    n: usize,

    /// Maximum number of files to inspect.
    #[arg(long, default_value_t = 30)]
    max: usize,

    /// Proposal number to scan files for.
    #[arg(long)]
    proposal: Option<String>,

    /// Year of the proposal to scan through, defaults to current.
    #[arg(long)]
    year: Option<i32>,
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

#[derive(Clone, Debug)]
struct InspectResult {
    instrument: String,
}

impl InspectResult {
    fn report(&self) {
        println!("  Instrument: {}", self.instrument.blue().bold());
    }
}

fn matches_instrument(actual: &str, filter: &Option<String>) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    let actual = actual.to_lowercase();
    let filter = filter.to_lowercase();

    if filter == "tbl" && actual.contains("test beamline") {
        true
    } else {
        actual == filter
    }
}

fn try_inspect_file(path: &Path) -> hdf5::Result<InspectResult> {
    let file = File::open(path)?;
    let name = load_instrument_name(&file)?;
    Ok(InspectResult { instrument: name })
}

fn inspect_file(path: &Path, instrument: &Option<String>) -> bool {
    let result = try_inspect_file(path);
    if let Ok(r) = &result {
        if !matches_instrument(&r.instrument, instrument) {
            return false;
        }
    }

    println!("{}:", path.to_str().unwrap().bold());
    match result {
        Ok(result) => {
            result.report();
        }
        Err(err) => {
            eprintln!("  Failed: {}", err);
        }
    }
    true
}

fn inspect_list_of_files(paths: &[PathBuf], args: &Arguments) {
    let mut n_inspected = 0;
    for path in paths {
        if inspect_file(path, &args.instrument) {
            n_inspected += 1;
            if n_inspected >= args.n {
                break;
            }
        }
    }
}

fn get_files_in_folder(folder: &Path, max_n: usize) -> Vec<PathBuf> {
    let Ok(dir_iter) = folder.read_dir() else {
        eprintln!("Failed to read directory: {}", folder.display());
        return Vec::new();
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
    files[start..].to_vec()
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
    let input_paths = if input_paths.len() > 1 || input_paths[0].is_file() {
        input_paths.to_vec()
    } else {
        get_files_in_folder(input_paths[0].as_path(), args.max)
    };
    inspect_list_of_files(&input_paths, args);
}

fn main() {
    let args = Arguments::parse();

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

    list_coda_files(&input_paths, &args);
}
