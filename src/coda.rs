use anyhow::Result;
use chrono::Datelike;
use std::path::PathBuf;

fn coda_base_dir(year: Option<i32>) -> PathBuf {
    let year = year.unwrap_or_else(|| chrono::Local::now().year());
    ["/ess", "data", "coda", year.to_string().as_str()]
        .iter()
        .collect()
}

pub fn coda_raw_dir(proposal_number: &str, year: Option<i32>) -> PathBuf {
    coda_base_dir(year).join(proposal_number).join("raw")
}

pub fn find_proposal(year: Option<i32>) -> Result<String> {
    let (path, _) = coda_base_dir(year)
        .read_dir()?
        .filter_map(|entry| entry.map(|e| e.path()).ok())
        .filter(|path| path.is_dir())
        .fold(
            (PathBuf::new(), std::time::SystemTime::UNIX_EPOCH),
            |(acc_path, acc_mtime), path| {
                let Ok(metadata) = path.join("raw").metadata() else {
                    return (acc_path, acc_mtime);
                };
                let mtime = metadata
                    .modified()
                    .expect("File modification time not available");
                if mtime > acc_mtime {
                    (path, mtime)
                } else {
                    (acc_path, acc_mtime)
                }
            },
        );
    Ok(path.file_name().unwrap().to_str().unwrap().to_string())
}
