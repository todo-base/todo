# Tree

## Prepare projects

```sh
$ todo new "project A"
    Creating `project A` project
```

```sh
$ todo new "project B"
    Creating `project B` project
```

```sh
$ todo new --with-manifest "project A/project C"
    Creating `project C` project under `${current_dir_path}/project A`
```

```sh
$ todo new "project A/project D"
    Creating `project D` project under `${current_dir_path}/project A`
```

## Tree empty projects

```sh
$ todo tree
Error: could not find `Project.toml` or `*.manifest.md` in `${current_dir_path}` or any parent directory
```

```sh
$ todo tree --pretty .
Trees of 4 projects

[project A]: 0
  │
  ├─ [project C]: 0
  │
  └─ [project D]: 0

[project B]: 0
```

```sh
$ todo tree --compact .
[project A]: 0
  ├─ [project C]: 0
  └─ [project D]: 0
[project B]: 0
```

```sh
$ echo r#"[display.project.title]
consist = "id"
id_before = ""
id_after = ""
show_steps_count = false

[display.project]
max_steps = 3
compact = true
separate_projects = false
"# > "todo.toml"
```

```sh
$ todo tree .
project A
  ├─ project C
  └─ project D
project B
```

```sh
$ todo tree "project A"
project A
  ├─ project C
  └─ project D
```

```sh
$ cd "project A"
$ todo tree
project A
  ├─ project C
  └─ project D
```

```sh
$ cd "project A/project C"
$ todo tree
project C
```

## Tree with one step

```sh
$ cd "project A"
$ todo add "task 1"
    Adding `task 1` issue to `project A` project
```

```sh
$ todo tree .
project A
  │  - task 1
  ├─ project C
  └─ project D
project B
```

```sh
$ todo tree "project A"
project A
  │  - task 1
  ├─ project C
  └─ project D
```

```sh
$ cd "project A"
$ todo tree
project A
  │  - task 1
  ├─ project C
  └─ project D
```

## Tree with multiple steps

```sh
$ cd "project A"
$ todo add "task 2"
    Adding `task 2` issue to `project A` project
```

```sh
$ cd "project A/project C"
$ todo add "task 1"
    Adding `task 1` issue to `project C` project
```

```sh
$ todo tree .
project A
  │  - task 1
  │  - task 2
  ├─ project C
  │    - task 1
  └─ project D
project B
```

```sh
$ todo tree "project A"
project A
  │  - task 1
  │  - task 2
  ├─ project C
  │    - task 1
  └─ project D
```

```sh
$ cd "project A"
$ todo tree
project A
  │  - task 1
  │  - task 2
  ├─ project C
  │    - task 1
  └─ project D
```

```sh
$ cd "project A/project C"
$ todo tree --pretty
Tree of 1 project

project C
  - task 1
```

```sh
$ cd "project A"
$ todo add "task 3"
    Adding `task 3` issue to `project A` project
```

```sh
$ cd "project A/project C"
$ todo add "task 2"
    Adding `task 2` issue to `project C` project
```

```sh
$ cd "project A/project D"
$ todo add "task D-1"
    Adding `task D-1` issue to `project D` project
```

```sh
$ todo tree .
project A
  │  - task 1
  │  - task 2
  │  - task 3
  ├─ project C
  │    - task 1
  │    - task 2
  └─ project D
       - task D-1
project B
```

```sh
$ todo tree "project A"
project A
  │  - task 1
  │  - task 2
  │  - task 3
  ├─ project C
  │    - task 1
  │    - task 2
  └─ project D
       - task D-1
```

```sh
$ cd "project A"
$ todo tree
project A
  │  - task 1
  │  - task 2
  │  - task 3
  ├─ project C
  │    - task 1
  │    - task 2
  └─ project D
       - task D-1
```

```sh
$ cd "project A/project C"
$ todo tree
project C
  - task 1
  - task 2
```

```sh
$ cd "project A/project D"
$ todo tree
project D
  - task D-1
```

```sh
$ cd "project B"
$ todo add "task B-1"
    Adding `task B-1` issue to `project B` project
```

```sh
$ cd "project B"
$ todo add --first "task B-2"
    Adding `task B-2` issue to `project B` project
```

```sh
$ todo tree .
project A
  │  - task 1
  │  - task 2
  │  - task 3
  ├─ project C
  │    - task 1
  │    - task 2
  └─ project D
       - task D-1
project B
  - task B-2
  - task B-1
```

```sh
$ cd "project B"
$ todo tree
project B
  - task B-2
  - task B-1
```
