//! CommonMark reader: parse a markdown source into a [`ParsedPlan`] using
//! `pulldown-cmark`'s `OffsetIter`.
//!
//! Mapping:
//! - list items (`-`/`*`/`+`) → issues; nesting → parent/subissue
//! - ATX headings (`#`) → sets (name = `/`-joined heading stack; an issue belongs to the heading scope active when it
//!   appears)
//! - first paragraph of an item → issue name (numeric prefix → id); a single leading `[name](file)` link →
//!   `IssueContent::Linked`
//! - remaining direct-child blocks (paragraphs, code blocks, …) → description, dedented to the item's content indent
//! - thematic break (`---`) → ignored (decorative)
//!
//! `issue_spans` records the full byte range of each item (incl. nested
//! children + description) for in-place edits.

use std::ops::Range;
use std::str::FromStr;

use indexmap::{IndexMap, IndexSet};
use once_cell::sync::Lazy;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use todo_lib::id::HashedId;
use todo_lib::issue::{Issue, IssueContent, IssueSet};
use todo_lib::plan::Plan;

use crate::generator::IdGenerator;
use crate::patch;

/// A parsed plan together with byte spans of each issue's list item in `source`.
pub struct ParsedPlan<ID> {
    pub plan: Plan<ID>,
    pub issue_spans: IndexMap<ID, Range<usize>>,
    pub name_spans: IndexMap<ID, Range<usize>>,
    pub content_spans: IndexMap<ID, Range<usize>>,
}

fn gfm_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options
}

