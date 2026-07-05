use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

pub fn create_expected_by_ext(file_path: &Path, extension: &str) -> anyhow::Result<PathBuf> {
    // here we would be iterating over the dir entries, so it would always have a parent
    let parent = file_path.parent().unwrap();

    // also i guess it wouldnt end in '..'
    let file_name = file_path.file_name().unwrap();

    let new_dir = parent.join("expected");
    fs::create_dir_all(&new_dir)?;

    let mut new_file_name = OsString::from(file_name);
    new_file_name.push(extension);

    Ok(new_dir.join(new_file_name))
}
