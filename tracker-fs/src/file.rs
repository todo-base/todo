use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::{DirEntry, WalkDir};

pub fn find_in_dir_and_parents(dir: impl AsRef<Path>, file_name: impl AsRef<Path>) -> Option<PathBuf> {
    let mut current_dir = dir.as_ref();
    loop {
        let file = current_dir.join(file_name.as_ref());
        if file.exists() {
            break Some(file);
        } else {
            current_dir = current_dir.parent()?;
        }
    }
}

pub fn find_by_name_part(dir: impl AsRef<Path>, file_name_part: impl AsRef<str>) -> Option<PathBuf> {
    for entry in WalkDir::new(dir)
        .max_depth(1)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if let Some(file_name) = path.file_name()
            && file_name.to_string_lossy().contains(file_name_part.as_ref())
        {
            return Some(path.to_path_buf());
        }
    }
    None
}

pub fn find_match_files(dir: impl AsRef<Path>, regex: &Regex) -> impl Iterator<Item = DirEntry> + '_ {
    WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let file_name = entry.file_name().to_string_lossy();
            regex.is_match(file_name.as_ref())
        })
}

pub fn find_by_regex(dir: impl AsRef<Path>, file_name_regex: &Regex) -> Option<PathBuf> {
    find_match_files(dir, file_name_regex)
        .next()
        .map(|entry| entry.into_path())
}
