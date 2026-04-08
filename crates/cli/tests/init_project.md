# Init project outside project dir

## Prepare directories

```sh
$ mkdir "project A"
$ mkdir "project B"
```

## Init project with config

```sh
$ todo init "project A"
    Initializing `project A` project
```

```sh
$ ls "project A"
Project.toml
```

```sh
$ cat "project A/Project.toml"
id = "project A"
name = "project A"
```

```sh
$ todo init "project A"
    Initializing `project A` project
Error: destination `${current_dir_path}/project A/Project.toml` already exists
```

```sh
$ todo init "project C"
    Initializing `project C` project
Error: destination `${current_dir_path}/project C` does not exists
```

## Init project with manifest

```sh
$ todo init --with-manifest "project B"
    Initializing `project B` project
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
````

```sh
$ todo init --with-manifest "project A"
    Initializing `project A` project
Error: destination `${current_dir_path}/project A/Project.toml` already exists
```

```sh
$ todo init --with-manifest "project B"
    Initializing `project B` project
Error: destination `${current_dir_path}/project B/project B.manifest.md` already exists
```

```sh
$ todo init "project B"
    Initializing `project B` project
Error: destination `${current_dir_path}/project B/project B.manifest.md` already exists
```

# Init project inside project dir

## Prepare directories

```sh
$ mkdir "project A"
$ mkdir "project B"
```

## Init project with config

```sh
$ cd "project A"
$ todo init
    Initializing `project A` project
```

```sh
$ ls "project A"
Project.toml
```

```sh
$ cat "project A/Project.toml"
id = "project A"
name = "project A"
```

```sh
$ cd "project A"
$ todo init
    Initializing `project A` project
Error: destination `${current_dir_path}/Project.toml` already exists
```

## Init project with manifest

```sh
$ cd "project B"
$ todo init --with-manifest
    Initializing `project B` project
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
````

```sh
$ cd "project A"
$ todo init --with-manifest
    Initializing `project A` project
Error: destination `${current_dir_path}/Project.toml` already exists
```

```sh
$ cd "project B"
$ todo init --with-manifest
    Initializing `project B` project
Error: destination `${current_dir_path}/project B.manifest.md` already exists
```

```sh
$ cd "project B"
$ todo init
    Initializing `project B` project
Error: destination `${current_dir_path}/project B.manifest.md` already exists
```

# Init subproject outside project dir

## Prepare parent projects

```sh
$ todo new "project A"
    Creating `project A` project
```

```sh
$ todo new --with-manifest "project B"
    Creating `project B` project
```

```sh
$ mkdir "project A/project B"
$ mkdir "project B/project B"
```

## Init subproject default

```sh
$ todo init "project A/project B"
    Initializing `project B` project under `${current_dir_path}/project A`
```

```sh
$ ls "project A"
Project.toml project B
```

```sh
$ ls "project A/project B"
Project.toml
```

```sh
$ cat "project A/Project.toml"
id = "project A"
name = "project A"
```

```sh
$ cat "project A/project B/Project.toml"
id = "project B"
name = "project B"
```

```sh
$ todo init "project A/project B"
    Initializing `project B` project under `${current_dir_path}/project A`
Error: destination `${current_dir_path}/project A/project B/Project.toml` already exists
```

## Init subproject with manifest

```sh
$ todo init --with-manifest "project B/project B"
    Initializing `project B` project under `${current_dir_path}/project B`
```

```sh
$ ls "project B"
project B project B.manifest.md
```

```sh
$ ls "project B/project B"
project B.manifest.md
```

````sh
$ cat "project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
````

````sh
$ cat "project B/project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
````

```sh
$ todo init --with-manifest "project B/project B"
    Initializing `project B` project under `${current_dir_path}/project B`
Error: destination `${current_dir_path}/project B/project B/project B.manifest.md` already exists
```

```sh
$ todo init "project B/project B"
    Initializing `project B` project under `${current_dir_path}/project B`
Error: destination `${current_dir_path}/project B/project B/project B.manifest.md` already exists
```

# Init subproject inside project dir

## Prepare parent projects

```sh
$ todo new "project A"
    Creating `project A` project
```

```sh
$ todo new --with-manifest "project B"
    Creating `project B` project
```

```sh
$ mkdir "project A/project B"
$ mkdir "project B/project B"
```

## Init subproject default

```sh
$ cd "project A/project B"
$ todo init
    Initializing `project B` project
```

```sh
$ ls "project A"
Project.toml project B
```

```sh
$ ls "project A/project B"
Project.toml
```

```sh
$ cat "project A/Project.toml"
id = "project A"
name = "project A"
```

```sh
$ cat "project A/project B/Project.toml"
id = "project B"
name = "project B"
```

```sh
$ cd "project A/project B"
$ todo init
    Initializing `project B` project
Error: destination `${current_dir_path}/Project.toml` already exists
```

## Init subproject with manifest

```sh
$ cd "project B/project B"
$ todo init --with-manifest
    Initializing `project B` project
```

```sh
$ ls "project B"
project B project B.manifest.md
```

```sh
$ ls "project B/project B"
project B.manifest.md
```

````sh
$ cat "project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
````

````sh
$ cat "project B/project B/project B.manifest.md"
# project B

```toml project
id = "project B"
name = "project B"
```
````

```sh
$ cd "project B/project B"
$ todo init --with-manifest
    Initializing `project B` project
Error: destination `${current_dir_path}/project B.manifest.md` already exists
```

```sh
$ cd "project B/project B"
$ todo init
    Initializing `project B` project
Error: destination `${current_dir_path}/project B.manifest.md` already exists
```