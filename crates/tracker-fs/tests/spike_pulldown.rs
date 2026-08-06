//! Spike: inspect pulldown-cmark events and source ranges.
//!
//! Run with:
//!   cargo test -p todo-tracker-fs --test spike_pulldown -- --ignored --nocapture

use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Fixture covering nested issues, `---` separators, headings, inline
/// descriptions, code-block-in-description and a `[name](file)` filelink with note.
const SAMPLE: &str = r#"
- task A
  - task AA

- task B [Mile 2](#mile-2)

---

- task C

# Mile 1

- task D
  One line description
- task E
  - task EA
  - task EB
- task F

---

- task G
  Multi line
  description

  ```code
  block
  ```

- task H
  - task HA
    - task HAA
      Deep level description

    - task HAB
  - task HB
- task I

# Mile 2

- [task K](task-basic.md) Trailing description first line.
  Second line
"#;

fn gfm_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options
}

#[test]
#[ignore = "spike: dump full event stream with byte ranges"]
fn dump_events() {
    for (event, range) in Parser::new_ext(SAMPLE, gfm_options()).into_offset_iter() {
        println!("{range:?}  {event:?}");
    }
}

#[test]
#[ignore = "spike: inspect list-item Start/End ranges to choose the span formula"]
fn item_ranges() {
    let mut list_depth = 0usize;
    // (depth, Start(Item) range, End(Item) range) collected in document order.
    let mut starts: Vec<(usize, Range<usize>)> = vec![];
    let mut ends: Vec<(usize, Range<usize>)> = vec![];

    for (event, range) in Parser::new_ext(SAMPLE, gfm_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::List(_)) => list_depth += 1,
            Event::End(TagEnd::List(_)) => list_depth -= 1,
            Event::Start(Tag::Item) => starts.push((list_depth, range)),
            Event::End(TagEnd::Item) => ends.push((list_depth, range)),
            _ => {},
        }
    }

    // Starts arrive in pre-order, ends in post-order; sorting both by range start
    // makes them pairable by index.
    starts.sort_by_key(|(_, range)| range.start);
    ends.sort_by_key(|(_, range)| range.start);

    println!("== {} items ==", starts.len());
    for (i, ((depth, start), (_, end))) in starts.iter().zip(&ends).enumerate() {
        let span = start.start..end.end;
        let slice = &SAMPLE[span.clone()];
        println!(
            "\n[{i}] depth={depth} (start_range={start:?}, end_range={end:?}, full={span:?})\n-----\n{slice}-----"
        );
    }

    assert_eq!(starts.len(), ends.len(), "Start(Item)/End(Item) counts must match");
}
