# Add issue to project

## Prepare projects

```sh
$ todo new "project A"
    Creating `project A` project
```

```sh
$ todo new --with-manifest "project B"
    Creating `project B` project
```

## Add issue inside project dir

```sh
$ todo add "task 1"
Error: could not find `Project.toml` or `*.manifest.md` in `${current_dir_path}` or any parent directory
```

```sh
$ cd "project A"
$ todo add "task 1"
    Adding `task 1` issue to `project A` project
```

```sh
$ ls "project A"
Project.toml TODO.md
```

```sh
$ cat "project A/TODO.md"
- task 1
```

```sh
$ cd "project A"
$ todo add "task 2"
    Adding `task 2` issue to `project A` project
```

```sh
$ cat "project A/TODO.md"
- task 1
- task 2
```

```sh
$ cd "project A"
$ todo add "task 1"
    Adding `task 1` issue to `project A` project
Error: issue `task 1` in `${current_dir_path}` already exists
```

```sh
$ cd "project A"
$ todo add "task 2"
    Adding `task 2` issue to `project A` project
Error: issue `task 2` in `${current_dir_path}` already exists
```

```sh
$ ls "project A"
Project.toml TODO.md
```

```sh
$ cat "project A/TODO.md"
- task 1
- task 2
```

```sh
$ cd "project A"
$ todo add --first "task 3"
    Adding `task 3` issue to `project A` project
```

```sh
$ cat "project A/TODO.md"
- task 3
- task 1
- task 2
```

```sh
$ cd "project A"
$ todo add --last "task 4"
    Adding `task 4` issue to `project A` project
```

```sh
$ cat "project A/TODO.md"
- task 3
- task 1
- task 2
- task 4
```

## Add issue to specifyed project

```sh
$ todo add "task 1" --project "project A"
    Adding `task 1` issue to `project A` project
Error: issue `task 1` in `${current_dir_path}/project A` already exists
```

```sh
$ rm "project A/TODO.md"
$ ls "project A"
Project.toml
```

```sh
$ todo add "task 1" --project "project A"
    Adding `task 1` issue to `project A` project
```

```sh
$ ls "project A"
Project.toml TODO.md
```

```sh
$ cat "project A/TODO.md"
- task 1
```

```sh
$ todo add "task 2" --project "project A"
    Adding `task 2` issue to `project A` project
```

```sh
$ cat "project A/TODO.md"
- task 1
- task 2
```

```sh
$ todo add --first "task 3" --project "project A"
    Adding `task 3` issue to `project A` project
```

```sh
$ cat "project A/TODO.md"
- task 3
- task 1
- task 2
```

```sh
$ todo add --last "task 4" --project "project A"
    Adding `task 4` issue to `project A` project
```

```sh
$ cat "project A/TODO.md"
- task 3
- task 1
- task 2
- task 4
```

```sh
$ todo add "task 1" --project "project B"
    Adding `task 1` issue to `project B` project
```

```sh
$ ls "project B"
project B.manifest.md
```

````sh
$ cat "project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
```md todo
- task 1
```
````

```sh
$ todo add "task 1" --project "project B"
    Adding `task 1` issue to `project B` project
Error: issue `task 1` in `${current_dir_path}/project B` already exists
```

````sh
$ cat "project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
```md todo
- task 1
```
````

```sh
$ todo add "task 2" --project "project B"
    Adding `task 2` issue to `project B` project
```

```sh
$ ls "project B"
project B.manifest.md
```

````sh
$ cat "project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
```md todo
- task 1
- task 2
```
````

```sh
$ todo add --first --project "project B" "task 3"
    Adding `task 3` issue to `project B` project
```

````sh
$ cat "project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
```md todo
- task 3
- task 1
- task 2
```
````

```sh
$ todo add --last --project "project B" "task 4"
    Adding `task 4` issue to `project B` project
```

````sh
$ cat "project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
```md todo
- task 3
- task 1
- task 2
- task 4
```
````

# Add issue to subproject

## Prepare subprojects

```sh
$ todo new "project A"
    Creating `project A` project
```

```sh
$ todo new "project A/project C"
    Creating `project C` project under `${current_dir_path}/project A`
```

```sh
$ todo new --with-manifest "project A/project D"
    Creating `project D` project under `${current_dir_path}/project A`
```

## Add issue from parent project dir

```sh
$ cd "project A"
$ todo add --project "project C" "task 1"
    Adding `task 1` issue to `project C` project
```

