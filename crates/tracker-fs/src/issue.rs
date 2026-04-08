use std::fs;
use std::io::{self, Read, Seek, Write};
use std::path::Path;

use todo_lib::issue::Issue;

use crate::Placement;

pub const MD_BLOCK_START: &str = "```md todo";
pub const MD_BLOCK_END: &str = "```";

pub trait SaveIssue {
    type Id;

    fn to_text(&self) -> String;
    fn add_first(&self, destination: &Placement<impl AsRef<Path>>) -> io::Result<()>;
    fn add_last(&self, destination: &Placement<impl AsRef<Path>>) -> io::Result<()>;
}

impl<ID> SaveIssue for Issue<ID> {
    type Id = ID;

    fn to_text(&self) -> String {
        let mut text = format!("- {}", self.name);
        for line in self.content.lines() {
            text.push_str("\n  ");
            text.push_str(line);
        }
        text
    }

    fn add_first(&self, destination: &Placement<impl AsRef<Path>>) -> io::Result<()> {
        if !destination.as_ref().as_ref().exists() {
            fs::File::create(destination.as_ref())?;
        }

        let text = self.to_text();

        match destination {
            Placement::WholeFile(path) => {
                let mut issues_content = fs::read_to_string(path)?;
                issues_content.insert(0, '\n');
                issues_content.insert_str(0, &text);

                fs::write(path, issues_content)?;
            },
            Placement::CodeBlockInFile(path) => {
                let mut manifest_content = fs::read_to_string(path)?;

                if let Some(start_range) = get_newline_mark_range(&manifest_content, MD_BLOCK_START, 0) {
                    let idx = manifest_content[start_range.1..]
                        .find('\n')
                        .map(|idx| idx + start_range.1)
                        .unwrap_or(manifest_content.len());
                    manifest_content.insert(idx, '\n');
                    manifest_content.insert_str(idx + 1, &text);
                } else {
                    if !manifest_content.is_empty() && !manifest_content.ends_with('\n') {
                        manifest_content.push('\n');
                    }
                    manifest_content.push_str(MD_BLOCK_START);
                    manifest_content.push('\n');
                    manifest_content.push_str(&text);
                    manifest_content.push('\n');
                    manifest_content.push_str(MD_BLOCK_END);
                    manifest_content.push('\n');
                }
                fs::write(path, manifest_content)?;
            },
        }
        Ok(())
    }

    fn add_last(&self, destination: &Placement<impl AsRef<Path>>) -> io::Result<()> {
        if !destination.as_ref().as_ref().exists() {
            fs::File::create(destination.as_ref())?;
        }

        let text = self.to_text();

        match destination {
            Placement::WholeFile(path) => {
                let mut file = fs::File::options().append(true).read(true).open(path)?;

                let file_size = std::fs::metadata(path)?.len();
                if file_size > 0 {
                    let mut reader = io::BufReader::new(&file);
                    reader.seek(io::SeekFrom::End(-1))?;

                    let mut last_ch = [0];
                    reader.read_exact(&mut last_ch)?;

                    if &last_ch != b"\n" {
                        file.write_all(b"\n")?;
                    }
                }

                file.write_all(text.as_bytes())?;
                file.write_all(b"\n")?;
            },
            Placement::CodeBlockInFile(path) => {
                let mut manifest_content = fs::read_to_string(path)?;

                if let Some(start_range) = get_newline_mark_range(&manifest_content, MD_BLOCK_START, 0) {
                    if let Some(end_range) = get_newline_mark_range(&manifest_content, MD_BLOCK_END, start_range.1 + 1)
                    {
                        manifest_content.insert(end_range.0, '\n');
                        manifest_content.insert_str(end_range.0, &text);
                    } else {
                        if !manifest_content.ends_with('\n') {
                            manifest_content.push('\n');
                        }
                        manifest_content.push_str(&text);
                        manifest_content.push('\n');
                    }
                } else {
                    if !manifest_content.is_empty() && !manifest_content.ends_with('\n') {
                        manifest_content.push('\n');
                    }
                    manifest_content.push_str(MD_BLOCK_START);
                    manifest_content.push('\n');
                    manifest_content.push_str(&text);
                    manifest_content.push('\n');
                    manifest_content.push_str(MD_BLOCK_END);
                    manifest_content.push('\n');
                }
                fs::write(path, manifest_content)?;
            },
        }
        Ok(())
    }
}

fn get_newline_mark_range(haystack: impl AsRef<str>, mark: impl AsRef<str>, from_idx: usize) -> Option<(usize, usize)> {
    let haystack = haystack.as_ref();
    let mark = mark.as_ref();

    for (start_idx, _) in haystack
        .match_indices(mark)
        .skip_while(|(start_idx, _)| *start_idx < from_idx)
    {
        if start_idx == 0 || haystack.as_bytes()[start_idx - 1] == b'\n' {
            let end_idx = start_idx + mark.len();
            if end_idx == haystack.len()
                || haystack[end_idx..]
                    .chars()
                    .next()
                    .map(|ch| ch.is_whitespace())
                    .unwrap_or(true)
            {
                return Some((start_idx, end_idx));
            }
        }
    }
    None
}
