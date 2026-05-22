# Tree with issue sets

## Prepare project with nested issue sets

```sh
$ todo new "project A"
    Creating `project A` project
```

```sh
$ echo "- top task A

# Set 1

- task 1A
- task 1B

## Sub set 1.1

- task 1.1A

## Sub set 1.2

- task 1.2A
- task 1.2B

# Set 2

- task 2A
" > "project A/TODO.md"
```

## Tree pretty with nested issue sets

```sh
$ todo tree --pretty .
Tree of 1 project

[project A]: 7
  - top task A

  # Set 1

  - task 1A
  - task 1B

  # Set 1/Sub set 1.1

  - task 1.1A

  # Set 1/Sub set 1.2

  - task 1.2A
  - task 1.2B

  # Set 2

  - task 2A
```

## Tree compact with nested issue sets

```sh
$ todo tree --compact .
[project A]: 7
  - top task A
  # Set 1
  - task 1A
  - task 1B
  # Set 1/Sub set 1.1
  - task 1.1A
  # Set 1/Sub set 1.2
  - task 1.2A
  - task 1.2B
  # Set 2
  - task 2A
```

## Prepare project with sibling issue sets

```sh
$ todo new "project A/project B"
    Creating `project B` project under `${current_dir_path}/project A`
```

```sh
$ echo "# Set Alpha

- alpha 1
- alpha 2

# Set Beta

- beta 1
" > "project A/project B/TODO.md"
```

## Tree pretty with sibling issue sets

```sh
$ todo tree --pretty "project A"
Trees of 2 projects

[project A]: 7
  │  - top task A
  │
  │  # Set 1
  │
  │  - task 1A
  │  - task 1B
  │
  │  # Set 1/Sub set 1.1
  │
  │  - task 1.1A
  │
  │  # Set 1/Sub set 1.2
  │
  │  - task 1.2A
  │  - task 1.2B
  │
  │  # Set 2
  │
  │  - task 2A
  │
  └─ [project B]: 3

       # Set Alpha

       - alpha 1
       - alpha 2

       # Set Beta

       - beta 1
```

```sh
$ todo tree --pretty "project A/project B"
Tree of 1 project

[project B]: 3

  # Set Alpha

  - alpha 1
  - alpha 2

  # Set Beta

  - beta 1
```

## List compact with sibling issue sets

```sh
$ todo tree --compact "project A"
[project A]: 7
  │  - top task A
  │  # Set 1
  │  - task 1A
  │  - task 1B
  │  # Set 1/Sub set 1.1
  │  - task 1.1A
  │  # Set 1/Sub set 1.2
  │  - task 1.2A
  │  - task 1.2B
  │  # Set 2
  │  - task 2A
  └─ [project B]: 3
       # Set Alpha
       - alpha 1
       - alpha 2
       # Set Beta
       - beta 1
```

```sh
$ todo tree --compact "project A/project B"
[project B]: 3
  # Set Alpha
  - alpha 1
  - alpha 2
  # Set Beta
  - beta 1
```
