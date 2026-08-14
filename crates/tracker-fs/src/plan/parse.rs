//! CommonMark reader: parse a markdown source into a [`ParsedPlan`] using
//! `pulldown-cmark`'s `OffsetIter`.
//!
//! Mapping:
//! - bullet list items (`-`/`*`/`+`) → issues; an ordered list (`1.`/`1)`) is ordinary content, and so is everything
//!   nested under it
//! - ATX headings (`#`) → sets (name = `/`-joined heading stack; an issue belongs to the heading scope active when it
//!   appears)
//! - first line of an item → issue name (numeric prefix → id); a `[name](file)` link opening that line →
//!   `IssueContent::Linked`
//! - rest of that line's block, plus the following direct-child blocks (paragraphs, code blocks, …) → description,
//!   dedented to the item's content indent
//! - nested lists closing an item → its subissues; a nested list with description content after it stays part of that
//!   description and yields no issues
//! - thematic break (`---`) → ignored between issues; indented to an item's content it is description text like any
//!   other block, and it ends the subissues, so a list right before it stays content
//!
//! An id spelled out in the source is never handed to another issue: generated
//! ids skip the ones the source already claims, and spelling the same id out
//! twice is a [`ParseError`].
//!
//! [`ParsedPlan`] carries the byte spans every issue occupies in the source, so
//! a writer can patch one issue in place.

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::ops::Range;
use std::str::FromStr;
use std::{io, mem};

use indexmap::{IndexMap, IndexSet};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use thiserror::Error;
use todo_lib::id::HashedId;
use todo_lib::issue::{Issue, IssueContent, IssueSet};
use todo_lib::plan::Plan;

use crate::generator::{IdGenerator, IntIdGenerator};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("in line {line}: issue id `{id}` is already used by `{owner}`")]
    DuplicateId { line: usize, id: String, owner: String },
}

impl From<ParseError> for io::Error {
    fn from(error: ParseError) -> Self {
        Self::new(io::ErrorKind::InvalidData, error)
    }
}

/// A parsed plan together with the byte spans its issues occupy in `source`, so
/// an edit can rewrite one issue and leave the rest of the file byte-identical.
///
/// The spans describe the `source` it was parsed from: they go stale as soon as
/// that text — or `plan` — changes, so patch one issue and parse again.
pub struct ParsedPlan<ID> {
    pub plan: Plan<ID>,
    /// The whole list item, its subissues and description included.
    pub issue_spans: IndexMap<ID, Range<usize>>,
    /// The name alone, as written — markup and all, without the id prefix.
    pub name_spans: IndexMap<ID, Range<usize>>,
    /// The description as written, still carrying the item's indentation that
    /// [`IssueContent`] holds stripped. Absent for an issue without one.
    pub content_spans: IndexMap<ID, Range<usize>>,
}

fn gfm_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options
}

struct Heading {
    level: usize,
    name: String,
}

/// The heading stack, turned into set names: an issue joins the set of the
/// heading scope active where it appears.
#[derive(Default)]
struct SetTracker {
    stack: Vec<Heading>,
    current: Option<String>,
    names: IndexSet<String>,
    /// Level of the heading being read; its text arrives as separate events.
    open_level: Option<usize>,
    text: String,
}

impl SetTracker {
    fn open(&mut self, level: HeadingLevel) {
        self.open_level = Some(level as usize);
        self.text.clear();
    }

    fn is_open(&self) -> bool {
        self.open_level.is_some()
    }

    fn push_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    /// The heading is complete: it replaces the scopes it closes and becomes the
    /// set that follows.
    fn close(&mut self) {
        let Some(level) = self.open_level.take() else {
            return;
        };
        while self.stack.last().is_some_and(|heading| heading.level >= level) {
            self.stack.pop();
        }
        self.stack.push(Heading {
            level,
            name: mem::take(&mut self.text),
        });

        let name = self
            .stack
            .iter()
            .map(|heading| heading.name.as_str())
            .collect::<Vec<_>>()
            .join("/");
        self.current = Some(name.clone());
        self.names.insert(name);
    }
}

/// The `<digits><space>` prefix `name` opens with, if any — the reader takes it
/// as the issue id rather than as part of the name.
pub fn id_prefix(name: &str) -> Option<&str> {
    let (prefix, _) = name.split_once(char::is_whitespace)?;
    (!prefix.is_empty() && prefix.bytes().all(|byte| byte.is_ascii_digit())).then_some(prefix)
}

