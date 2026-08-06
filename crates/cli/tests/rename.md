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

## Rename a nested issue (only the target region is patched)

```sh
$ cd "project A"
$ todo add --last "parent"
    Adding `parent` issue to `project A` project
```

```sh
$ cd "project A"
$ todo rename "parent" "parent renamed"
    Renaming `parent` issue in `project A` project
```

```sh
$ cat "project A/TODO.md"
- renamed task
- parent renamed
```

## Error: issue not found

```sh
$ cd "project A"
$ todo rename "nope" "x"
    Renaming `nope` issue in `project A` project
Error: issue `nope` not found
```
