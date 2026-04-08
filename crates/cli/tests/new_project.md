# New project

## New project default

```sh
$ todo new "project A"
    Creating `project A` project
```

```sh
$ ls "./project A"
Project.toml
```

```sh
$ cat "./project A/Project.toml"
id = "project A"
name = "project A"
```

```sh
$ todo new "project A"
    Creating `project A` project
Error: destination `${current_dir_path}/project A` already exists
```

## New project with manifest

```sh
$ todo new --with-manifest "project B"
    Creating `project B` project
```

```sh
$ ls "./project B"
project B.manifest.md
```

````sh
$ cat "./project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
````

```sh
$ todo new --with-manifest "project A"
    Creating `project A` project
Error: destination `${current_dir_path}/project A` already exists
```

```sh
$ todo new --with-manifest "project B"
    Creating `project B` project
Error: destination `${current_dir_path}/project B` already exists
```

```sh
$ todo new "project B"
    Creating `project B` project
Error: destination `${current_dir_path}/project B` already exists
```

# New subproject

## Prepare parent projects

```sh
$ todo new "project A"
    Creating `project A` project
```

```sh
$ todo new --with-manifest "project B"
    Creating `project B` project
```

## New subproject default

```sh
$ todo new "project A/project B"
    Creating `project B` project under `${current_dir_path}/project A`
```

```sh
$ ls "./project A/project B"
Project.toml
```

```sh
$ cat "./project A/project B/Project.toml"
id = "project B"
name = "project B"
```

```sh
$ todo new "project A/project B"
    Creating `project B` project under `${current_dir_path}/project A`
Error: destination `${current_dir_path}/project A/project B` already exists
```

## New subproject with manifest

```sh
$ todo new --with-manifest "project B/project B"
    Creating `project B` project under `${current_dir_path}/project B`
```

```sh
$ ls "./project B/project B"
project B.manifest.md
```

````sh
$ cat "./project B/project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
````

```sh
$ todo new --with-manifest "project B/project B"
    Creating `project B` project under `${current_dir_path}/project B`
Error: destination `${current_dir_path}/project B/project B` already exists
```

```sh
$ todo new "project B/project B"
    Creating `project B` project under `${current_dir_path}/project B`
Error: destination `${current_dir_path}/project B/project B` already exists
```
