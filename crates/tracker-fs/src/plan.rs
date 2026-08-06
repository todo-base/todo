use std::io;
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
    ID: HashedId + Clone + PartialEq + FromStr,
    GEN: IdGenerator<Id = ID> + Copy,
{
    type Id = ID;

    fn load(source: &Placement<impl AsRef<Path>>, id_generator: GEN) -> io::Result<Option<Plan<Self::Id>>> {
        let path = source.as_ref().as_ref();
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let plan_src: &str = match source {
            Placement::WholeFile(_) => content.as_str(),
            Placement::CodeBlockInFile(_) => extract_md_todo_block(&content).map(|(_, block)| block).unwrap_or(""),
        };
        let parsed = parse::parse::<ID, GEN>(plan_src, &id_generator);
        Ok(Some(parsed.plan))
    }
}

/// Extract the inner text of a ` ```md todo ` fenced block (for
/// [`Placement::CodeBlockInFile`]). Returns `None` if no such block is present.
/// Accounts for nested code fences inside the block.
pub fn extract_md_todo_block(content: &str) -> Option<(usize, &str)> {
    let start_prefix = format!("{MD_BLOCK_START} ");
    let mut offset = 0usize;
    let mut block_start: Option<usize> = None;
    let mut inner: usize = 0;
    for line in content.split_inclusive('\n') {
        let line_end = offset + line.len();
        let trimmed = line.trim_end();
        match block_start {
            None => {
                let lower = trimmed.trim_start().to_ascii_lowercase();
                if lower == MD_BLOCK_START || lower.starts_with(&start_prefix) {
                    block_start = Some(line_end);
                    inner = 0;
                }
            },
            Some(start) => {
                if trimmed.trim_start().starts_with(MD_BLOCK_END) {
                    if trimmed.chars().nth(3).map(|ch| !ch.is_whitespace()).unwrap_or(false) {
                        inner += 1;
                    } else if inner == 0 {
                        return Some((start, &content[start..offset]));
                    } else {
                        inner -= 1;
                    }
                }
            },
        }
        offset = line_end;
    }
    // unterminated block: take to end of content
    block_start.map(|start| (start, &content[start..]))
}
