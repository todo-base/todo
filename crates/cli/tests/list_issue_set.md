# List with issue sets

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

## List pretty with nested issue sets

```sh
$ todo list --pretty .
List items of 1 project

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

## List compact with nested issue sets

```sh
$ todo list --compact .
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
$ todo new "project B"
    Creating `project B` project
```

```sh
$ echo "# Set Alpha

- alpha 1
- alpha 2

# Set Beta

- beta 1
" > "project B/TODO.md"
```

## List pretty with sibling issue sets

```sh
$ todo list --pretty --project "project B"
List items of 1 project

[project B]: 3

# Set Alpha

- alpha 1
- alpha 2

# Set Beta

- beta 1
```

## List compact with sibling issue sets

```sh
$ todo list --compact --project "project B"
[project B]: 3
# Set Alpha
- alpha 1
- alpha 2
# Set Beta
- beta 1
```
