use std::fmt::Display;
use std::io;
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;

use fs_err as fs;
use todo_lib::id::HashedId;
use todo_lib::plan::Plan;

use crate::Placement;
use crate::generator::IdGenerator;
use crate::issue::{MD_BLOCK_END, MD_BLOCK_START};

pub mod parse;

pub trait LoadProjectPlan<GEN> {
    type Id;

    fn load(source: &Placement<impl AsRef<Path>>, id_generator: GEN) -> io::Result<Option<Plan<Self::Id>>>;
}

impl<ID, GEN> LoadProjectPlan<GEN> for Plan<ID>
where
    ID: HashedId + Clone + PartialEq + Display + FromStr,
    GEN: IdGenerator<Id = ID> + Copy,
{
    type Id = ID;

    fn load(source: &Placement<impl AsRef<Path>>, id_generator: GEN) -> io::Result<Option<Plan<Self::Id>>> {
        let Some(plan_source) = PlanSource::read(source)? else {
            return Ok(None);
        };
        let parsed = parse::parse::<ID, GEN>(plan_source.text(), &id_generator)?;
        Ok(Some(parsed.plan))
    }
}

/// The file a plan lives in, together with the byte range its markdown occupies:
/// the whole file, or the inner text of the ` ```md todo ` block.
pub struct PlanSource {
    content: String,
    span: Range<usize>,
}

impl PlanSource {
    /// Read the file behind `source`; `None` when it does not exist. A file
    /// carrying no plan block yields an empty plan text rather than an error.
    pub fn read(source: &Placement<impl AsRef<Path>>) -> io::Result<Option<Self>> {
        let path = source.as_ref().as_ref();
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let span = match source {
            Placement::WholeFile(_) => 0..content.len(),
            Placement::CodeBlockInFile(_) => extract_md_todo_block(&content).unwrap_or(0..0),
        };
        Ok(Some(Self { content, span }))
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    /// The plan markdown itself — what [`parse`](parse::parse) reads.
    pub fn text(&self) -> &str {
        &self.content[self.span.clone()]
    }

    /// Map a range within [`text`](Self::text) onto the whole file.
    pub fn file_range(&self, range: &Range<usize>) -> Range<usize> {
        self.span.start + range.start..self.span.start + range.end
    }
}

/// Does `fence` open the plan block — ` ```md todo `, optionally followed by more
/// info-string words?
fn opens_md_todo_block(fence: &str) -> bool {
    let Some(marker) = fence.get(..MD_BLOCK_START.len()) else {
        return false;
    };
    marker.eq_ignore_ascii_case(MD_BLOCK_START)
        && matches!(fence[MD_BLOCK_START.len()..].chars().next(), None | Some(' '))
}

/// Byte range of the inner text of a ` ```md todo ` fenced block (for
/// [`Placement::CodeBlockInFile`]). Returns `None` if no such block is present.
/// Accounts for nested code fences inside the block, indented ones included.
pub fn extract_md_todo_block(content: &str) -> Option<Range<usize>> {
    let mut offset = 0usize;
    let mut block_start: Option<usize> = None;
    let mut inner: usize = 0;
    for line in content.split_inclusive('\n') {
        let line_end = offset + line.len();
        let fence = line.trim();
        match block_start {
            None => {
                if opens_md_todo_block(fence) {
                    block_start = Some(line_end);
                    inner = 0;
                }
            },
            Some(start) => {
                if let Some(info) = fence.strip_prefix(MD_BLOCK_END) {
                    // an info string means a nested block opens here rather than closing ours
                    if !info.is_empty() {
                        inner += 1;
                    } else if inner == 0 {
                        return Some(start..offset);
                    } else {
                        inner -= 1;
                    }
                }
            },
        }
        offset = line_end;
    }
    // unterminated block: take to end of content
    block_start.map(|start| start..content.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn block_of(content: &str) -> Option<&str> {
        extract_md_todo_block(content).map(|range| &content[range])
    }

    #[test]
    fn takes_the_block_body() {
        assert_eq!(block_of("# P\n\n```md todo\n- a\n```\n\nafter\n"), Some("- a\n"));
        assert_eq!(block_of("```MD TODO plan\n- a\n```\nafter\n"), Some("- a\n"));
        assert_eq!(block_of("# P\n\njust text\n"), None);
        assert_eq!(block_of("```md todoish\n- a\n```\n"), None);
    }

    #[test]
    fn indented_fences_do_not_end_the_block() {
        let content = "```md todo\n- a\n\n  ```rust\n  code\n  ```\n```\nafter\n";
        assert_eq!(block_of(content), Some("- a\n\n  ```rust\n  code\n  ```\n"));
        assert_eq!(block_of("```md todo\n- a\n  ```\n\nafter\n"), Some("- a\n"));
    }

    #[test]
    fn unterminated_block_runs_to_the_end() {
        assert_eq!(block_of("```md todo\n- a\n- b\n"), Some("- a\n- b\n"));
    }
}
