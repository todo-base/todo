//! The writer and the reader must agree: whatever `to_text` writes, `parse` has
//! to read back as the same issue.
//!
//! A description closing with a bullet list is the one thing the writer changes:
//! it appends a `---` so the list is not read back as subissues. That `---` is
//! ordinary description text, so writing the result again changes nothing.

use std::path::PathBuf;

use todo_lib::issue::{Issue, IssueContent};
use todo_tracker_fs::generator::IntIdGenerator;
use todo_tracker_fs::issue::{CONTENT_SEPARATOR, SaveIssue};
use todo_tracker_fs::plan::parse::{ends_with_bullet_item, parse};

const DESCRIPTIONS: [&str; 11] = [
    "one line",
    "line one\nline two",
    "para one\n\npara two",
    "intro:\n- alpha\n- beta",
    "intro:\n- alpha\n\noutro",
    "- alpha",
    "- alpha\n  - nested",
    "* star\n+ plus",
    "steps:\n1. first\n2. second",
    "text\n\n```code\n- not a list\n```",
    "closed already:\n- alpha\n---",
];

#[track_caller]
fn read_back(issue: &Issue<u64>) -> Issue<u64> {
    let text = format!("{}\n", issue.to_text());
    let parsed = parse::<u64, _>(&text, &IntIdGenerator::new(1))
        .unwrap_or_else(|error| panic!("reading {text:?} back: {error}"));

    let ids: Vec<u64> = parsed.plan.issues().keys().copied().collect();
    assert_eq!(ids, vec![1], "{text:?} must read back as exactly one issue");
    parsed.plan.into_issues().shift_remove(&1).unwrap()
}

/// What the reader owes back: the description as given, plus the `---` the writer
/// had to append to keep a closing list out of the subissues.
fn expected(content: &str) -> String {
    if ends_with_bullet_item(content) {
        format!("{content}\n{CONTENT_SEPARATOR}")
    } else {
        content.to_string()
    }
}

#[track_caller]
fn assert_writing_is_stable(issue: &Issue<u64>) {
    let once = issue.to_text();
    let twice = read_back(issue).to_text();
    assert_eq!(twice, once, "writing an issue back must reach a fixed point");
}

#[test]
fn inline_description_survives_a_write_and_read() {
    for content in DESCRIPTIONS {
        let issue = Issue::new(1, "task").with_content(content);
        let back = read_back(&issue);
        assert_eq!(back.name, "task");
        assert_eq!(
            back.content,
            IssueContent::Inline(expected(content)),
            "content {content:?}"
        );
        assert_writing_is_stable(&issue);
    }
}

#[test]
fn linked_note_survives_a_write_and_read() {
    for note in DESCRIPTIONS {
        let issue = Issue::new(1, "task").with_content_file_and_note("task.md", note);
        let back = read_back(&issue);
        assert_eq!(back.name, "task");
        assert_eq!(
            back.content,
            IssueContent::Linked {
                file: PathBuf::from("task.md"),
                note: Some(expected(note)),
            },
            "note {note:?}"
        );
        assert_writing_is_stable(&issue);
    }
}

#[test]
fn an_issue_without_a_description_survives_a_write_and_read() {
    let issue = Issue::new(1, "task");
    assert_eq!(read_back(&issue), issue);
}
