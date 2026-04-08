# List

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

## List empty projects

```sh
$ todo list
Error: could not find `Project.toml` or `*.manifest.md` in `${current_dir_path}` or any parent directory
```

```sh
$ todo list .
List steps of 4 projects

[project A]: 0

[project A/project C]: 0

[project A/project D]: 0

[project B]: 0
```

```sh
$ todo list "project A"
List steps of 3 projects

[project A]: 0

[project A/project C]: 0

[project A/project D]: 0
```

```sh
$ todo list --project "project A"
List steps of 1 project

[project A]: 0
```

```sh
$ todo list --project "./project A"
List steps of 1 project

[project A]: 0
```

```sh
$ cd "project A"
$ todo list
List steps of 1 project

[project A]: 0
```

```sh
$ cd "project A"
$ todo list .
List steps of 3 projects

[project A]: 0

[project A/project C]: 0

[project A/project D]: 0
```

```sh
$ cd "project A/project C"
$ todo list
List steps of 1 project

[project C]: 0
```

## List one step

```sh
$ cd "project A"
$ todo add "task 1"
    Adding `task 1` issue to `project A` project
```

```sh
$ todo list .
List steps of 4 projects

[project A]: 1
- task 1

[project A/project C]: 0

[project A/project D]: 0

[project B]: 0
```

```sh
$ todo list "project A"
List steps of 3 projects

[project A]: 1
- task 1

[project A/project C]: 0

[project A/project D]: 0
```

```sh
$ todo list --project "project A"
List steps of 1 project

[project A]: 1
- task 1
```

```sh
$ cd "project A"
$ todo list
List steps of 1 project

[project A]: 1
- task 1
```

```sh
$ cd "project A"
$ todo list .
List steps of 3 projects

[project A]: 1
- task 1

[project A/project C]: 0

[project A/project D]: 0
```

## List multiple steps

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
$ todo list .
List steps of 4 projects

[project A]: 2
- task 1
- task 2

[project A/project C]: 1
- task 1

[project A/project D]: 0

[project B]: 0
```

```sh
$ todo list "project A"
List steps of 3 projects

[project A]: 2
- task 1
- task 2

[project A/project C]: 1
- task 1

[project A/project D]: 0
```

```sh
$ cd "project A"
$ todo list
List steps of 1 project

[project A]: 2
- task 1
- task 2
```

```sh
$ cd "project A/project C"
$ todo list
List steps of 1 project

[project C]: 1
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
$ todo list .
List steps of 4 projects

[project A]: 3
- task 1
- task 2
- task 3

[project A/project C]: 2
- task 1
- task 2

[project A/project D]: 1
- task D-1

[project B]: 0
```

```sh
$ todo list "project A"
List steps of 3 projects

[project A]: 3
- task 1
- task 2
- task 3

[project A/project C]: 2
- task 1
- task 2

[project A/project D]: 1
- task D-1
```

```sh
$ todo list --project "project A"
List steps of 1 project

[project A]: 3
- task 1
- task 2
- task 3
```

```sh
$ cd "project A/project C"
$ todo list
List steps of 1 project

[project C]: 2
- task 1
- task 2
```

```sh
$ todo list --project "project D"
List steps of 1 project

[project D]: 1
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
$ todo list .
List steps of 4 projects

[project A]: 3
- task 1
- task 2
- task 3

[project A/project C]: 2
- task 1
- task 2

[project A/project D]: 1
- task D-1

[project B]: 2
- task B-2
- task B-1
```

```sh
$ todo list --project "project B"
List steps of 1 project

[project B]: 2
- task B-2
- task B-1
```