/// Trim `range` to non-whitespace bounds; an id prefix is consumed as the id.
fn split_id_prefix<ID: FromStr>(source: &str, range: Range<usize>) -> (Option<ID>, Range<usize>) {
    let span = trim_span(source, range);
    let Some(prefix) = id_prefix(&source[span.clone()]) else {
        return (None, span);
    };
    let rest = trim_span(source, span.start + prefix.len()..span.end);
    (ID::from_str(prefix).ok(), rest)
}

fn trim_span(source: &str, range: Range<usize>) -> Range<usize> {
    let bytes = source.as_bytes();
    let Range { mut start, mut end } = range;
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    start..end
}

/// End of the first line of `range`; the line break itself is not included.
fn first_line_end(source: &str, range: &Range<usize>) -> usize {
    source[range.clone()]
        .find('\n')
        .map_or(range.end, |offset| range.start + offset)
}

/// 1-based number of the line `offset` falls on.
fn line_at(source: &str, offset: usize) -> usize {
    source[..offset].bytes().filter(|&byte| byte == b'\n').count() + 1
}

/// Indent of the line `content_start` belongs to — the column at which an item's
/// content, and every continuation line of it, begins.
fn content_indent(source: &str, content_start: usize) -> usize {
    let line_start = source[..content_start].rfind('\n').map_or(0, |offset| offset + 1);
    content_start - line_start
}

/// Strip up to `indent` leading spaces from each line of `text` (logical content).
fn dedent_lines(text: &str, indent: usize) -> String {
    text.lines()
        .map(|line| {
            let leading_spaces = line.bytes().take(indent).take_while(|&byte| byte == b' ').count();
            &line[leading_spaces..]
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Markers that open a list item carrying an issue.
const BULLET_MARKERS: [char; 3] = ['-', '*', '+'];

/// Does `line` open a bullet list item — the markup a reader takes for an issue?
pub fn is_bullet_item(line: &str) -> bool {
    if let Some(rest) = line.trim_start().strip_prefix(BULLET_MARKERS) {
        rest.is_empty() || rest.starts_with([' ', '\t'])
    } else {
        false
    }
}

/// Would `name`, written as an issue name, read back as a reference to a file
/// rather than as plain text? Asks the reader itself, so the two cannot drift.
pub fn reads_as_filelink(name: &str) -> bool {
    let Ok(parsed) = parse::<u64, _>(&format!("- {name}"), &IntIdGenerator::new(1)) else {
        return false;
    };
    parsed
        .plan
        .issues()
        .values()
        .next()
        .is_some_and(|issue| matches!(issue.content, IssueContent::Linked { .. }))
}

/// Would the last block of `content` read back as a list of issues rather than as
/// description text? A writer has to close such content with a `---`.
pub fn ends_with_bullet_item(content: &str) -> bool {
    content
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .is_some_and(is_bullet_item)
}

/// Offset an item's content starts at: past its bullet marker (`-`, `*`, `+`) and
/// the spaces separating it from the content.
fn item_content_start(source: &str, item_start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut offset = item_start;
    if bytes
        .get(offset)
        .is_some_and(|&byte| BULLET_MARKERS.contains(&byte.into()))
    {
        offset += 1;
    }
    while matches!(bytes.get(offset), Some(b' ' | b'\t')) {
        offset += 1;
    }
    offset
}

/// A `[name](file)` link opening an item's first line.
struct Filelink {
    range: Range<usize>,
    dest: String,
}

/// A list the reader is inside.
struct OpenList {
    /// Index the first issue of this list took in `raw_items`.
    first_item: usize,
    /// An ordered list carries no issues, only content.
    ordered: bool,
}

/// A direct-child block of an item, in document order.
enum ChildBlock {
    /// Description content: a paragraph, a code block, a heading, a `---`, …
    Description(Range<usize>),
    /// A nested list, along with the [`RawItem`] indices it produced.
    List { range: Range<usize>, items: Range<usize> },
}

impl ChildBlock {
    fn range(&self) -> &Range<usize> {
        match self {
            Self::Description(range) | Self::List { range, .. } => range,
        }
    }
}

struct RawItem<ID> {
    parent_idx: Option<usize>,
    item_range: Range<usize>,
    list_depth: usize,
    set_name: Option<String>,
    /// The item's first block has not been read yet: its first line is the name.
    in_name_block: bool,
    /// Source range of that first block. Starts as the whole item and is cut back
    /// to where the next block (a nested list, a code block, …) begins.
    name_block: Range<usize>,
    /// Column the item's content starts at, used to dedent its description.
    content_indent: usize,
    link: Option<Filelink>,
    children: Vec<ChildBlock>,
    /// Index in `children` where the trailing nested lists — the subissues —
    /// begin; everything before it is description.
    subissue_split: usize,
    id_override: Option<ID>,
    content: IssueContent,
    name_span: Option<Range<usize>>,
    content_span: Option<Range<usize>>,
}

impl<ID> RawItem<ID> {
    fn new(source: &str, parent_idx: Option<usize>, item_range: Range<usize>, list_depth: usize) -> Self {
        let content_start = item_content_start(source, item_range.start);
        let name_block = content_start..item_range.end;
        Self {
            parent_idx,
            item_range,
            list_depth,
            set_name: None,
            in_name_block: true,
            name_block,
            content_indent: content_indent(source, content_start),
            link: None,
            children: Vec::new(),
            subissue_split: 0,
            id_override: None,
            content: IssueContent::Empty,
            name_span: None,
            content_span: None,
        }
    }

    /// Nested lists closing the item are its subissues; a list followed by more
    /// description content belongs to that description.
    fn split_subissues(&mut self) {
        self.subissue_split = self
            .children
            .iter()
            .rposition(|block| !matches!(block, ChildBlock::List { .. }))
            .map_or(0, |idx| idx + 1);
    }

    fn name<'src>(&self, source: &'src str) -> &'src str {
        self.name_span.as_ref().map_or("", |span| &source[span.clone()])
    }

    fn description(&self) -> Option<Range<usize>> {
        let blocks = &self.children[..self.subissue_split];
        let (first, last) = (blocks.first()?, blocks.last()?);
        Some(first.range().start..last.range().end)
    }

    /// Lists demoted to description content; the issues they produced are dropped.
    fn demoted_lists(&self) -> impl Iterator<Item = &Range<usize>> {
        self.children[..self.subissue_split]
            .iter()
            .filter_map(|block| match block {
                ChildBlock::List { items, .. } => Some(items),
                ChildBlock::Description(_) => None,
            })
    }
}

/// The item the reader is inside, if any.
fn current_item<'a, ID>(raw_items: &'a mut [RawItem<ID>], item_stack: &[usize]) -> Option<&'a mut RawItem<ID>> {
    let &idx = item_stack.last()?;
    Some(&mut raw_items[idx])
}

