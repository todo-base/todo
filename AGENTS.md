# AGENTS.md

> **NOTE** *(for humans only)*: **Why AGENTS.md?..** One AGENTS.md works across many agents (see [agents.md](https://agents.md/)).

## Commands

```bash
# Build
cargo build
cargo build --release

# Test all workspace members
cargo test --workspace

# Test a specific crate
cargo test -p todo-cli
cargo test -p todo-tracker-fs

# Run a single test by name
cargo test -p todo-cli add_issue_test_cases

# Format code
cargo +nightly fmt --all

# Lint
cargo clippy --workspace
```

## Architecture

This is a Rust workspace implementing a local file-based task manager CLI. All crates live under `crates/`. The layers (see `doc/layers.md`) separate UI / storage interfaces from app logic and domain:

```
cli / (future: web)       ← UI layer
        ↓
      app                 ← application logic, config, project discovery
        ↓
      lib                 ← domain models (Project, Issue, Plan)
        ↑
  tracker-fs              ← file system storage (reads/writes TOML + Markdown)
```

- **`lib`** (`todo-lib`, `crates/lib/`): Pure domain types — `Project`, `Issue`, `Plan`, `Id`. No I/O.
- **`app`** (`todo-app`, `crates/app/`): Business logic. Loads config, locates projects, coordinates between CLI and storage. Key files: `config.rs` (config loading), `target.rs` (Path/Id/Name resolution), `project.rs` + `project/metadata.rs` (project location), `issue.rs`.
- **`tracker-fs`** (`todo-tracker-fs`, `crates/tracker-fs/`): File system storage, fronted by `tracker.rs`. Projects are described by `Project.toml` or `*.manifest.md` files; issues live in the manifest or `TODO.md`. Plan/task parsing from Markdown is in `plan/parse.rs`; file layout decisions in `placement.rs`; file writing in `generator.rs`. Project discovery via `walkdir`.
- **`cli`** (`todo-cli`, `crates/cli/`): Binary `todo` (`main.rs`). Clap-based argument parsing in `opts.rs` (`--config`-style overrides in `opts/config_override.rs`), command dispatch in `command.rs`, output formatting in `display.rs`.
- **`tests`** (`crates/tests/`): Shared test helpers (`init_logger`, `target_build_dir`).

## Testing Approach

CLI integration tests use the `md-cli-test` crate: each `.rs` test file in `crates/cli/tests/` loads a corresponding `.md` file that contains shell command blocks and expected output. This means test cases are in the `.md` files, not the `.rs` files. When adding or modifying CLI behavior, update the relevant `.md` test file.

Tracker-FS tests use `temp_testdir` to create isolated directories per test and `function_name!` to derive unique directory names.

## Configuration

The app loads `todo.toml` in layers (via the `config` + `config-load` crates), later sources overriding earlier ones: `~/.todo/todo.toml`, then `todo.toml` searched upward from CWD through parent directories. The path of the root (home) config can be overridden with the `TODO_ROOT_CONFIG` env var, and individual fields with `TODO_CONFIG_*` env vars (e.g. `TODO_CONFIG_HTTP_PORT=8090` sets `http.port`). Config controls project search paths, default project location, and display options. See `crates/app/src/config.rs`.

## Code conventions

### Naming — no single-letter variables, no ad-hoc acronyms

- Do NOT use single-letter variable names (`s`, `e`, `n`, `c`, …) where a full,
  readable name communicates intent (`start`, `end`, `count`, `byte`, …).
  Applies to byte/offset cursors, temporaries, accumulators, closure parameters
  and match/destructuring bindings.
- Single letters ARE allowed where they follow an established convention:
  - loop and index counters: `i`, `j`, `k` (incl. `.enumerate()`);
  - comparator / reducer pairs: `sort_by(|a, b| …)`, `fold(|acc, x| …)`;
  - the callable parameter of a higher-order helper: `f`;
  - mathematical formulas, mirroring their notation: `x`, `y`, `n`, `t`.
- NEVER shorten a multi-word name to an acronym of its initials:
  `ns` ← `name_span`, `cs` ← `content_span`, `pc` ← `project_config`.
- To shorten, DROP the words the context already provides and keep a whole word
  of the phrase — the one that carries the meaning here:
  `name_span` → `span`, `raw_item` → `item`, `project_root_dir` → `root_dir`.
- Keep the balance: shorten only while the name still reads on its own. If the
  qualifier carries information, KEEP it — `|set_name|` beats `|name|` even when
  the receiving `sets()` call is right there.
- Abbreviations that are common in Rust and in most projects ARE allowed and need
  no expansion: `idx`, `cur`, `desc`, `dest`, `src`, `len`, `ptr`, `buf`, `ch`,
  `err`, `dir`, `env`, `cfg`, `gen`, `meta`, `args`, `id`, `fs`, `io`, `str`, …
  The same goes for abbreviations a dependency uses in its own API.

```rust
// GOOD — the surrounding code already says which span it is
if let Some(span) = name_span {
    name_spans.insert(id.clone(), span);
}
```

```rust
// BAD — acronym of the initials
if let Some(ns) = name_span {
    name_spans.insert(id.clone(), ns);
}
```

### Comments and docs — short, result only

- Write a comment ONLY where it adds something the code cannot state itself.
  No comment beats a comment that restates the signature.
- Describe the FINAL result. NEVER record how the code got there: no notes from
  a discussion, no contrast with a rejected alternative (`— not the file's
  content`), no history (`was a Cow before`, `renamed from …`), no phase or
  ticket chatter.
- Keep it to one or two lines. Cut hedges, repetitions, and examples that an
  adjacent test already shows.

```rust
// GOOD — states what the type is and what it is for
/// Captured file metadata (modification time + size) used to detect that a file
/// has not changed between reading and writing it.
```

```rust
// BAD — carries an argument from the discussion + a stale implementation note
/// Captured file metadata (modification time + size) — not the file's content.
/// Keeps the parsed zero-copy content of untouched issues valid (Phase 3).
```

### DRY — never duplicate code

- NEVER duplicate logic. Write code to be reusable, and ALWAYS reuse an existing
  helper instead of copy-pasting.
- Before writing a loop/helper that trims, parses, converts, formats, etc.,
  check whether one already exists (in the crate or workspace) and CALL it.
- If ≥2 lines of identical logic appear twice, extract them into a helper (or
  reuse the existing one). Refactor duplication on sight, not "later".
- Applies to edits: when new code mirrors existing logic, reuse the existing
  function — never inline a second copy.

### Conversions for `impl AsRef<T> + Into<String>` parameters

This rule applies ONLY to parameters declared with the dual bound
`impl AsRef<str> + Into<String>` (cheap borrow + owned conversion).

- Convert **only inside the body of the function that declares the bound**:
  - need a `&str` → `value.as_ref()`;
  - need an owned value → prefer **`value.into()`**; type inference picks the
    concrete type (`String`), which then satisfies e.g. `impl Into<Cow<'src, str>>`.
- NEVER route an owned value through a redundant copy — no
  `value.as_ref().to_string()` / `.to_owned()`.
- Do NOT bind an intermediate that is used only once
  (`let x: ConcreteType = value.into();`); convert inline where it is used.
- **At the call site pass the argument as is** — do NOT add explicit `.into()`
  or `.as_ref()`. The callee's generic bound already accepts the value.

```rust
// GOOD — caller passes the value as is; the callee converts in its body
issue::add(ProjectData::Fs(project_metadata), &config.source, order, issue_name, "")?;

pub fn add(name: impl AsRef<str> + Into<String>, /* … */) -> io::Result<()> {
    let name_ref = name.as_ref();           // borrow to read
    // …
    let issue = Issue::new(0, name.into()); // own via `into`
}
```

```rust
// BAD — conversion pushed onto the caller
issue::add(ProjectData::Fs(project_metadata), &config.source, order, issue_name.into(), "")?;
issue::add(ProjectData::Fs(project_metadata), &config.source, order, issue_name.as_ref(), "")?;
```

```rust
// BAD — over-engineered body: redundant copy + throwaway bindings
let name = name.as_ref().to_string();
let content: String = content.into();
let issue = Issue::new(0, name).with_content(content);
```
