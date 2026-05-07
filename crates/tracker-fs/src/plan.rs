use std::io::{self, BufRead};
use std::mem;
use std::path::Path;
use std::str::FromStr;

use either::Either;
use fs_err as fs;
use indexmap::IndexMap;
use todo_lib::id::HashedId;
use todo_lib::issue::{Issue, IssueContent};
use todo_lib::plan::Plan;

use self::parse::Item;
use crate::Placement;
use crate::generator::IdGenerator;
use crate::issue::{MD_BLOCK_END, MD_BLOCK_START};

pub mod parse;

pub trait LoadProjectPlan<GEN> {
    type Id;

    fn load(source: &Placement<impl AsRef<Path>>, id_generator: GEN) -> io::Result<Option<Plan<Self::Id>>>;

    fn load_to_lines(
        source: &Placement<impl AsRef<Path>>,
    ) -> io::Result<impl IntoIterator<Item = (usize, io::Result<String>)>>;

    fn load_from_lines(
        lines: impl IntoIterator<Item = (usize, io::Result<String>)>,
        id_generator: GEN,
    ) -> io::Result<Plan<Self::Id>>;
}

impl<ID, GEN> LoadProjectPlan<GEN> for Plan<ID>
where
    ID: HashedId + Clone + FromStr,
    GEN: IdGenerator<Id = ID> + Copy,
{
    type Id = ID;

    fn load(source: &Placement<impl AsRef<Path>>, id_generator: GEN) -> io::Result<Option<Plan<Self::Id>>> {
        if source.as_ref().as_ref().exists() {
            let lines = <Self as LoadProjectPlan<GEN>>::load_to_lines(source)?;
            let plan = Self::load_from_lines(lines, id_generator)?;

            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }

    fn load_to_lines(
        source: &Placement<impl AsRef<Path>>,
    ) -> io::Result<impl IntoIterator<Item = (usize, io::Result<String>)>> {
        let lines = match source {
            Placement::WholeFile(path) => {
                let file = fs::File::open(path.as_ref())?;
                Either::Left(io::BufReader::new(file).lines().enumerate())
            },
            Placement::CodeBlockInFile(path) => {
                let file = fs::File::open(path.as_ref())?;

                let mut in_block = false;
                let mut inner_blocks: usize = 0;
                Either::Right(io::BufReader::new(file).lines().enumerate().filter(move |(_, line)| {
                    let Ok(line) = line else {
                        return false;
                    };

                    if !in_block {
                        if let Some(start) = line.get(..line.len().min(MD_BLOCK_START.len() + 1))
                            && start.trim().to_lowercase() == MD_BLOCK_START
                        {
                            in_block = true;
                        }
                        false
                    } else {
                        let line = line.trim_end();
                        if line.starts_with(MD_BLOCK_END) {
                            if line.chars().nth(3).map(|ch| !ch.is_whitespace()).unwrap_or(false) {
                                inner_blocks += 1;
                            } else if inner_blocks == 0 {
                                in_block = false;
                            } else {
                                inner_blocks -= 1;
                            }
                        }
                        in_block
                    }
                }))
            },
        };
        Ok(lines)
    }

    fn load_from_lines(
        lines: impl IntoIterator<Item = (usize, io::Result<String>)>,
        id_generator: GEN,
    ) -> io::Result<Plan<Self::Id>> {
        let mut planned = Plan::<ID>::new();
        let mut last = Last::<ID>::new();

        for (line_idx, line) in lines {
            let line = line?;

            match Item::parse(line, id_generator) {
                (Item::Empty, _) => {
                    if let Line::Separator = last.line {
                        planned.add_issues(last.extract_issues());
                    } else {
                        last.intermediate_blank.push('\n');
                    }
                    last.line = Line::Empty;
                },
                (Item::Separator, _) => {
                    if let Line::Empty = last.line {
                        last.line = Line::Separator;
                    } else {
                        last.line = Line::Other;
                    }
                },
                (Item::Issue(mut issue), issue_level) => {
                    if issue_level == last.issue_level {
                        if let (Line::Issue | Line::Description | Line::Empty, Some(id)) =
                            (last.line, last.issue_parent_id.clone())
                        {
                            last.parsed_issues
                                .get_mut(&id)
                                .expect("issue for previous parent must parsed")
                                .subissues
                                .insert(issue.id.clone());
                            issue.parent_id = Some(id);
                        }
                    } else if issue_level != 0 {
                        let parent_issue = if issue_level == last.issue_level + 1 {
                            last.parsed_issues.last_mut().map(|(_, last_issue)| last_issue)
                        } else if issue_level < last.issue_level {
                            last.find_parent(issue_level)
                        } else {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "in line {line_idx}: issue level = {issue_level} is greater than previous issue level + 1 = {}",
                                    last.issue_level + 1
                                ),
                            ));
                        }.ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "in line {line_idx}: parent not found for issue level = {issue_level}"
                                ),
                            )
                        })?;

                        parent_issue.subissues.insert(issue.id.clone());
                        issue.parent_id = Some(parent_issue.id.clone());
                    }

                    last.insert_issue(issue, issue_level);
                    last.line = Line::Issue;
                },
                (Item::Milestone(mut milestone), _milestone_level) => {
                    milestone.needed_issues.extend(last.parsed_issues.keys().cloned());
                    planned.add_issues(last.extract_issues());
                    planned.add_milestone(milestone);
                    last.line = Line::Milestone;
                },
                (Item::Text(text), text_level) => {
                    if matches!(last.line, Line::Issue | Line::Description | Line::Empty)
                        && text_level > last.issue_level
                    {
                        let mut padding = (last.issue_level + 1) * 2;
                        let description_line = text.trim_start_matches(|ch| {
                            if padding > 0 && ch == ' ' {
                                true
                            } else {
                                padding -= 1;
                                false
                            }
                        });

                        let target_issue = last
                            .parsed_issues
                            .last_mut()
                            .expect("issue for description must exist")
                            .1;

                        match &mut target_issue.content {
                            val @ IssueContent::Empty => {
                                *val = IssueContent::Inline(description_line.into());
                            },
                            IssueContent::Linked { note: val @ None, .. } => {
                                *val = Some(description_line.into());
                            },
                            IssueContent::Inline(content)
                            | IssueContent::Linked {
                                note: Some(content), ..
                            } => {
                                if !content.is_empty() {
                                    content.push('\n');
                                }
                                if !last.intermediate_blank.is_empty() {
                                    let blank = mem::take(&mut last.intermediate_blank);
                                    content.push_str(&blank);
                                }

                                content.push_str(description_line);
                            },
                        }

                        last.line = Line::Description;
                    } else if text.trim().is_empty() {
                        last.intermediate_blank.push_str(&text);
                        last.intermediate_blank.push('\n');
                        last.line = Line::Empty;
                    } else {
                        last.line = Line::Other;
                    }
                },
            }

            if last.line != Line::Empty {
                last.intermediate_blank.clear();
            }
        }
        planned.add_issues(last.extract_issues());

        Ok(planned)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Line {
    #[default]
    None,
    Empty,
    Separator,
    Issue,
    Description,
    Milestone,
    Other,
}

