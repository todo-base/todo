# Todo

A minimalist, universal and locally-oriented task manager.

## Features

- Create and init new project
- Add issue to the project
- Rename an issue in place, leaving the rest of the file untouched
- List issues of the projects
- View tree of the projects
- Use local file-based project config and issue storage
- Search for projects in the current directory and in configured search roots

## Installation

Make sure you have Rust installed:

```bash
rustc --version
```

Install the CLI-application using `cargo`:

```bash
cargo install --git https://github.com/noogen-projects/todo
```

Or clone and build manually:

```bash
git clone https://github.com/noogen-projects/todo
cd todo
cargo build --release
```

## Usage

Create a new project:

```sh
$ todo new "Life goals"
    Creating `Life goals` project
```

Add issues to the project:

```sh
$ cd "Life goals"
$ todo add "plant a tree"
    Adding `plant a tree` issue to `Life goals` project
$ todo add "build a house"
    Adding `build a house` issue to `Life goals` project
$ todo add --first "raise a son"
    Adding `raise a son` issue to `Life goals` project
```

Rename an issue:

```sh
$ todo rename "plant a tree" "plant an oak"
    Renaming `plant a tree` issue in `Life goals` project
```

List issues of the project:

```sh
$ todo list
List items of 1 project

[Life goals]: 3
- raise a son
- plant an oak
- build a house
```

### Examples

For more advanced usage, including the `tree` command and using subprojects, see examples in `.md`-files in the [`./cli/tests/`](./cli/tests/) directory.

## Data Storage

Currently, `todo` supports only a simple file system storage. Projects are stored in a directory with a `Project.toml` file or a `*.manifest.md` file. Issues are stored in the manifest file or in the `TODO.md` file in the project root directory.

### Plan format

Issue files are plain CommonMark, so they stay readable and editable by hand. The
reader maps the markup like this:

```markdown
# Milestone A                  <- heading: the set the issues below belong to

- 12 buy the seedlings         <- bullet item: an issue; `12 ` is its id
  Pick a two-year-old one.     <- indented to the item: the issue description

  Any nursery will do.
- plant a tree                 <- an issue without an explicit id gets one
  - dig the hole               <- a list closing the item: its subissues
  - water it
- [build a house](house.md)    <- a link opening the line: the issue content
                                  lives in that file
```

Rules worth knowing:

- Only **bullet** lists (`-`, `*`, `+`) carry issues. A numbered list is ordinary
  text, and so is anything nested under it — handy for prose that just enumerates.
- The **first line** of an item is the issue name; everything else indented to the
  item is its description.
- A leading `12 ` is read as the issue **id**. Ids must be unique: a repeated one
  is an error, and generated ids never take an id the file already spells out.
- A nested list is **subissues only when it closes the item**. With description
  content after it, it stays part of that description. To keep a description that
  ends with a list from being read as subissues, close it with a `---`; `todo`
  adds that line for you when it writes such a description.
- Anything else — paragraphs between issues, numbered lists, ASCII art — is
  ignored, so notes can live next to the plan.

In a `*.manifest.md` file the same markup lives inside a ` ```md todo ` fenced
block.

## License

This project is licensed under the [MIT](./LICENSE) License.

## Contributing

Contributions are welcome! Feel free to open issues and submit pull requests.
