use std::path::PathBuf;
use std::fs::{read_to_string, read_dir};
use std::mem::replace;

pub fn get_directory_contents(path: &PathBuf, dump: &mut Vec<PathBuf>) -> Result<usize, String> {
    let contents = match read_dir(path) {
        Ok(contents) => contents,
        Err(e) => return Err(format!("{e:?}"))
    };
    let _ = replace(dump, contents.filter_map(Result::ok).map(|x| x.path()).collect());
    Ok(dump.len())
}

pub fn get_raw_contents(path: &PathBuf) -> Vec<String> {
    let contents = match read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return vec![]
    };

    contents.lines().map(|x| x.to_string()).collect()
}
