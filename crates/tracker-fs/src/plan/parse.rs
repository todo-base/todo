use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::Regex;
use todo_lib::issue::{Issue, Milestone};

use crate::generator::IdGenerator;

pub enum Item<ID> {
    Empty,
    Separator,
    Issue(Issue<ID>),
    Milestone(Milestone<ID>),
    Text(String),
}

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
        } else if <Milestone<ID> as ParseLine<GEN>>::regex().is_match(&line) {
            Item::Milestone(Milestone::parse_line(line_trimmed, id_generator))
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

        Self {
            id,
            name,
            parent_id: None,
            content: Default::default(),
            subissues: Default::default(),
            relations: Default::default(),
        }
    }
}

impl<ID, GEN> ParseLine<GEN> for Milestone<ID>
where
    ID: FromStr,
    GEN: IdGenerator<Id = ID>,
{
    fn regex() -> &'static Regex {
        static MILESTONE_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^#\s+([0-9]+\s)?\s*(.*)").expect("regex must be correct"));
        &MILESTONE_REGEX
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

        Self {
            id,
            name,
            needed_issues: Default::default(),
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
    fn parse_milestone() {
        let id_generator = IntIdGenerator::new(1);

        let milestone =
            <Milestone<u64> as ParseLine<&IntIdGenerator>>::parse_line("# Milestone without id", &id_generator);
        assert_eq!(milestone.id, 1);
        assert_eq!(milestone.name, "Milestone without id");

        let milestone =
            <Milestone<u64> as ParseLine<&IntIdGenerator>>::parse_line("# 25 Milestone with id", &id_generator);
        assert_eq!(milestone.id, 25);
        assert_eq!(milestone.name, "Milestone with id");

        let milestone =
            <Milestone<u64> as ParseLine<&IntIdGenerator>>::parse_line("# 25Milestone without id", &id_generator);
        assert_eq!(milestone.id, 2);
        assert_eq!(milestone.name, "25Milestone without id");
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
            ("# Milestone", Item::Milestone(Milestone::new(3, "Milestone"))),
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
                (Item::Milestone(milestone), Item::Milestone(parsed_milestone)) => {
                    assert_eq!(milestone.id, parsed_milestone.id);
                    assert_eq!(milestone.name, parsed_milestone.name);
                },
                (Item::Text(text), Item::Text(parsed_text)) => {
                    assert_eq!(text, parsed_text);
                },
                _ => assert!(false),
            }
        }
    }
}
