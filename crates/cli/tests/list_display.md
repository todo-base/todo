# List display

## Prepare projects

```sh
$ todo new "project A"
    Creating `project A` project
```

```sh
$ echo "- To complete task 1
  - Subtask 1
  - Subtask 2
- Some other task 2

# Milestone A

- Some other task 3
  - Subtask 3
- Some other task 4
  Description of some other task 4

- final task
" > "project A/TODO.md"
```

```sh
$ todo new "project B" --with-manifest
    Creating `project B` project
```

```sh
$ todo add "task B-1" --project "project B"
    Adding `task B-1` issue to `project B` project
```

## List pretty

```sh
$ todo list --pretty .
List steps of 2 projects

[project A]: 6
- To complete task 1
- Some other task 2

# Milestone A

- Some other task 3
- Some other task 4
- final task

[project B]: 1
- task B-1
```

## List compact

```sh
$ todo list --compact .
[project A]: 6
- To complete task 1
- Some other task 2
# Milestone A
- Some other task 3
- Some other task 4
- final task
[project B]: 1
- task B-1
```

## Prepare display config

```sh
$ echo r#"[display.project.title]
consist = "id"
id_before = ""
id_after = ""

[display.project]
max_steps = 4
compact = true
"# > "todo.toml"
```

## List configured

```sh
$ todo list .
project A: 6
- To complete task 1
- Some other task 2
# Milestone A
- Some other task 3
..2
project B: 1
- task B-1
```

```sh
$ todo list . --pretty
List steps of 2 projects

project A: 6
- To complete task 1
- Some other task 2

# Milestone A

- Some other task 3
..2

project B: 1
- task B-1
```

```sh
$ todo list . --pretty --max-steps 5
List steps of 2 projects

project A: 6
- To complete task 1
- Some other task 2

# Milestone A

- Some other task 3
- Some other task 4
..1

project B: 1
- task B-1
```

## Prepare display id and name

```sh
$ echo r#"[display.project.title]
consist = "id_and_name"
id_before = "["
id_after = "]"

[display.project]
max_steps = 2
compact = true
separate_projects = true
"# > "todo.toml"
```

```sh
$ echo r#"
id = "project-a"
name = "A"
"# > "project A/Project.toml"
```

## List configured

```sh
$ todo list .
[project B]: 1
- task B-1

[project-a] A: 6
- To complete task 1
- Some other task 2
..4
```
