# Duplicate issue id

## Prepare project

```sh
$ todo new "project A"
    Creating `project A` project
```

## An id spelled out twice is rejected

```sh
$ echo "- 7 first task
- second task
- 7 duplicate
" > "project A/TODO.md"
```

```sh
$ cd "project A"
$ todo list
Error: in line 3: issue id `7` is already used by `first task`
```

## A generated id never takes one the source claims

An issue without an id keeps the one the source spells out further down.

```sh
$ echo "- 25 named
- auto
- 1 explicit
" > "project A/TODO.md"
```

```sh
$ cd "project A"
$ todo list
List items of 1 project

[project A]: 3
- named
- auto
- explicit
```
