use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use const_format::concatcp;
use fs_err::File;
use function_name::named;
use temp_testdir::TempDir;
use tests::init_logger;
use todo_lib::issue::{Issue, IssueSet};
use todo_lib::plan::Plan;
use todo_tracker_fs::Placement;
use todo_tracker_fs::generator::IntIdGenerator;
use todo_tracker_fs::plan::LoadProjectPlan;

#[rustfmt::skip]
static TASK_LIST_TEXT: &str = concatcp!(r"
- task A
  - task AA

- task B [Mile 2](#mile-2)

---

- task C

# Mile 1

- task D
  One line description
- task E
  - task EA
  - task EB
- task F

---

- task G
  Multi line
  description

  ```code
  block
  ```

- task H
  - task HA
    - task HAA
      Deep level description

    - task HAB
  - task HB
- task I

# Mile 2

- [task K](", BASIC_TASK_FILE_PATH, ") Trailing description first line.
  Second line
");

const BASIC_TASK_FILE_PATH: &str = "task-basic.md";

const BASIC_TASK_FILE_CONTENT: &str = r"
Some content.

## Some section

Some section content.
";

#[track_caller]
fn assert_task_list_plan(plan: &Plan<u64>) {
    let issue_ids: Vec<u64> = plan.issues().keys().copied().collect();
    assert_eq!(issue_ids, vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
    ]);

    let set_names: Vec<&str> = plan.sets().keys().map(|s| s.as_str()).collect();
    assert_eq!(set_names, vec!["Mile 1", "Mile 2"]);

    assert_eq!(*plan.get_issue(&1).unwrap(), Issue::new(1, "task A").with_subissue(2));
    assert_eq!(*plan.get_issue(&2).unwrap(), Issue::new(2, "task AA").with_parent_id(1));
    assert_eq!(*plan.get_issue(&3).unwrap(), Issue::new(3, "task B [Mile 2](#mile-2)"));
    assert_eq!(*plan.get_issue(&4).unwrap(), Issue::new(4, "task C"));
    assert_eq!(
        *plan.get_issue(&5).unwrap(),
        Issue::new(5, "task D").with_content("One line description")
    );
    assert_eq!(
        *plan.get_issue(&6).unwrap(),
        Issue::new(6, "task E").with_subissue(7).with_subissue(8)
    );
    assert_eq!(*plan.get_issue(&7).unwrap(), Issue::new(7, "task EA").with_parent_id(6));
    assert_eq!(*plan.get_issue(&8).unwrap(), Issue::new(8, "task EB").with_parent_id(6));
    assert_eq!(*plan.get_issue(&9).unwrap(), Issue::new(9, "task F"));
    assert_eq!(
        *plan.get_issue(&10).unwrap(),
        Issue::new(10, "task G").with_content(
            r"Multi line
description

```code
block
```"
        )
    );
    assert_eq!(
        *plan.get_issue(&11).unwrap(),
        Issue::new(11, "task H").with_subissue(12).with_subissue(15)
    );
    assert_eq!(
        *plan.get_issue(&12).unwrap(),
        Issue::new(12, "task HA")
            .with_parent_id(11)
            .with_subissue(13)
            .with_subissue(14)
    );
    assert_eq!(
        *plan.get_issue(&13).unwrap(),
        Issue::new(13, "task HAA")
            .with_parent_id(12)
            .with_content("Deep level description")
    );
    assert_eq!(
        *plan.get_issue(&14).unwrap(),
        Issue::new(14, "task HAB").with_parent_id(12)
    );
    assert_eq!(
        *plan.get_issue(&15).unwrap(),
        Issue::new(15, "task HB").with_parent_id(11)
    );
    assert_eq!(*plan.get_issue(&16).unwrap(), Issue::new(16, "task I"));

    assert_eq!(
        *plan.get_set("Mile 1").unwrap(),
        IssueSet::new("Mile 1")
            .with_issue(5)
            .with_issue(6)
            .with_issue(7)
            .with_issue(8)
            .with_issue(9)
            .with_issue(10)
            .with_issue(11)
            .with_issue(12)
            .with_issue(13)
            .with_issue(14)
            .with_issue(15)
            .with_issue(16)
    );
    assert_eq!(*plan.get_set("Mile 2").unwrap(), IssueSet::new("Mile 2").with_issue(17));
    assert_eq!(
        *plan.get_issue(&17).unwrap(),
        Issue::new(17, "task K")
            .with_content_file_and_note(BASIC_TASK_FILE_PATH, "Trailing description first line.\nSecond line")
    );
}

fn create_temp_project_root_dir(temp_dir: impl AsRef<Path>, project: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let root_dir = temp_dir.as_ref().join(project);
    if !root_dir.exists() {
        fs::create_dir(&root_dir)?;
    }
    Ok(root_dir)
}

fn create_task_files(project_root: impl AsRef<Path>) -> anyhow::Result<()> {
    let basic_task_content_file_path = project_root.as_ref().join(BASIC_TASK_FILE_PATH);
    File::create(basic_task_content_file_path.clone())?.write_all(BASIC_TASK_FILE_CONTENT.as_bytes())?;

    Ok(())
}

#[test]
#[named]
fn tasks_from_todo_file() -> anyhow::Result<()> {
    init_logger();

    let temp_dir = TempDir::default();
    let project_root = create_temp_project_root_dir(&temp_dir, function_name!())?;
    let todo_file_path = project_root.join("TODO.md");

    File::create(todo_file_path.clone())?.write_all(TASK_LIST_TEXT.as_bytes())?;

    create_task_files(&project_root)?;

    let id_generator = IntIdGenerator::new(1);
    let plan = Plan::load(&Placement::WholeFile(todo_file_path), &id_generator)?.unwrap();

    assert_task_list_plan(&plan);

    Ok(())
}

#[test]
#[named]
fn tasks_from_manifest_file() -> anyhow::Result<()> {
    init_logger();

    let temp_dir = TempDir::default();
    let project_root = create_temp_project_root_dir(&temp_dir, function_name!())?;
    let manifest_file_path = project_root.join(format!("{}.manifest.md", function_name!()));

    File::create(manifest_file_path.clone())?.write_all(
        format!(
            r"
# Project header

Some description.

List 1:

- item 1
- item 2
  - item 3

```md
# Regular internal markdown

List 2:

- item 4
- item 5
```

```md todo
{}
```

List 3:

- item 6
- item 7",
            TASK_LIST_TEXT
        )
        .as_bytes(),
    )?;

    create_task_files(&project_root)?;

    let id_generator = IntIdGenerator::new(1);
    let plan = Plan::load(&Placement::CodeBlockInFile(manifest_file_path), &id_generator)?.unwrap();

    assert_task_list_plan(&plan);

    Ok(())
}

#[test]
#[named]
fn subsets() -> anyhow::Result<()> {
    init_logger();

    let temp_dir = TempDir::default();
    let project_root = create_temp_project_root_dir(&temp_dir, function_name!())?;
    let todo_file_path = project_root.join("TODO.md");

    let text = r"
- top task A

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
";

    File::create(todo_file_path.clone())?.write_all(text.as_bytes())?;

    let id_generator = IntIdGenerator::new(1);
    let plan = Plan::load(&Placement::WholeFile(todo_file_path), &id_generator)?.unwrap();

    let set_names: Vec<_> = plan.sets().keys().map(|set_name| set_name.as_str()).collect();
    assert_eq!(set_names, vec![
        "Set 1",
        "Set 1/Sub set 1.1",
        "Set 1/Sub set 1.2",
        "Set 2"
    ]);

    let issue = Issue::new(1, "top task A");
    assert_eq!(*plan.get_issue(&issue.id).unwrap(), issue);
    assert!(plan.find_sets(&issue.id).next().is_none());

    let set_1 = IssueSet::new("Set 1").with_issue(2).with_issue(3);
    assert_eq!(*plan.get_set(&set_1.name).unwrap(), set_1);

    let issue = Issue::new(2, "task 1A");
    assert_eq!(*plan.get_issue(&issue.id).unwrap(), issue);
    assert_eq!(plan.find_sets(&issue.id).next().unwrap().name, set_1.name);

    let issue = Issue::new(3, "task 1B");
    assert_eq!(*plan.get_issue(&issue.id).unwrap(), issue);
    assert_eq!(plan.find_sets(&issue.id).next().unwrap().name, set_1.name);

    let set_1_1 = IssueSet::new("Set 1/Sub set 1.1").with_issue(4);
    assert_eq!(*plan.get_set(&set_1_1.name).unwrap(), set_1_1);

    let issue = Issue::new(4, "task 1.1A");
    assert_eq!(*plan.get_issue(&issue.id).unwrap(), issue);
    assert_eq!(plan.find_sets(&issue.id).next().unwrap().name, set_1_1.name);

    let set_1_2 = IssueSet::new("Set 1/Sub set 1.2").with_issue(5).with_issue(6);
    assert_eq!(*plan.get_set(&set_1_2.name).unwrap(), set_1_2);

    let issue = Issue::new(5, "task 1.2A");
    assert_eq!(*plan.get_issue(&issue.id).unwrap(), issue);
    assert_eq!(plan.find_sets(&issue.id).next().unwrap().name, set_1_2.name);

    let issue = Issue::new(6, "task 1.2B");
    assert_eq!(*plan.get_issue(&issue.id).unwrap(), issue);
    assert_eq!(plan.find_sets(&issue.id).next().unwrap().name, set_1_2.name);

    let set_2 = IssueSet::new("Set 2").with_issue(7);
    assert_eq!(*plan.get_set(&set_2.name).unwrap(), set_2);

    let issue = Issue::new(7, "task 2A");
    assert_eq!(*plan.get_issue(&issue.id).unwrap(), issue);
    assert_eq!(
        plan.find_sets(&issue.id).next().map(|set| set.name.as_str()),
        Some("Set 2")
    );

    Ok(())
}