/// Parse CommonMark `source` into a [`ParsedPlan`].
pub fn parse<ID, GEN>(source: &str, id_gen: &GEN) -> Result<ParsedPlan<ID>, ParseError>
where
    ID: HashedId + Clone + Display + FromStr,
    GEN: IdGenerator<Id = ID>,
{
    let mut raw_items: Vec<RawItem<ID>> = Vec::new();
    let mut item_stack: Vec<usize> = Vec::new();
    let mut list_stack: Vec<OpenList> = Vec::new();
    let mut list_depth: usize = 0;
    // Anything below an ordered list is content, however it is marked up.
    let mut ordered_depth: usize = 0;

    let mut sets = SetTracker::default();

    for (event, range) in Parser::new_ext(source, gfm_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::List(first_number)) => {
                // A nested list ends the item's name line, whatever it turns out to be.
                if let Some(cur) = current_item(&mut raw_items, &item_stack)
                    && list_depth == cur.list_depth
                    && cur.in_name_block
                {
                    finalize_name(cur, source, range.start);
                }
                let ordered = first_number.is_some();
                ordered_depth += usize::from(ordered);
                list_stack.push(OpenList {
                    first_item: raw_items.len(),
                    ordered,
                });
                list_depth += 1;
            },
            Event::End(TagEnd::List(_)) => {
                list_depth -= 1;
                let list = list_stack.pop().expect("list stack balanced");
                ordered_depth -= usize::from(list.ordered);
                let item_count = raw_items.len();
                if let Some(cur) = current_item(&mut raw_items, &item_stack)
                    && list_depth == cur.list_depth
                {
                    // Only a bullet list can hold subissues; an ordered one is content.
                    cur.children.push(if list.ordered {
                        ChildBlock::Description(range)
                    } else {
                        ChildBlock::List {
                            range,
                            items: list.first_item..item_count,
                        }
                    });
                }
            },

            // A heading inside a list item is description content
            // (e.g. ASCII-art setext-lookalikes), not a set delimiter.
            Event::Start(Tag::Heading { level, .. }) => match current_item(&mut raw_items, &item_stack) {
                None => sets.open(level),
                // A setext heading opens at the item's first line, so that line is
                // still the name and only its underline joins the description.
                Some(cur)
                    if list_depth == cur.list_depth && cur.in_name_block && cur.name_block.start == range.start =>
                {
                    finalize_name(cur, source, range.end);
                },
                Some(cur) if list_depth == cur.list_depth => {
                    if cur.in_name_block {
                        finalize_name(cur, source, range.start);
                    }
                    cur.children.push(ChildBlock::Description(range));
                },
                Some(_) => {},
            },
            Event::End(TagEnd::Heading(_)) => sets.close(),

            // A `---` indented to the item's content is description text like any
            // other block; it also ends the subissues, keeping a list before it content.
            Event::Rule => {
                if let Some(cur) = current_item(&mut raw_items, &item_stack)
                    && list_depth == cur.list_depth
                {
                    if cur.in_name_block {
                        finalize_name(cur, source, range.start);
                    }
                    cur.children.push(ChildBlock::Description(range));
                }
            },

            Event::Start(Tag::Item) if ordered_depth == 0 => {
                let mut item = RawItem::new(source, item_stack.last().copied(), range, list_depth);
                item.set_name = sets.current.clone();
                let idx = raw_items.len();
                raw_items.push(item);
                item_stack.push(idx);
            },
            Event::End(TagEnd::Item) if ordered_depth == 0 => {
                let idx = item_stack.pop().expect("item stack balanced");
                let cur = &mut raw_items[idx];
                if cur.in_name_block {
                    finalize_name(cur, source, range.end);
                }
                finalize_item(cur, source);
            },

            Event::Start(Tag::Paragraph) => {
                if let Some(cur) = current_item(&mut raw_items, &item_stack)
                    && list_depth == cur.list_depth
                    && !cur.in_name_block
                {
                    cur.children.push(ChildBlock::Description(range));
                }
            },
            Event::Start(Tag::CodeBlock(_)) => {
                if let Some(cur) = current_item(&mut raw_items, &item_stack)
                    && list_depth == cur.list_depth
                {
                    if cur.in_name_block {
                        finalize_name(cur, source, range.start);
                    }
                    cur.children.push(ChildBlock::Description(range));
                }
            },
            // Only a loose list reports paragraphs; it bounds the name block exactly.
            Event::End(TagEnd::Paragraph) => {
                if let Some(cur) = current_item(&mut raw_items, &item_stack)
                    && cur.in_name_block
                {
                    finalize_name(cur, source, range.end);
                }
            },

            // `Code` too: a set named `` # Mile `1` `` keeps the `1`.
            Event::Text(text) | Event::Code(text) => {
                if sets.is_open() {
                    sets.push_text(text.as_ref());
                }
            },
            // Only a link opening the name line makes the issue a linked one;
            // anywhere else it is part of the name or of the description.
            Event::Start(Tag::Link { dest_url, .. }) => {
                if let Some(cur) = current_item(&mut raw_items, &item_stack)
                    && cur.in_name_block
                    && cur.name_block.start == range.start
                {
                    cur.link = Some(Filelink {
                        range,
                        dest: dest_url.into_string(),
                    });
                }
            },

            _ => {},
        }
    }

    let dropped = demoted_items(&raw_items);
    let ids = assign_ids(&raw_items, &dropped, source, id_gen)?;
    Ok(build_plan(raw_items, &ids, &sets.names, source))
}

