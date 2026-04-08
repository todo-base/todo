# Tree complex

## Without subprojects

### Prepare

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

args:
--project (-p)
--project-id
--project-name
--project-path

      UI         STORAGE
-----------------------------
|  cli  web  |  tracker-fs  |
|  ...       |  tracker-db  |  INTERFACE
|            |  ...         |
-----------------------------
|           app             |  APP LOGIC
-----------------------------
|           lib             |  DOMAIN
-----------------------------

- final task
" > "project A/TODO.md"
```

```sh
$ todo new "project B" --with-manifest
    Creating `project B` project
```

````sh
$ echo r#"# project B

```toml project
id = "project B"
name = "project B"
```
```md todo
- To complete task 1
  - Subtask 1
  - Subtask 2
- Some other task 2

# Milestone A

- Some other task 3
  - Subtask 3
- Some other task 4

args:
--project (-p)
--project-id
--project-name
--project-path

      UI         STORAGE
-----------------------------
|  cli  web  |  tracker-fs  |
|  ...       |  tracker-db  |  INTERFACE
|            |  ...         |
-----------------------------
|           app             |  APP LOGIC
-----------------------------
|           lib             |  DOMAIN
-----------------------------

- final task
"# > "project B/project B.manifest.md"
````

### Trees

```sh
$ todo tree .
Trees of 2 projects

[project A]: 6
  - To complete task 1
  - Some other task 2

  # Milestone A

  - Some other task 3
  - Some other task 4
  - final task

[project B]: 6
  - To complete task 1
  - Some other task 2

  # Milestone A

  - Some other task 3
  - Some other task 4
  - final task
```

```sh
$ cd "project A"
$ todo tree
Tree of 1 project

[project A]: 6
  - To complete task 1
  - Some other task 2

  # Milestone A

  - Some other task 3
  - Some other task 4
  - final task
```

```sh
$ cd "project B"
$ todo tree
Tree of 1 project

[project B]: 6
  - To complete task 1
  - Some other task 2

  # Milestone A

  - Some other task 3
  - Some other task 4
  - final task
```

## With subprojects

### Prepare

```sh
$ cd "project A"
$ todo new "project C"
    Creating `project C` project
```

```sh
$ echo "- To complete task 1
  - Subtask 1
  - Subtask 2
- Some other task 2
" > "project A/project C/TODO.md"
```

```sh
$ cd "project B"
$ todo new "project D"
    Creating `project D` project
```

```sh
$ echo "- To complete task 1
  - Subtask 1
" > "project B/project D/TODO.md"
```

### Trees

```sh
$ todo tree .
Trees of 4 projects

[project A]: 6
  │  - To complete task 1
  │  - Some other task 2
  │
  │  # Milestone A
  │
  │  - Some other task 3
  │  - Some other task 4
  │  - final task
  │
  └─ [project C]: 2
       - To complete task 1
       - Some other task 2

[project B]: 6
  │  - To complete task 1
  │  - Some other task 2
  │
  │  # Milestone A
  │
  │  - Some other task 3
  │  - Some other task 4
  │  - final task
  │
  └─ [project D]: 1
       - To complete task 1
```

```sh
$ echo r#"[display.project.title]
consist = "id"
id_before = "["
id_after = "]"
show_steps_count = true

[display.project]
max_steps = 4
show_substeps = true
compact = false
separate_projects = true
"# > "todo.toml"
```

```sh
$ todo tree .
Trees of 4 projects

[project A]: 9
  │  - To complete task 1
  │    - Subtask 1
  │    - Subtask 2
  │  - Some other task 2
  │  ..5
  │
  └─ [project C]: 4
       - To complete task 1
         - Subtask 1
         - Subtask 2
       - Some other task 2

[project B]: 9
  │  - To complete task 1
  │    - Subtask 1
  │    - Subtask 2
  │  - Some other task 2
  │  ..5
  │
  └─ [project D]: 2
       - To complete task 1
         - Subtask 1
```

```sh
$ todo tree --max-steps 6 "project A"
Trees of 2 projects

[project A]: 9
  │  - To complete task 1
  │    - Subtask 1
  │    - Subtask 2
  │  - Some other task 2
  │
  │  # Milestone A
  │
  │  - Some other task 3
  │  ..3
  │
  └─ [project C]: 4
       - To complete task 1
         - Subtask 1
         - Subtask 2
       - Some other task 2
```

```sh
$ todo tree --max-steps 7 "project B"
Trees of 2 projects

[project B]: 9
  │  - To complete task 1
  │    - Subtask 1
  │    - Subtask 2
  │  - Some other task 2
  │
  │  # Milestone A
  │
  │  - Some other task 3
  │    - Subtask 3
  │  ..2
  │
  └─ [project D]: 2
       - To complete task 1
         - Subtask 1
```
