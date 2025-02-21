use hdf5::{types::VarLenUnicode, Dataset, File, Group, Result};

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

fn main() -> Result<()> {
    let file = File::open("data/994160_00041231.hdf")?;
    let name = load_instrument_name(&file)?;
    dbg!(name);
    Ok(())
}