/// Flags the items a demoted list produced: they are description text, not issues.
fn demoted_items<ID>(raw_items: &[RawItem<ID>]) -> Vec<bool> {
    let mut dropped = vec![false; raw_items.len()];
    for raw in raw_items {
        for items in raw.demoted_lists() {
            dropped[items.clone()].fill(true);
        }
    }
    dropped
}

/// Turn the read items into a plan, skipping the ones without an id — those a
/// demoted list produced.
fn build_plan<ID: HashedId + Clone>(
    raw_items: Vec<RawItem<ID>>,
    ids: &[Option<ID>],
    set_names: &IndexSet<String>,
    source: &str,
) -> ParsedPlan<ID> {
    let mut children: Vec<IndexSet<ID>> = vec![IndexSet::new(); raw_items.len()];
    for (index, raw) in raw_items.iter().enumerate() {
        if let (Some(parent_index), Some(id)) = (raw.parent_idx, &ids[index]) {
            children[parent_index].insert(id.clone());
        }
    }

    let mut plan = Plan::new();
    for set_name in set_names {
        plan.add_set(IssueSet::new(set_name.clone()));
    }

    let mut issue_spans: IndexMap<ID, Range<usize>> = IndexMap::new();
    let mut name_spans: IndexMap<ID, Range<usize>> = IndexMap::new();
    let mut content_spans: IndexMap<ID, Range<usize>> = IndexMap::new();
    for ((index, raw), child_set) in raw_items.into_iter().enumerate().zip(children) {
        let Some(id) = ids[index].clone() else {
            continue;
        };
        let mut issue = Issue::new(id.clone(), raw.name(source));
        issue.parent_id = raw.parent_idx.and_then(|parent_index| ids[parent_index].clone());
        issue.content = raw.content;
        issue.subissues = child_set;
        plan.add_issue(issue);

        issue_spans.insert(id.clone(), raw.item_range);
        if let Some(span) = raw.name_span {
            name_spans.insert(id.clone(), span);
        }
        if let Some(span) = raw.content_span {
            content_spans.insert(id.clone(), span);
        }
        if let Some(set_name) = &raw.set_name {
            plan.add_issue_to_set(set_name, id);
        }
    }

    ParsedPlan {
        plan,
        issue_spans,
        name_spans,
        content_spans,
    }
}