fn heading_level_num(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

static ID_PREFIX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[[:space:]]*([0-9]+)[[:space:]]+(.*)$").expect("regex must be correct"));

/// Trim `source[start..end]` to non-whitespace bounds; if it begins with a
/// numeric id prefix return the parsed id and the borrowed remainder bounds.
fn name_span(source: &str, start: usize, end: usize) -> (Option<usize>, usize, usize) {
    let (from, to) = trim_span(source, start, end);
    if let Some(caps) = ID_PREFIX_REGEX.captures(&source[from..to]) {
        let id = caps.get(1).and_then(|id_match| id_match.as_str().parse::<usize>().ok());
        let rest = caps.get(2).expect("second group is mandatory");
        (id, from + rest.start(), from + rest.end())
    } else {
        (None, from, to)
    }
}

fn trim_span(source: &str, start: usize, end: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let (mut from, mut to) = (start, end);
    while from < to && bytes[from].is_ascii_whitespace() {
        from += 1;
    }
    while to > from && bytes[to - 1].is_ascii_whitespace() {
        to -= 1;
    }
    (from, to)
}

struct RawItem<ID> {
    parent_idx: Option<usize>,
    item_range: Range<usize>,
    list_depth: usize,
    set_name: Option<String>,
    first_block_done: bool,
    in_name_block: bool,
    name_started: bool,
    name_start: usize,
    name_end: usize,
    text_before_link: bool,
    in_link: bool,
    link_dest: Option<String>,
    link_text_start: usize,
    link_text_end: usize,
    link_end: usize,
    desc_start: Option<usize>,
    desc_end: usize,
    id_override: Option<ID>,
    content: IssueContent,
    name_span: Option<Range<usize>>,
    content_span: Option<Range<usize>>,
}

impl<ID> RawItem<ID> {
    fn new(parent_idx: Option<usize>, item_range: Range<usize>, list_depth: usize) -> Self {
        let item_start = item_range.start;
        Self {
            parent_idx,
            item_range,
            list_depth,
            set_name: None,
            first_block_done: false,
            in_name_block: false,
            name_started: false,
            name_start: item_start,
            name_end: item_start,
            text_before_link: false,
            in_link: false,
            link_dest: None,
            link_text_start: item_start,
            link_text_end: item_start,
            link_end: 0,
            desc_start: None,
            desc_end: 0,
            id_override: None,
            content: IssueContent::Empty,
            name_span: None,
            content_span: None,
        }
    }

    fn is_filelink(&self) -> bool {
        self.link_dest.is_some() && !self.text_before_link && self.link_text_end > self.link_text_start
    }

    fn begin_name(&mut self, at: usize) {
        if !self.name_started {
            self.name_start = at;
            self.name_started = true;
        }
    }

    fn extend_desc(&mut self, range: Range<usize>) {
        self.desc_start.get_or_insert(range.start);
        self.desc_end = self.desc_end.max(range.end);
    }
}

/// Parse CommonMark `source` into a [`ParsedPlan`].
pub fn parse<ID, GEN>(source: &str, id_gen: &GEN) -> ParsedPlan<ID>
where
    ID: HashedId + Clone + FromStr,
    GEN: IdGenerator<Id = ID>,
{
    let mut raw_items: Vec<RawItem<ID>> = Vec::new();
    let mut item_stack: Vec<usize> = Vec::new();
    let mut list_depth: usize = 0;

    let mut heading_stack: Vec<(usize, String)> = Vec::new();
    let mut current_set: Option<String> = None;
    let mut plan_sets: Vec<String> = Vec::new();

    let mut heading_pending: Option<usize> = None;
    let mut heading_text = String::new();

    for (event, range) in Parser::new_ext(source, gfm_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::List(_)) => list_depth += 1,
            Event::End(TagEnd::List(_)) => list_depth -= 1,

            Event::Start(Tag::Heading { level, .. }) => {
                if item_stack.is_empty() {
                    heading_pending = Some(heading_level_num(level));
                    heading_text.clear();
                } else if let Some(&idx) = item_stack.last() {
                    // A heading inside a list item is description content
                    // (e.g. ASCII-art setext-lookalikes), not a set delimiter.
                    let cur = &mut raw_items[idx];
                    if list_depth == cur.list_depth && cur.first_block_done {
                        cur.extend_desc(range);
                    }
                }
            },
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = heading_pending.take() {
                    while let Some((top, _)) = heading_stack.last() {
                        if *top >= level {
                            heading_stack.pop();
                        } else {
                            break;
                        }
                    }
                    heading_stack.push((level, heading_text.clone()));
                    let set_name = heading_stack
                        .iter()
                        .map(|(_, heading)| heading.as_str())
                        .collect::<Vec<_>>()
                        .join("/");
                    current_set = Some(set_name.clone());
                    if !plan_sets.contains(&set_name) {
                        plan_sets.push(set_name);
                    }
                }
            },

            Event::Start(Tag::Item) => {
                let mut item = RawItem::new(item_stack.last().copied(), range, list_depth);
                item.set_name = current_set.clone();
                item.in_name_block = true;
                let idx = raw_items.len();
                raw_items.push(item);
                item_stack.push(idx);
            },
            Event::End(TagEnd::Item) => {
                let idx = item_stack.pop().expect("item stack balanced");
                let cur = &mut raw_items[idx];
                if cur.in_name_block {
                    finalize_name(cur, source);
                }
                finalize_item(cur, source);
            },

            Event::Start(Tag::Paragraph) => {
                if let Some(&idx) = item_stack.last() {
                    let cur = &mut raw_items[idx];
                    if list_depth == cur.list_depth && !cur.in_name_block && cur.first_block_done {
                        cur.extend_desc(range);
                    }
                }
            },
            Event::Start(Tag::CodeBlock(_)) => {
                if let Some(&idx) = item_stack.last() {
                    let cur = &mut raw_items[idx];
                    if list_depth == cur.list_depth {
                        if cur.in_name_block {
                            finalize_name(cur, source);
                        }
                        cur.extend_desc(range);
                    }
                }
            },
            Event::End(TagEnd::Paragraph) => {
                if let Some(&idx) = item_stack.last() {
                    let cur = &mut raw_items[idx];
                    if cur.in_name_block {
                        finalize_name(cur, source);
                    }
                }
            },

            Event::Text(text) => {
                if heading_pending.is_some() {
                    heading_text.push_str(text.as_ref());
                } else if let Some(&idx) = item_stack.last() {
                    let cur = &mut raw_items[idx];
                    if cur.in_name_block {
                        cur.begin_name(range.start);
                        if cur.in_link {
                            if cur.link_text_end <= cur.link_text_start {
                                cur.link_text_start = range.start;
                            }
                            cur.link_text_end = range.end;
                        } else {
                            if cur.link_dest.is_none() {
                                cur.text_before_link = true;
                            }
                            cur.name_end = cur.name_end.max(range.end);
                        }
                    }
                }
            },
            Event::Code(_) => {
                if let Some(&idx) = item_stack.last() {
                    let cur = &mut raw_items[idx];
                    if cur.in_name_block && !cur.in_link && cur.link_dest.is_none() {
                        cur.begin_name(range.start);
                        cur.text_before_link = true;
                        cur.name_end = cur.name_end.max(range.end);
                    }
                }
            },
            Event::Start(Tag::Link { dest_url, .. }) => {
                if let Some(&idx) = item_stack.last() {
                    let cur = &mut raw_items[idx];
                    if cur.in_name_block {
                        cur.begin_name(range.start);
                        cur.in_link = true;
                        cur.link_dest = Some(dest_url.into_string());
                        cur.link_text_start = range.start;
                        cur.link_text_end = range.start;
                    }
                }
            },
            Event::End(TagEnd::Link) => {
                if let Some(&idx) = item_stack.last() {
                    let cur = &mut raw_items[idx];
                    if cur.in_name_block {
                        cur.in_link = false;
                        cur.link_end = range.end;
                        cur.name_end = cur.name_end.max(range.end);
                    }
                }
            },

            _ => {},
        }
    }

    // Second pass: assign ids, build Issues, link parent/subissue, attach sets.
    let ids: Vec<ID> = raw_items
        .iter()
        .map(|raw| raw.id_override.clone().unwrap_or_else(|| id_gen.next()))
        .collect();

    let mut children: Vec<IndexSet<ID>> = vec![IndexSet::new(); raw_items.len()];
    let mut set_membership: Vec<Option<String>> = Vec::with_capacity(raw_items.len());
    for (index, raw) in raw_items.iter().enumerate() {
        if let Some(parent_index) = raw.parent_idx {
            children[parent_index].insert(ids[index].clone());
        }
        set_membership.push(raw.set_name.clone());
    }

    let mut plan = Plan::new();
    let mut issue_spans: IndexMap<ID, Range<usize>> = IndexMap::new();
    let mut name_spans: IndexMap<ID, Range<usize>> = IndexMap::new();
    let mut content_spans: IndexMap<ID, Range<usize>> = IndexMap::new();
    for ((index, raw), child_set) in raw_items.into_iter().enumerate().zip(children) {
        let id = ids[index].clone();
        let name_span = raw.name_span;
        let content_span = raw.content_span;
        let name = name_span
            .as_ref()
            .map(|span| &source[span.start..span.end])
            .unwrap_or_default();
        let mut issue = Issue::new(id.clone(), name);
        issue.parent_id = raw.parent_idx.map(|parent_index| ids[parent_index].clone());
        issue.content = raw.content;
        issue.subissues = child_set;
        plan.add_issue(issue);
        issue_spans.insert(id.clone(), raw.item_range);
        if let Some(span) = name_span {
            name_spans.insert(id.clone(), span);
        }
        if let Some(span) = content_span {
            content_spans.insert(id.clone(), span);
        }
    }
    for set_name in &plan_sets {
        plan.add_set(IssueSet::new(set_name.clone()));
    }
    for (index, set_name) in set_membership.iter().enumerate() {
        if let Some(name) = set_name {
            plan.add_issue_to_set(name, ids[index].clone());
        }
    }

    ParsedPlan {
        plan,
        issue_spans,
        name_spans,
        content_spans,
    }
}

