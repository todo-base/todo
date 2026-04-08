use std::path::{self, PathBuf};

#[derive(Debug, Default, Clone, Copy)]
pub enum TrackerType {
    #[default]
    Fs,
}

#[derive(Debug)]
pub struct Target<ID = String> {
    pub tracker: TrackerType,
    pub location: Location<ID>,
}

#[derive(Debug, Clone)]
pub enum Location<ID = String> {
    Path(PathBuf),
    Id(ID),
    Name(String),
    IdOrName(String),
}

impl<ID> Location<ID> {
    pub fn from_unknown(location: impl Into<String>) -> Self {
        let location = location.into();
        if location == "." || location == ".." || location.chars().any(path::is_separator) {
            Self::Path(PathBuf::from(location))
        } else {
            Self::IdOrName(location)
        }
    }
}
