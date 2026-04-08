use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use fs_err::File;
use function_name::named;
use temp_testdir::TempDir;
use tests::init_logger;
use todo_lib::issue::{Issue, Milestone};
use todo_lib::plan::Plan;
use todo_lib::plan::Step::*;
use todo_tracker_fs::generator::IntIdGenerator;
use todo_tracker_fs::plan::LoadProjectPlan;
use todo_tracker_fs::Placement;

static TASK_LIST_TEXT: &'static str = r"
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
";

#[track_caller]
fn assert_task_list_plan(plan: &Plan<u64>) {
    assert_eq!(plan.steps().into_iter().copied().collect::<Vec<_>>(), vec![
        Issue(1),
        Issue(2),
        Issue(3),
        Issue(4),
        Milestone(5),
        Issue(6),
        Issue(7),
        Issue(8),
        Issue(9),
        Issue(10),
        Issue(11),
        Issue(12),
        Issue(13),
        Issue(14),
        Issue(15),
        Issue(16),
        Issue(17),
        Milestone(18),
    ]);

    assert_eq!(*plan.get_issue(&1).unwrap(), Issue::new(1, "task A").with_subissue(2));
    assert_eq!(*plan.get_issue(&2).unwrap(), Issue::new(2, "task AA").with_parent_id(1));
    assert_eq!(*plan.get_issue(&3).unwrap(), Issue::new(3, "task B [Mile 2](#mile-2)"));
    assert_eq!(*plan.get_issue(&4).unwrap(), Issue::new(4, "task C"));
    assert_eq!(
        *plan.get_issue(&6).unwrap(),
        Issue::new(6, "task D").with_content("One line description")
    );
    assert_eq!(
        *plan.get_issue(&7).unwrap(),
        Issue::new(7, "task E").with_subissue(8).with_subissue(9)
    );
    assert_eq!(*plan.get_issue(&8).unwrap(), Issue::new(8, "task EA").with_parent_id(7));
    assert_eq!(*plan.get_issue(&9).unwrap(), Issue::new(9, "task EB").with_parent_id(7));
    assert_eq!(*plan.get_issue(&10).unwrap(), Issue::new(10, "task F"));
    assert_eq!(
        *plan.get_issue(&11).unwrap(),
        Issue::new(11, "task G").with_content(
            r"Multi line
description

```code
block
```"
        )
    );
    assert_eq!(
        *plan.get_issue(&12).unwrap(),
        Issue::new(12, "task H").with_subissue(13).with_subissue(16)
    );
    assert_eq!(
        *plan.get_issue(&13).unwrap(),
        Issue::new(13, "task HA")
            .with_parent_id(12)
            .with_subissue(14)
            .with_subissue(15)
    );
    assert_eq!(
        *plan.get_issue(&14).unwrap(),
        Issue::new(14, "task HAA")
            .with_parent_id(13)
            .with_content("Deep level description")
    );
    assert_eq!(
        *plan.get_issue(&15).unwrap(),
        Issue::new(15, "task HAB").with_parent_id(13)
    );
    assert_eq!(
        *plan.get_issue(&16).unwrap(),
        Issue::new(16, "task HB").with_parent_id(12)
    );
    assert_eq!(*plan.get_issue(&17).unwrap(), Issue::new(17, "task I"));

    assert_eq!(
        *plan.get_milestone(&5).unwrap(),
        Milestone::new(5, "Mile 1").with_needed_issue(4)
    );
    assert_eq!(
        *plan.get_milestone(&18).unwrap(),
        Milestone::new(18, "Mile 2")
            .with_needed_issue(11)
            .with_needed_issue(12)
            .with_needed_issue(13)
            .with_needed_issue(14)
            .with_needed_issue(15)
            .with_needed_issue(16)
            .with_needed_issue(17)
    );
}

fn create_temp_project_root_dir(temp_dir: impl AsRef<Path>, project: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let root_dir = temp_dir.as_ref().join(project);
    if !root_dir.exists() {
        fs::create_dir(&root_dir)?;
    }
    Ok(root_dir)
}

#[test]
#[named]
fn tasks_from_todo_file() -> anyhow::Result<()> {
    init_logger();

    let temp_dir = TempDir::default();
    let project_root = create_temp_project_root_dir(&temp_dir, function_name!())?;
    let todo_file_path = project_root.join("TODO.md");

    File::create(todo_file_path.clone())?.write_all(TASK_LIST_TEXT.as_bytes())?;

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

    let id_generator = IntIdGenerator::new(1);
    let plan = Plan::load(&Placement::CodeBlockInFile(manifest_file_path), &id_generator)?.unwrap();

    assert_task_list_plan(&plan);

    Ok(())
}
