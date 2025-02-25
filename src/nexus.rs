use anyhow::{Context, Result};
use chrono::{FixedOffset, Local};
use hdf5::{
    types::{VarLenAscii, VarLenUnicode},
    File, Group,
};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct InspectResult {
    pub instrument: String,
    pub start_time: Option<chrono::DateTime<Local>>,
    pub modification_time: Option<chrono::DateTime<Local>>,
}

pub fn try_inspect_file(path: &Path) -> hdf5::Result<InspectResult> {
    let mtime = get_modification_time(path);

    let entry = File::open(path)?.group("entry")?;

    let start_time = load_datetime(&entry, "start_time").ok();

    let instrument = entry.group("instrument")?;
    let instrument_name = load_string(&instrument, "name")?;

    Ok(InspectResult {
        instrument: instrument_name,
        start_time: start_time.map(|t| t.into()),
        modification_time: mtime,
    })
}

fn load_string(base: &Group, name: &str) -> hdf5::Result<String> {
    let dataset = base.dataset(name)?;
    match dataset.read_scalar::<VarLenUnicode>() {
        Ok(val) => Ok(val.into()),
        Err(_) => Ok(dataset.read_scalar::<VarLenAscii>()?.into()),
    }
}

fn load_datetime(base: &Group, name: &str) -> Result<chrono::DateTime<FixedOffset>> {
    let time_str = load_string(base, name)?;
    chrono::DateTime::parse_from_rfc3339(&time_str).context("Failed to parse datetime")
}

fn get_modification_time(path: &Path) -> Option<chrono::DateTime<Local>> {
    let mtime = path.metadata().ok()?.modified().ok()?;
    Some(mtime.into())
}
