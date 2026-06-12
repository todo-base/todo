# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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
