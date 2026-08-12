# Rename issue

## Prepare project

```sh
$ todo new "project A"
    Creating `project A` project
```

## Rename issue (in-place, WholeFile)

```sh
$ cd "project A"
$ todo add "task 1"
    Adding `task 1` issue to `project A` project
```

```sh
$ cd "project A"
$ todo rename "task 1" "renamed task"
    Renaming `task 1` issue in `project A` project
```

```sh
$ cat "project A/TODO.md"
- renamed task
```

## Only the target name is patched

Descriptions, nesting, markup and ids around the target stay byte-identical.

```sh
$ echo "- parent task
  Description line
  - child task
- 5 numbered
  Note of the numbered issue
" > "project A/TODO.md"
```

```sh
$ cd "project A"
$ todo rename "parent task" "**renamed** parent"
    Renaming `parent task` issue in `project A` project
```

```sh
$ cat "project A/TODO.md"
- **renamed** parent
  Description line
  - child task
- 5 numbered
  Note of the numbered issue
```

## Rename a nested issue

```sh
$ cd "project A"
$ todo rename "child task" "child renamed"
    Renaming `child task` issue in `project A` project
```

```sh
$ cat "project A/TODO.md"
- **renamed** parent
  Description line
  - child renamed
- 5 numbered
  Note of the numbered issue
```

## Error: issue not found

```sh
$ cd "project A"
$ todo rename "nope" "x"
    Renaming `nope` issue in `project A` project
Error: issue `nope` not found
```

## Error: the new name would not read back as a name

```sh
$ cd "project A"
$ todo rename "child renamed" "12 boom"
    Renaming `child renamed` issue in `project A` project
Error: invalid issue name: must not start with `12` — it would be read back as an issue id
```

```sh
$ cd "project A"
$ todo rename "child renamed" ""
    Renaming `child renamed` issue in `project A` project
Error: invalid issue name: must not be empty
```

## Error: the new name is already taken

```sh
$ cd "project A"
$ todo rename "child renamed" "numbered"
    Renaming `child renamed` issue in `project A` project
Error: issue `numbered` already exists
```

## Rename inside a manifest `md todo` block

```sh
$ todo new "project B" --with-manifest
    Creating `project B` project
```

```sh
$ cd "project B"
$ todo add "task one"
    Adding `task one` issue to `project B` project
```

```sh
$ cd "project B"
$ todo rename "task one" "renamed one"
    Renaming `task one` issue in `project B` project
```

````sh
$ cat "project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
```md todo
- renamed one
```
````
