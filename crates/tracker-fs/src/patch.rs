//! In-place source patching helpers for the writer.
//!
//! An edit rewrites only the affected byte span, leaving the rest of the source
//! byte-identical.

use std::ops::Range;
use std::path::Path;
use std::time::SystemTime;
use std::{fs, io};

/// Replace `source[range]` with `replacement`, returning the new string.
pub fn replace_range(source: impl AsRef<str>, range: Range<usize>, replacement: impl AsRef<str>) -> String {
    let (source, replacement) = (source.as_ref(), replacement.as_ref());
    let old_len = range.end - range.start;
    let mut out = String::with_capacity(source.len() - old_len + replacement.len());
    out.push_str(&source[..range.start]);
    out.push_str(replacement);
    out.push_str(&source[range.end..]);
    out
}

/// Captured file metadata (modification time + size) used to detect that a file
/// has not changed between reading and writing it.
#[derive(Clone, Copy)]
pub struct FileMetaSnapshot {
    modified: SystemTime,
    len: u64,
}

impl FileMetaSnapshot {
    /// Capture the current `mtime` + size of `path`. Call this BEFORE reading
    /// the file, so a modification between snapshot and read is not missed.
    pub fn capture(path: impl AsRef<Path>) -> io::Result<Self> {
        let meta = fs::metadata(path.as_ref())?;
        Ok(Self {
            modified: meta.modified()?,
            len: meta.len(),
        })
    }
}

/// Error if `path` no longer matches `snapshot` (the file changed since capture).
pub fn verify_unchanged(path: impl AsRef<Path>, snapshot: &FileMetaSnapshot) -> io::Result<()> {
    let path = path.as_ref();
    let meta = fs::metadata(path)?;
    if meta.modified()? != snapshot.modified || meta.len() != snapshot.len {
        return Err(io::Error::other(format!(
            "`{}` was modified since read; retry",
            path.display()
        )));
    }
    Ok(())
}

/// Verify the file is unchanged, then overwrite it with `new_content`.
pub fn write_if_unchanged(path: impl AsRef<Path>, snapshot: &FileMetaSnapshot, new_content: &str) -> io::Result<()> {
    let path = path.as_ref();
    verify_unchanged(path, snapshot)?;
    fs::write(path, new_content)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use temp_testdir::TempDir;

    use super::*;

    #[test]
    fn writes_when_unchanged() {
        let dir = TempDir::default();
        let path = dir.join("unchanged");
        fs::write(&path, "hello").unwrap();
        let snapshot = FileMetaSnapshot::capture(&path).unwrap();
        write_if_unchanged(&path, &snapshot, "world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "world");
    }

    #[test]
    fn refuses_and_preserves_when_modified() {
        let dir = TempDir::default();
        let path = dir.join("modified");
        fs::write(&path, "hello").unwrap();
        let snapshot = FileMetaSnapshot::capture(&path).unwrap();
        // external modification after snapshot (different size → reliably detected)
        fs::write(&path, "changed by someone else").unwrap();
        let result = write_if_unchanged(&path, &snapshot, "world");
        assert!(result.is_err(), "write must be refused after modification");
        // the external change is preserved, not overwritten
        assert_eq!(fs::read_to_string(&path).unwrap(), "changed by someone else");
    }
}