fn finalize_name<ID: FromStr>(cur: &mut RawItem<ID>, source: &str) {
    let is_filelink = cur.is_filelink();
    let (start, end) = if is_filelink {
        (cur.link_text_start, cur.link_text_end)
    } else {
        (cur.name_start, cur.name_end)
    };
    let (id_num, from, to) = name_span(source, start, end);
    cur.name_span = Some(from..to);
    cur.id_override = id_num.and_then(|parsed_id| parsed_id.to_string().parse::<ID>().ok());
    cur.first_block_done = true;
    cur.in_name_block = false;
}

fn finalize_item<ID>(cur: &mut RawItem<ID>, source: &str) {
    let is_filelink = cur.is_filelink();
    let indent = patch::item_content_indent(source, cur.item_range.start);
    let (content, content_span) = if is_filelink {
        let dest = cur.link_dest.take().unwrap_or_default();
        let (from, to) = trim_span(source, cur.link_end, cur.item_range.end);
        let note = (from < to).then(|| patch::dedent_lines(&source[from..to], indent));
        let span = (from < to).then_some(from..to);
        (
            IssueContent::Linked {
                file: dest.into(),
                note,
            },
            span,
        )
    } else if let Some(start) = cur.desc_start {
        let (from, to) = trim_span(source, start, cur.desc_end);
        if from < to {
            (
                IssueContent::Inline(patch::dedent_lines(&source[from..to], indent)),
                Some(from..to),
            )
        } else {
            (IssueContent::Empty, None)
        }
    } else {
        (IssueContent::Empty, None)
    };
    cur.content = content;
    cur.content_span = content_span;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::IntIdGenerator;

    fn plan_of(src: &str) -> Plan<u64> {
        parse::<u64, _>(src, &IntIdGenerator::new(1)).plan
    }

    #[test]
    fn plain_issues_and_nesting() {
        let plan = plan_of("- a\n  - aa\n- b\n");
        let ids: Vec<u64> = plan.issues().keys().copied().collect();
        assert_eq!(ids, vec![1, 2, 3]);
        assert_eq!(*plan.get_issue(&1).unwrap(), Issue::new(1, "a").with_subissue(2));
        assert_eq!(*plan.get_issue(&2).unwrap(), Issue::new(2, "aa").with_parent_id(1));
        assert_eq!(plan.get_issue(&3).unwrap().name, "b");
    }

    #[test]
    fn numeric_id_prefix() {
        let plan = plan_of("- 25 named\n- auto\n");
        assert_eq!(plan.get_issue(&25).unwrap().name, "named");
        assert_eq!(plan.get_issue(&1).unwrap().name, "auto");
    }

    #[test]
    fn headings_define_sets() {
        let plan = plan_of("- a\n# Mile 1\n- b\n");
        let names: Vec<&str> = plan.sets().keys().map(|set_name| set_name.as_str()).collect();
        assert_eq!(names, vec!["Mile 1"]);
        // "a" appears before the heading → no set; "b" → "Mile 1"
        assert_eq!(
            plan.get_set("Mile 1")
                .unwrap()
                .issues
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![2]
        );
    }

    #[test]
    fn filelink_issue() {
        let plan = plan_of("- [task K](task-k.md) trailing note\n");
        let issue = plan.get_issue(&1).unwrap();
        assert_eq!(issue.name, "task K");
        match &issue.content {
            IssueContent::Linked { file, note } => {
                assert_eq!(file.to_str().unwrap(), "task-k.md");
                assert_eq!(note.as_deref(), Some("trailing note"));
            },
            other => panic!("expected Linked, got {other:?}"),
        }
    }

    #[test]
    fn description_is_dedented() {
        // a blank line separates the name from the description paragraph
        let plan = plan_of("- g\n\n  Multi line\n  description\n");
        let issue = plan.get_issue(&1).unwrap();
        assert_eq!(issue.name, "g");
        match &issue.content {
            IssueContent::Inline(content) => assert_eq!(content, "Multi line\ndescription"),
            other => panic!("expected Inline, got {other:?}"),
        }
    }
}