#[derive(Debug, Default)]
struct Last<ID> {
    issue_level: usize,
    issue_parent_id: Option<ID>,
    intermediate_blank: String,
    line: Line,
    parsed_issues: IndexMap<ID, Issue<ID>>,
}

impl<ID: HashedId + PartialEq + Clone> Last<ID> {
    fn new() -> Self {
        Self {
            issue_level: 0,
            issue_parent_id: None,
            intermediate_blank: String::new(),
            line: Line::None,
            parsed_issues: IndexMap::new(),
        }
    }

    fn find_parent(&mut self, item_level: usize) -> Option<&mut Issue<ID>> {
        let diff = self.issue_level - item_level;

        let mut parent_issue_idx = self.parsed_issues.get_index_of(self.issue_parent_id.as_ref()?)?;
        for _ in 0..diff {
            let parent_issue = self.parsed_issues.get_index(parent_issue_idx)?.1;
            parent_issue_idx = self.parsed_issues.get_index_of(parent_issue.parent_id.as_ref()?)?;
        }
        Some(self.parsed_issues.get_index_mut(parent_issue_idx)?.1)
    }

    fn extract_issues(&mut self) -> impl IntoIterator<Item = Issue<ID>> + '_ {
        self.parsed_issues.drain(..).map(|(_, issue)| issue)
    }

    fn insert_issue(&mut self, issue: Issue<ID>, issue_level: usize) {
        self.issue_parent_id.clone_from(&issue.parent_id);
        self.issue_level = issue_level;
        self.parsed_issues.insert(issue.id.clone(), issue);
    }
}
