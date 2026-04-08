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

This is a Rust workspace implementing a local file-based task manager CLI. The layers are:

```
cli / (future: web)       ← UI layer
        ↓
      app                 ← application logic, config, project discovery
        ↓
      lib                 ← domain models (Project, Issue, Plan)
        ↑
  tracker-fs              ← file system storage (reads/writes TOML + Markdown)
```

- **`lib`** (`todo-lib`): Pure domain types — `Project`, `Issue`, `Plan`, `Id`. No I/O.
- **`app`** (`todo-app`): Business logic. Loads config, locates projects, coordinates between CLI and storage. Key files: `config.rs` (config loading), `target.rs` (Path/Id/Name resolution), `lib.rs` (project location logic).
- **`tracker-fs`** (`todo-tracker-fs`): File system storage. Projects are described by `Project.toml` or `*.manifest.md` files; issues live in `TODO.md`. Plan/task parsing from Markdown is in `plan/parse.rs`. Project discovery via `walkdir`.
- **`cli`** (`todo-cli`): Binary `todo`. Clap-based argument parsing in `opts.rs`, command dispatch in `command.rs`, output formatting in `display.rs`.
- **`tests`**: Shared test helpers (`init_logger`, `target_build_dir`).

## Testing Approach

CLI integration tests use the `md-cli-test` crate: each `.rs` test file in `cli/tests/` loads a corresponding `.md` file that contains shell command blocks and expected output. This means test cases are in the `.md` files, not the `.rs` files. When adding or modifying CLI behavior, update the relevant `.md` test file.

Tracker-FS tests use `temp_testdir` to create isolated directories per test and `function_name!` to derive unique directory names.

## Configuration

The app loads config from `todo.toml` (searched upward from CWD and in `~/.config/todo/`). The `config` and `config-load` crates are used for layered config loading. Config controls project search paths, default project location, and display options.
