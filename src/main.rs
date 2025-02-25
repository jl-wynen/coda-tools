mod coda;
mod nexus;

use anyhow::{bail, Result};
use chrono::SecondsFormat;
use clap::Parser;
use colored::Colorize;
use std::path::{Path, PathBuf};

/// Inspect NeXus files created by CODA.
///
/// By default, it will find the current CODA proposal and raw data folder
/// by inspecting modification times of files.
/// Specify a path to a directory to only search within that directory.
/// Or specify a list of files to only inspect those files.
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
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

    /// Enable extra output.
    #[arg(short, long)]
    verbose: bool,
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

fn format_maybe_time(time: Option<chrono::DateTime<chrono::Local>>) -> String {
    time.map(|t| t.to_rfc3339_opts(SecondsFormat::Secs, true))
        .unwrap_or_else(|| "?".to_string())
}

fn report_on_file(results: &nexus::InspectResult, verbose: bool) {
    println!("  Instrument: {}", results.instrument.blue().bold());
    if verbose {
        println!("  Start time: {}", format_maybe_time(results.start_time));
        println!(
            "  Modified:   {}",
            format_maybe_time(results.modification_time)
        );
    }
}

fn inspect_file(path: &Path, instrument: &Option<String>, verbose: bool) -> bool {
    let result = nexus::try_inspect_file(path);
    if let Ok(r) = &result {
        if !matches_instrument(&r.instrument, instrument) {
            return false;
        }
    }

    println!("{}:", path.to_str().unwrap().bold());
    match result {
        Ok(result) => {
            report_on_file(&result, verbose);
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
        if inspect_file(path, &args.instrument, args.verbose) {
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
