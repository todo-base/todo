use std::mem;
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::Regex;
use todo_lib::id::HashedId;
use todo_lib::issue::{Issue, IssueContent};

use crate::generator::IdGenerator;

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub level: usize,
    pub name: String,
}

impl Header {
    pub fn new(level: usize, name: impl Into<String>) -> Self {
        Self {
            level,
            name: name.into(),
        }
    }
}

#[derive(Debug)]
pub enum Item<ID> {
    Empty,
    Separator,
    Issue(Issue<ID>),
    Header(Header),
    Text(String),
}

impl<ID: HashedId + PartialEq> PartialEq for Item<ID> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Issue(left), Self::Issue(right)) => left == right,
            (Self::Header(left), Self::Header(right)) => left == right,
            (Self::Text(left), Self::Text(right)) => left == right,
            _ => mem::discriminant(self) == mem::discriminant(other),
        }
    }
}

impl<ID: HashedId> Eq for Item<ID> {}

impl<ID: FromStr> Item<ID> {
    pub fn parse<GEN: IdGenerator<Id = ID>>(line: impl Into<String>, id_generator: GEN) -> (Self, usize) {
        static SEPARATOR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^-{3,}\s?.*").expect("regex must be correct"));

        let line = line.into();
        let line_trimmed = line.trim_start_matches(' ');
        let line_level = (line.len() - line_trimmed.len()) / 2;

        let item = if line.is_empty() {
            Item::Empty
        } else if <Issue<ID> as ParseLine<GEN>>::regex().is_match(&line) {
            Item::Issue(Issue::parse_line(line_trimmed, id_generator))
        } else if SEPARATOR_REGEX.is_match(&line) {
            Item::Separator
        } else if let Some(captures) = header_regex().captures(&line) {
            let level = captures.get(1).map(|hashes| hashes.as_str().len()).unwrap_or(1);
            let name = captures
                .get(2)
                .map(|mat| mat.as_str().trim().to_string())
                .unwrap_or_default();
            Item::Header(Header { level, name })
        } else {
            Item::Text(line)
        };

        (item, line_level)
    }
}

pub trait ParseLine<GEN> {
    fn regex() -> &'static Regex;
    fn parse_line(line: &str, id_generator: GEN) -> Self;
}

fn header_regex() -> &'static Regex {
    static HEADER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(#+)\s+(.*)").expect("regex must be correct"));
    &HEADER_REGEX
}

fn issue_name_filelink_regex() -> &'static Regex {
    static FILELINK_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\s*\[(.*)\]\((.*)\)(.*)").expect("regex must be correct"));
    &FILELINK_REGEX
}

impl<ID, GEN> ParseLine<GEN> for Issue<ID>
where
    ID: FromStr,
    GEN: IdGenerator<Id = ID>,
{
    fn regex() -> &'static Regex {
        static ISSUE_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^\s*[-+]\s+([0-9]+\s)?\s*(.*)").expect("regex must be correct"));
        &ISSUE_REGEX
    }

    fn parse_line(line: &str, id_generator: GEN) -> Self {
        let captures = <Self as ParseLine<GEN>>::regex().captures(line);

        let id = captures
            .as_ref()
            .and_then(|caps| caps.get(1))
            .and_then(|value| value.as_str().trim().parse().ok())
            .unwrap_or_else(|| id_generator.next());

        let name = captures
            .as_ref()
            .and_then(|caps| caps.get(2))
            .map(|mat| mat.as_str().trim().to_string())
            .unwrap_or_default();

        let name_captures = issue_name_filelink_regex().captures(&name);
        let (name, content_file_path, trailing_note) = name_captures
            .as_ref()
            .map(|caps| {
                let name = caps
                    .get(1)
                    .map(|value| value.as_str().trim())
                    .unwrap_or(&name)
                    .to_string();
                let content_file = caps.get(2).map(|value| value.as_str().trim().to_string());
                let trailing_note = caps.get(3).map(|value| value.as_str().trim().to_string());

                (name, content_file, trailing_note)
            })
            .unwrap_or((name, None, None));

        let content = if let Some(path) = content_file_path {
            IssueContent::Linked {
                file: path.into(),
                note: trailing_note,
            }
        } else {
            IssueContent::Empty
        };

        Self {
            id,
            name,
            parent_id: None,
            content,
            subissues: Default::default(),
            relations: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::IntIdGenerator;

    #[test]
    fn parse_issue() {
        let id_generator = IntIdGenerator::new(1);

        let issue = <Issue<u64> as ParseLine<&IntIdGenerator>>::parse_line("- task without id", &id_generator);
        assert_eq!(issue.id, 1);
        assert_eq!(issue.name, "task without id");

        let issue = <Issue<u64> as ParseLine<&IntIdGenerator>>::parse_line("- 25 task with id", &id_generator);
        assert_eq!(issue.id, 25);
        assert_eq!(issue.name, "task with id");

        let issue = <Issue<u64> as ParseLine<&IntIdGenerator>>::parse_line("- 25task without id", &id_generator);
        assert_eq!(issue.id, 2);
        assert_eq!(issue.name, "25task without id");
    }

    #[test]
    fn parse_header() {
        let id_generator = IntIdGenerator::new(1);

        let (item, _) = Item::<u64>::parse("# Header level 1", &id_generator);
        assert_eq!(item, Item::Header(Header::new(1, "Header level 1")));

        let (item, _) = Item::<u64>::parse("## Header level 2", &id_generator);
        assert_eq!(item, Item::Header(Header::new(2, "Header level 2")));

        let (item, _) = Item::<u64>::parse("### Header level 3", &id_generator);
        assert_eq!(item, Item::Header(Header::new(3, "Header level 3")));
    }

    #[test]
    fn parse_item() {
        let id_generator = IntIdGenerator::new(1);
        let pairs = [
            ("Task list", Item::Text("Task list".into())),
            ("---", Item::Separator),
            ("", Item::Empty),
            ("- Task 1", Item::Issue(Issue::new(1, "Task 1"))),
            ("  task 1 description", Item::Text("  task 1 description".into())),
            ("  - Subtask 1", Item::Issue(Issue::new(2, "Subtask 1"))),
            ("", Item::Empty),
            ("---", Item::Separator),
            ("", Item::Empty),
            ("# Set", Item::Header(Header::new(1, "Set"))),
            ("## Sub set", Item::Header(Header::new(2, "Sub set"))),
        ];

        for (line, item) in pairs {
            let parsed_item = Item::parse(line, &id_generator).0;
            match (item, parsed_item) {
                (Item::Empty, Item::Empty) => {},
                (Item::Separator, Item::Separator) => {},
                (Item::Issue(issue), Item::Issue(parsed_issue)) => {
                    assert_eq!(issue.id, parsed_issue.id);
                    assert_eq!(issue.name, parsed_issue.name);
                },
                (Item::Header(header), Item::Header(parsed_header)) => {
                    assert_eq!(header, parsed_header);
                },
                (Item::Text(text), Item::Text(parsed_text)) => {
                    assert_eq!(text, parsed_text);
                },
                _ => panic!("Incorrect parse result"),
            }
        }
    }
}
