//! In-place source patching helpers for the writer.
//!
//! An edit rewrites only the affected byte span, leaving the rest of the source
//! byte-identical.

use std::ops::Range;
use std::path::Path;
use std::time::SystemTime;
use std::{io, process};

use fs_err as fs;

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

/// Captured file metadata (modification time + size), a best-effort guard against
/// overwriting an edit made by someone else between reading a file and writing it
/// back. Two writes landing within one filesystem timestamp tick that leave the
/// size equal look unchanged, and the check cannot be atomic with the write that
/// follows it — it narrows the window rather than closing it.
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

    /// Size of the file when it was captured.
    pub fn size(&self) -> u64 {
        self.len
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

/// Verify the file is unchanged, then replace it with `new_content`.
pub fn write_if_unchanged(path: impl AsRef<Path>, snapshot: &FileMetaSnapshot, new_content: &str) -> io::Result<()> {
    let path = path.as_ref();
    verify_unchanged(path, snapshot)?;
    replace_file(path, new_content)
}

/// Write `content` to a sibling temp file and rename it over `path`, so a write
/// cut short by a crash or a full disk leaves the original intact instead of a
/// truncated file.
fn replace_file(path: &Path, content: &str) -> io::Result<()> {
    let Some(file_name) = path.file_name() else {
        return Err(io::Error::other(format!("`{}` is not a file", path.display())));
    };
    let mut temp_name = file_name.to_os_string();
    temp_name.push(format!(".{}.tmp", process::id()));
    let temp_path = path.with_file_name(temp_name);

    let replaced = fs::write(&temp_path, content)
        .and_then(|_| fs::set_permissions(&temp_path, fs::metadata(path)?.permissions()))
        .and_then(|_| fs::rename(&temp_path, path));
    if replaced.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    replaced
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

    #[test]
    fn leaves_no_temp_file_behind() {
        let dir = TempDir::default();
        let path = dir.join("kept");
        fs::write(&path, "hello").unwrap();
        let snapshot = FileMetaSnapshot::capture(&path).unwrap();

        write_if_unchanged(&path, &snapshot, "world").unwrap();
        fs::write(&path, "changed by someone else").unwrap();
        write_if_unchanged(&path, &snapshot, "world").unwrap_err();

        let left: Vec<_> = fs::read_dir(dir.as_ref())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(
            left,
            ["kept"],
            "neither a written nor a refused edit may leak a temp file"
        );
    }

    #[cfg(unix)]
    #[test]
    fn keeps_the_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::default();
        let path = dir.join("mode");
        fs::write(&path, "hello").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        let snapshot = FileMetaSnapshot::capture(&path).unwrap();

        write_if_unchanged(&path, &snapshot, "world").unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o640, "replacing the file must not widen its mode");
    }
}
