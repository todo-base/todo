# List complex

## Prepare project

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

## List steps from complex file

```sh
$ todo list .
List steps of 2 projects

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
$ todo list
List steps of 1 project

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
$ todo list
List steps of 1 project

[project B]: 6
- To complete task 1
- Some other task 2

# Milestone A

- Some other task 3
- Some other task 4
- final task
```
