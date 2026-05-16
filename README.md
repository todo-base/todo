# Todo

A minimalist, universal and locally-oriented task manager.

## Features

- Create and init new project
- Add issue to the project
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

List issues of the project:

```sh
$ todo list
List items of 1 project

[Life goals]: 3
- raise a son
- plant a tree
- build a house
```

### Examples

For more advanced usage, including the `tree` command and using subprojects, see examples in `.md`-files in the [`./cli/tests/`](./cli/tests/) directory.

## Data Storage

Currently, `todo` supports only a simple file system storage. Projects are stored in a directory with a `Project.toml` file or a `*.manifest.md` file. Issues are stored in the manifest file or in the `TODO.md` file in the project root directory.

## License

This project is licensed under the [MIT](./LICENSE) License.

## Contributing

Contributions are welcome! Feel free to open issues and submit pull requests.