```sh
$ ls "project A"
Project.toml project C project D
```

```sh
$ ls "project A/project C"
Project.toml TODO.md
```

```sh
$ cat "project A/project C/TODO.md"
- task 1
```

```sh
$ cd "project A"
$ todo add "task 2" --project "project C"
    Adding `task 2` issue to `project C` project
```

```sh
$ cat "project A/project C/TODO.md"
- task 1
- task 2
```

```sh
$ cd "project A"
$ todo add "task 1" --project "project C"
    Adding `task 1` issue to `project C` project
Error: issue `task 1` in `${current_dir_path}/project C` already exists
```

```sh
$ cd "project A"
$ todo add --project "project C" "task 2"
    Adding `task 2` issue to `project C` project
Error: issue `task 2` in `${current_dir_path}/project C` already exists
```

```sh
$ ls "project A"
Project.toml project C project D
```

```sh
$ ls "project A/project C"
Project.toml TODO.md
```

```sh
$ cat "project A/project C/TODO.md"
- task 1
- task 2
```

```sh
$ cd "project A"
$ todo add --first --project "project C" "task 3"
    Adding `task 3` issue to `project C` project
```

```sh
$ cat "project A/project C/TODO.md"
- task 3
- task 1
- task 2
```

```sh
$ cd "project A"
$ todo add --last --project "project C" "task 4"
    Adding `task 4` issue to `project C` project
```

```sh
$ cat "project A/project C/TODO.md"
- task 3
- task 1
- task 2
- task 4
```

## Add issue inside subproject dir

```sh
$ cd "project A/project D"
$ todo add "task 1"
    Adding `task 1` issue to `project D` project
```

```sh
$ ls "project A"
Project.toml project C project D
```

```sh
$ ls "project A/project D"
project D.manifest.md
```

````sh
$ cat "project A/project D/project D.manifest.md"
# project D

```toml project
id = "project D"
name = "project D"
```
```md todo
- task 1
```
````

```sh
$ cd "project A/project D"
$ todo add "task 1"
    Adding `task 1` issue to `project D` project
Error: issue `task 1` in `${current_dir_path}` already exists
```

````sh
$ cat "project A/project D/project D.manifest.md"
# project D

```toml project
id = "project D"
name = "project D"
```
```md todo
- task 1
```
````

```sh
$ cd "project A/project D"
$ todo add "task 2"
    Adding `task 2` issue to `project D` project
```

```sh
$ ls "project A"
Project.toml project C project D
```

```sh
$ ls "project A/project D"
project D.manifest.md
```

````sh
$ cat "project A/project D/project D.manifest.md"
# project D

```toml project
id = "project D"
name = "project D"
```
```md todo
- task 1
- task 2
```
````

```sh
$ cd "project A/project D"
$ todo add --first "task 3"
    Adding `task 3` issue to `project D` project
```

````sh
$ cat "project A/project D/project D.manifest.md"
# project D

```toml project
id = "project D"
name = "project D"
```
```md todo
- task 3
- task 1
- task 2
```
````

```sh
$ cd "project A/project D"
$ todo add --last "task 4"
    Adding `task 4` issue to `project D` project
```

````sh
$ cat "project A/project D/project D.manifest.md"
# project D

```toml project
id = "project D"
name = "project D"
```
```md todo
- task 3
- task 1
- task 2
- task 4
```
````

## Add issue to located project

```sh
$ todo add --project-id "project C" "task 5"
    Adding `task 5` issue to `project C` project
```

```sh
$ cat "project A/project C/TODO.md"
- task 3
- task 1
- task 2
- task 4
- task 5
```

```sh
$ todo add --project-name "project C" "task 6"
    Adding `task 6` issue to `project C` project
```

```sh
$ cat "project A/project C/TODO.md"
- task 3
- task 1
- task 2
- task 4
- task 5
- task 6
```

```sh
$ todo add --project-path "project A/project C" "task 7"
    Adding `task 7` issue to `project C` project
```

```sh
$ cat "project A/project C/TODO.md"
- task 3
- task 1
- task 2
- task 4
- task 5
- task 6
- task 7
```

```sh
$ todo add --project "project D" "task 5"
    Adding `task 5` issue to `project D` project
```

````sh
$ cat "project A/project D/project D.manifest.md"
# project D

```toml project
id = "project D"
name = "project D"
```
```md todo
- task 3
- task 1
- task 2
- task 4
- task 5
```
````