/// Ids of the items that stay, `None` for the ones demoted into a description.
/// Ids the source spells out are reserved up front, so a generated one never
/// takes an id claimed further down; spelling the same id out twice is an error.
fn assign_ids<'src, ID, GEN>(
    raw_items: &[RawItem<ID>],
    dropped: &[bool],
    source: &'src str,
    id_gen: &GEN,
) -> Result<Vec<Option<ID>>, ParseError>
where
    ID: HashedId + Clone + Display,
    GEN: IdGenerator<Id = ID>,
{
    let reserved: HashSet<ID> = raw_items
        .iter()
        .zip(dropped)
        .filter(|&(_, &dropped)| !dropped)
        .filter_map(|(raw, _)| raw.id_override.clone())
        .collect();

    let mut assigned: HashMap<ID, &'src str> = HashMap::with_capacity(raw_items.len());
    let mut ids = Vec::with_capacity(raw_items.len());
    for (raw, &dropped) in raw_items.iter().zip(dropped) {
        if dropped {
            ids.push(None);
            continue;
        }
        let name = raw.name(source);
        let id = match &raw.id_override {
            Some(id) => {
                if let Some(owner) = assigned.get(id) {
                    return Err(ParseError::DuplicateId {
                        line: line_at(source, raw.item_range.start),
                        id: id.to_string(),
                        owner: (*owner).to_string(),
                    });
                }
                id.clone()
            },
            None => loop {
                let id = id_gen.next();
                if !reserved.contains(&id) && !assigned.contains_key(&id) {
                    break id;
                }
            },
        };
        assigned.insert(id.clone(), name);
        ids.push(Some(id));
    }
    Ok(ids)
}

/// Split the item's first block, which ends at `block_end`: its first line is the
/// issue name — the label of a `[name](file)` link when one opens it — and the
/// remainder starts the description.
fn finalize_name<ID: FromStr>(cur: &mut RawItem<ID>, source: &str, block_end: usize) {
    cur.in_name_block = false;
    cur.name_block.end = cur.name_block.end.min(block_end).max(cur.name_block.start);
    let block = cur.name_block.clone();
    let line_end = first_line_end(source, &block);

    let label = cur.link.as_ref().and_then(|link| {
        let label_start = link.range.start + 1;
        let label_end = link.range.start + source[link.range.clone()].rfind("](")?;
        (label_start <= label_end).then_some((label_start..label_end, link.range.end))
    });
    let (name, desc_start) = match label {
        Some((label, link_end)) => (label, link_end),
        None => {
            // A reference or autolink carries no file path: it stays in the name.
            cur.link = None;
            (block.start..line_end, line_end)
        },
    };

    let (id, span) = split_id_prefix(source, name);
    cur.name_span = Some(span);
    cur.id_override = id;

    // What is left of the first block opens the description — when there is any.
    let rest = trim_span(source, desc_start..block.end);
    if !rest.is_empty() {
        cur.children.push(ChildBlock::Description(rest));
    }
}

fn finalize_item<ID>(cur: &mut RawItem<ID>, source: &str) {
    cur.split_subissues();
    let span = cur
        .description()
        .map(|range| trim_span(source, range))
        .filter(|span| !span.is_empty());
    let text = span
        .as_ref()
        .map(|span| dedent_lines(&source[span.clone()], cur.content_indent));

    cur.content = match cur.link.take() {
        Some(link) => IssueContent::Linked {
            file: link.dest.into(),
            note: text,
        },
        None => text.map_or(IssueContent::Empty, IssueContent::Inline),
    };
    cur.content_span = span;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::IntIdGenerator;
    use crate::patch;

    fn plan_of(src: &str) -> Plan<u64> {
        parse::<u64, _>(src, &IntIdGenerator::new(1)).unwrap().plan
    }

    #[track_caller]
    fn assert_content(plan: &Plan<u64>, id: u64, expected: &str) {
        match &plan.get_issue(&id).unwrap().content {
            IssueContent::Inline(content) => assert_eq!(content, expected),
            other => panic!("expected Inline, got {other:?}"),
        }
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
    fn heading_markup_stays_out_of_the_set_name() {
        let plan = plan_of("# Mile `1` *и* [ссылка](x.md)\n\n- a\n");
        let names: Vec<&str> = plan.sets().keys().map(|set_name| set_name.as_str()).collect();
        assert_eq!(names, vec!["Mile 1 и ссылка"]);
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

    #[track_caller]
    fn assert_linked(plan: &Plan<u64>, id: u64, file: &str, note: Option<&str>) {
        match &plan.get_issue(&id).unwrap().content {
            IssueContent::Linked {
                file: linked_file,
                note: linked_note,
            } => {
                assert_eq!(linked_file.to_str().unwrap(), file);
                assert_eq!(linked_note.as_deref(), note);
            },
            other => panic!("expected Linked, got {other:?}"),
        }
    }

    #[test]
    fn filelink_issue() {
        let plan = plan_of("- [task K](task-k.md) trailing note\n  second line\n");
        assert_eq!(plan.get_issue(&1).unwrap().name, "task K");
        assert_linked(&plan, 1, "task-k.md", Some("trailing note\nsecond line"));
    }

    #[test]
    fn filelink_subissues_are_not_a_note() {
        let plan = plan_of("- [task K](task-k.md)\n  - subtask\n");
        assert_linked(&plan, 1, "task-k.md", None);
        assert_eq!(plan.get_issue(&2).unwrap().name, "subtask");
    }

    #[test]
    fn description_is_dedented() {
        let plan = plan_of("- g\n\n  Multi line\n  description\n");
        assert_eq!(plan.get_issue(&1).unwrap().name, "g");
        assert_content(&plan, 1, "Multi line\ndescription");
    }

    #[test]
    fn name_ends_at_the_first_line() {
        let plan = plan_of("- task D\n  One line description\n- task E\n");
        assert_eq!(plan.get_issue(&1).unwrap().name, "task D");
        assert_content(&plan, 1, "One line description");
        assert_eq!(plan.get_issue(&2).unwrap().name, "task E");
    }

    #[test]
    fn nested_description_is_dedented_to_its_own_level() {
        let plan = plan_of("- a\n  - b\n    Multi line\n    description\n");
        assert_eq!(plan.get_issue(&2).unwrap().name, "b");
        assert_content(&plan, 2, "Multi line\ndescription");
    }

    #[test]
    fn inline_markup_is_kept_in_the_name() {
        let plan = plan_of("- **bold name** and `code` rest\n");
        assert_eq!(plan.get_issue(&1).unwrap().name, "**bold name** and `code` rest");
    }

    #[test]
    fn name_span_points_at_the_raw_source() {
        let source = "- 25 **bold name** rest\n";
        let parsed = parse::<u64, _>(source, &IntIdGenerator::new(1)).unwrap();
        assert_eq!(&source[parsed.name_spans[&25].clone()], "**bold name** rest");
    }

    const NESTED_SOURCE: &str = "- parent\n  Desc line\n  - child\n- next\n";

    #[test]
    fn issue_span_covers_the_item_with_its_subtree() {
        let parsed = parse::<u64, _>(NESTED_SOURCE, &IntIdGenerator::new(1)).unwrap();
        let span_of = |id| &NESTED_SOURCE[parsed.issue_spans[&id].clone()];
        assert_eq!(span_of(1), "- parent\n  Desc line\n  - child\n");
        assert_eq!(span_of(2), "- child\n");
        assert_eq!(span_of(3), "- next\n");
    }

    #[test]
    fn content_span_keeps_the_source_indentation() {
        let parsed = parse::<u64, _>(NESTED_SOURCE, &IntIdGenerator::new(1)).unwrap();
        assert_eq!(&NESTED_SOURCE[parsed.content_spans[&1].clone()], "Desc line");
        assert_eq!(parsed.content_spans.get(&2), None, "an issue without a description");

        let source = "- g\n  Multi\n  line\n";
        let parsed = parse::<u64, _>(source, &IntIdGenerator::new(1)).unwrap();
        // the span carries the indentation; `IssueContent` holds it stripped
        assert_eq!(&source[parsed.content_spans[&1].clone()], "Multi\n  line");
        assert_content(&parsed.plan, 1, "Multi\nline");
    }

    #[test]
    fn patching_a_span_leaves_the_rest_byte_identical() {
        let parsed = parse::<u64, _>(NESTED_SOURCE, &IntIdGenerator::new(1)).unwrap();

        let renamed = patch::replace_range(NESTED_SOURCE, parsed.name_spans[&2].clone(), "renamed");
        assert_eq!(renamed, "- parent\n  Desc line\n  - renamed\n- next\n");

        let rewritten = patch::replace_range(NESTED_SOURCE, parsed.content_spans[&1].clone(), "New desc");
        assert_eq!(rewritten, "- parent\n  New desc\n  - child\n- next\n");

        let removed = patch::replace_range(NESTED_SOURCE, parsed.issue_spans[&1].clone(), "");
        assert_eq!(removed, "- next\n");
    }

    #[test]
    fn demoted_items_get_no_spans() {
        let source = "- a\n\n  desc\n\n  - sub\n\n  tail\n";
        let parsed = parse::<u64, _>(source, &IntIdGenerator::new(1)).unwrap();
        let ids: Vec<u64> = parsed.issue_spans.keys().copied().collect();
        assert_eq!(ids, vec![1]);
        assert_eq!(parsed.name_spans.len(), 1);
    }

    #[test]
    fn ordered_list_holds_no_issues() {
        let plan = plan_of("Steps:\n1. first\n2. second\n\n- a task\n");
        let names: Vec<&str> = plan.issues().values().map(|issue| issue.name.as_str()).collect();
        assert_eq!(names, vec!["a task"]);
    }

    #[test]
    fn ordered_list_inside_an_item_is_description() {
        let plan = plan_of("- task\n  1. first\n  2. second\n");
        let ids: Vec<u64> = plan.issues().keys().copied().collect();
        assert_eq!(ids, vec![1]);
        assert_content(&plan, 1, "1. first\n2. second");
    }

    #[test]
    fn bullets_under_an_ordered_list_are_not_issues() {
        let plan = plan_of("1. step\n   - not a task\n     - nor this\n");
        assert_eq!(plan.issues().len(), 0);
    }

    #[test]
    fn a_bullet_list_after_an_ordered_one_still_holds_issues() {
        let plan = plan_of("- task\n  1. step\n\n  - subtask\n");
        assert_eq!(*plan.get_issue(&1).unwrap(), {
            let mut issue = Issue::new(1, "task").with_subissue(2);
            issue.content = IssueContent::Inline("1. step".into());
            issue
        });
        assert_eq!(plan.get_issue(&2).unwrap().name, "subtask");
    }

    #[test]
    fn subissues_are_not_a_description() {
        let plan = plan_of("- a\n  - b\n");
        assert_eq!(plan.get_issue(&1).unwrap().content, IssueContent::Empty);
    }

    #[test]
    fn list_followed_by_description_is_description() {
        let plan = plan_of("- a\n\n  desc1\n\n  - sub\n\n  desc2\n");
        let ids: Vec<u64> = plan.issues().keys().copied().collect();
        assert_eq!(ids, vec![1], "a list before description content yields no issues");
        assert_content(&plan, 1, "desc1\n\n- sub\n\ndesc2");
    }

    #[test]
    fn only_the_trailing_list_is_subissues() {
        let plan = plan_of("- a\n  - sub1\n\n  desc\n\n  - sub2\n");
        assert_eq!(*plan.get_issue(&1).unwrap(), {
            let mut issue = Issue::new(1, "a").with_subissue(2);
            issue.content = IssueContent::Inline("- sub1\n\ndesc".into());
            issue
        });
        assert_eq!(*plan.get_issue(&2).unwrap(), Issue::new(2, "sub2").with_parent_id(1));
    }

    #[test]
    fn separator_keeps_a_closing_list_in_the_description() {
        let plan = plan_of("- task\n  intro:\n  - alpha\n  - beta\n  ---\n");
        let ids: Vec<u64> = plan.issues().keys().copied().collect();
        assert_eq!(ids, vec![1], "the closed list yields no issues");
        assert_content(&plan, 1, "intro:\n- alpha\n- beta\n---");
    }

    #[test]
    fn separator_is_shown_wherever_the_file_has_it() {
        assert_content(&plan_of("- task\n\n  ---\n"), 1, "---");
        assert_content(&plan_of("- task\n  ---\n"), 1, "---");
        assert_content(
            &plan_of("- task\n\n  above\n\n  ---\n\n  below\n"),
            1,
            "above\n\n---\n\nbelow",
        );
    }

    #[test]
    fn separator_under_the_name_line_stays_out_of_the_name() {
        let source = "- task\n  ---\n- next\n";
        let parsed = parse::<u64, _>(source, &IntIdGenerator::new(1)).unwrap();
        assert_eq!(parsed.plan.get_issue(&1).unwrap().name, "task");
        assert_eq!(&source[parsed.name_spans[&1].clone()], "task");
        assert_eq!(parsed.plan.get_issue(&2).unwrap().name, "next");
    }

    #[test]
    fn editing_the_description_takes_its_separator_along() {
        let source = "- task\n  intro:\n  - alpha\n  ---\n- next\n";
        let parsed = parse::<u64, _>(source, &IntIdGenerator::new(1)).unwrap();
        assert_content(&parsed.plan, 1, "intro:\n- alpha\n---");
        assert_eq!(&source[parsed.content_spans[&1].clone()], "intro:\n  - alpha\n  ---");

        let edited = patch::replace_range(source, parsed.content_spans[&1].clone(), "just text");
        assert_eq!(edited, "- task\n  just text\n- next\n");
    }

    #[test]
    fn demoted_list_drops_its_whole_subtree() {
        let plan = plan_of("- a\n\n  - sub\n    - deep\n\n  tail\n");
        let ids: Vec<u64> = plan.issues().keys().copied().collect();
        assert_eq!(ids, vec![1]);
        assert_content(&plan, 1, "- sub\n  - deep\n\ntail");
    }

    #[test]
    fn generated_id_skips_the_one_the_source_claims() {
        let plan = plan_of("- 25 named\n- auto\n- 1 explicit\n");
        assert_eq!(plan.get_issue(&25).unwrap().name, "named");
        assert_eq!(plan.get_issue(&2).unwrap().name, "auto");
        assert_eq!(plan.get_issue(&1).unwrap().name, "explicit");
    }

    #[test]
    fn repeated_explicit_id_is_an_error() {
        let Err(error) = parse::<u64, _>("- 7 first\n- 7 second\n", &IntIdGenerator::new(1)) else {
            panic!("a repeated id must be rejected");
        };
        assert_eq!(error.to_string(), "in line 2: issue id `7` is already used by `first`");
    }

    #[test]
    fn repeated_explicit_id_in_a_demoted_list_is_not_an_error() {
        let plan = plan_of("- 7 first\n\n  - 7 sub\n\n  tail\n");
        assert_eq!(plan.issues().len(), 1);
        assert_content(&plan, 7, "- 7 sub\n\ntail");
    }

    #[test]
    fn link_inside_the_name_is_not_a_file_reference() {
        let plan = plan_of("- task B [Mile 2](#mile-2)\n");
        let issue = plan.get_issue(&1).unwrap();
        assert_eq!(issue.name, "task B [Mile 2](#mile-2)");
        assert_eq!(issue.content, IssueContent::Empty);
    }
}
