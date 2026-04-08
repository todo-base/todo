use std::fmt::{Display, Write};

use indexmap::{IndexMap, IndexSet};
use todo_app::config::{DisplayProjectConfig, TitleConsist};
use todo_lib::id::HashedId;
use todo_lib::plan::Step;
use todo_lib::project::Project;
use todo_tracker_fs::FsTracker;

#[macro_export]
macro_rules! out {
    ($($arg:tt)*) => {{
        print!($($arg)*)
    }};
}

#[macro_export]
macro_rules! outln {
    ($($arg:tt)*) => {{
        println!($($arg)*)
    }};
}

pub fn out_errors(errors: impl IntoIterator<Item = impl Display>) {
    for err in errors {
        outln!("Error: {err}");
    }
}

pub trait DisplayList<ID> {
    fn format_project_title_key(&self, project: &Project<ID>, title: TitleConsist, with_parents: bool) -> String;
    fn display_project_title(
        &self,
        project: &Project<ID>,
        title: TitleConsist,
        title_key: Option<impl AsRef<str>>,
        config: &DisplayProjectConfig,
    );
    fn display_steps_list(
        &self,
        prefix: impl AsRef<str>,
        indent: impl AsRef<str>,
        project: &Project<ID>,
        config: &DisplayProjectConfig,
    );
    fn display_projects_list(&self, config: &DisplayProjectConfig);
    fn display_projects_tree(&self, config: &DisplayProjectConfig);
}

impl<ID: HashedId + Clone + Display> DisplayList<ID> for FsTracker<ID> {
    fn format_project_title_key(&self, project: &Project<ID>, consist: TitleConsist, with_parents: bool) -> String {
        let mut output = String::new();
        format_project_title_key_inner(self, project, consist, with_parents, &mut output);
        output
    }

    fn display_project_title(
        &self,
        project: &Project<ID>,
        consist: TitleConsist,
        title_key: Option<impl AsRef<str>>,
        config: &DisplayProjectConfig,
    ) {
        if let Some(before) = match consist {
            TitleConsist::Id => &config.title.id_before,
            TitleConsist::Name => &config.title.name_before,
            TitleConsist::IdAndName => {
                if let Some(id_before) = &config.title.id_and_name_before {
                    out!("{id_before}");
                }
                &config.title.id_before
            },
        } {
            out!("{before}");
        }

        if let Some(title_key) = title_key {
            out!("{}", title_key.as_ref());
        } else {
            out!("{}", self.format_project_title_key(project, consist, true));
        }

        if let TitleConsist::IdAndName = consist {
            if let Some(id_after) = &config.title.id_after {
                out!("{id_after}");
            }

            if project.id().to_string() != project.name() {
                if let Some(separator) = &config.title.id_and_name_separator {
                    out!("{separator}");
                }
                out!("{}", project.name());
            }
        }

        if let Some(after) = match consist {
            TitleConsist::Id => &config.title.id_after,
            TitleConsist::Name => &config.title.name_after,
            TitleConsist::IdAndName => &config.title.id_and_name_after,
        } {
            out!("{after}");
        }

        if config.title.show_steps_count {
            let steps_count = self
                .project_plan(project.id())
                .map(|plan| {
                    plan.steps().iter().fold(0_usize, |count, step| {
                        if !config.show_substeps
                            && let Step::Issue(id) = step
                            && let Some(issue) = plan.get_issue(id)
                            && issue.parent_id.is_some()
                        {
                            return count;
                        }
                        count + 1
                    })
                })
                .unwrap_or(0);
            out!(": {steps_count}")
        }
    }

    fn display_steps_list(
        &self,
        prefix: impl AsRef<str>,
        indent: impl AsRef<str>,
        project: &Project<ID>,
        config: &DisplayProjectConfig,
    ) {
        let prefix = prefix.as_ref();
        let indent = indent.as_ref();
        let max_count = config.max_steps.unwrap_or(usize::MAX);

        if let Some(plan) = self.project_plan(project.id()) {
            let mut parent_ids = Vec::new();

            let mut step_count = 0;
            let mut display_line = |level, text: &str| {
                if step_count < max_count {
                    outln!("{prefix}{indent}{:1$}{text}", "", level * 2);
                }
                step_count += 1;
                step_count < max_count
            };

            let mut is_next_displayed = max_count > 0;
            for step in plan.steps() {
                match step {
                    Step::Issue(id) => {
                        if let Some(issue) = plan.get_issue(id) {
                            if let Some(parent_id) = issue.parent_id {
                                if config.show_substeps {
                                    loop {
                                        if Some(parent_id) == parent_ids.last().copied() || parent_ids.pop().is_none() {
                                            break;
                                        }
                                    }

                                    is_next_displayed =
                                        display_line(parent_ids.len() + 1, &format!("- {}", issue.name));
                                    parent_ids.push(issue.id);
                                }
                            } else {
                                is_next_displayed = display_line(0, &format!("- {}", issue.name));
                                parent_ids.clear();
                            }
                        }
                    },
                    Step::Milestone(id) => {
                        if let Some(milestone) = plan.get_milestone(id) {
                            if !config.compact && is_next_displayed {
                                outln!("{prefix}");
                            }
                            is_next_displayed = display_line(0, &format!("# {}", milestone.name));
                            if !config.compact && is_next_displayed {
                                outln!("{prefix}");
                            }
                        }
                    },
                }
            }
            if step_count > max_count {
                outln!("{prefix}{indent}..{}", step_count - max_count);
            }
        }
    }

    fn display_projects_list(&self, config: &DisplayProjectConfig) {
        if !config.compact {
            let count = self.projects().len();

            if count == 1 {
                outln!("List steps of {count} project");
            } else {
                outln!("List steps of {count} projects");
            }
        }

        let mut projects = Vec::new();
        for project in self.projects().values() {
            let consist = if matches!(config.title.consist, TitleConsist::IdAndName) && project.name().is_empty() {
                TitleConsist::Id
            } else {
                config.title.consist
            };
            projects.push((self.format_project_title_key(project, consist, true), consist, project));
        }

        projects.sort_by(|(title_key1, ..), (title_key2, ..)| title_key1.cmp(title_key2));

        let mut is_first_project = true;
        for (title_key, consist, project) in projects {
            if !config.compact || (!is_first_project && config.separate_projects) {
                outln!();
            }

            self.display_project_title(project, consist, Some(title_key), config);
            outln!();
            self.display_steps_list("", "", project, config);

            is_first_project = false;
        }
    }

    fn display_projects_tree(&self, config: &DisplayProjectConfig) {
        if !config.compact {
            let count = self.projects().len();

            if count == 1 {
                outln!("Tree of {count} project");
            } else {
                outln!("Trees of {count} projects");
            }
        }

        let project_ids = self.projects().values().filter_map(|project| {
            if project.parent_id().is_none() {
                Some(project.id())
            } else {
                None
            }
        });

        display_nested_projecs(self, project_ids, &self.subprojects(), config, "");
    }
}

fn format_project_title_key_inner<ID>(
    tracker: &FsTracker<ID>,
    project: &Project<ID>,
    consist: TitleConsist,
    with_parents: bool,
    output: &mut String,
) where
    ID: HashedId + Clone + Display,
{
    if with_parents
        && let Some(parent_id) = project.parent_id()
        && let Some(parent) = tracker.projects().get(parent_id)
    {
        format_project_title_key_inner(tracker, parent, consist, true, output);
        write!(output, "/").expect("Failed to write to string");
    }

    match consist {
        TitleConsist::Id | TitleConsist::IdAndName => write!(output, "{id}", id = project.id()),
        TitleConsist::Name => write!(output, "{name}", name = project.name()),
    }
    .expect("Failed to write to string");
}

fn display_nested_projecs<'a, ID>(
    tracker: &FsTracker<ID>,
    project_ids: impl IntoIterator<Item = &'a ID>,
    subprojects: &IndexMap<ID, IndexSet<ID>>,
    config: &DisplayProjectConfig,
    prefix: impl AsRef<str>,
) where
    ID: HashedId + Clone + Display + 'a,
{
    let prefix = prefix.as_ref();
    let mut prepared_projects = Vec::new();

    for project_id in project_ids {
        if let Some(project) = tracker.projects().get(project_id) {
            let consist = if matches!(config.title.consist, TitleConsist::IdAndName) && project.name().is_empty() {
                TitleConsist::Id
            } else {
                config.title.consist
            };
            prepared_projects.push((
                tracker.format_project_title_key(project, consist, false),
                consist,
                project,
            ));
        }
    }

    prepared_projects.sort_by(|(title_key1, ..), (title_key2, ..)| title_key1.cmp(title_key2));

    let mut is_first_project = true;
    let count = prepared_projects.len();

    for (idx, (title_key, consist, project)) in prepared_projects.into_iter().enumerate() {
        let is_last_project = idx + 1 == count;

        if !config.compact || (!is_first_project && config.separate_projects) {
            outln!("{prefix}");
        }

        if prefix.len() > 2 {
            let connector = if is_last_project { "└─" } else { "├─" };
            out!("{}{connector} ", prefix.trim_end_matches("│"));
        } else {
            out!("{prefix}");
        }
        tracker.display_project_title(project, consist, Some(title_key), config);
        outln!();

        let children = subprojects.get(project.id());
        let child_prefix = if children.is_some() {
            "  │"
        } else if !prefix.is_empty() {
            "  "
        } else {
            ""
        };
        let steps_prefix = if is_last_project {
            let trim_prefix = prefix.trim_end_matches("│");
            let space = if trim_prefix.len() < prefix.len() { " " } else { "" };
            format!("{trim_prefix}{space}{child_prefix}")
        } else {
            format!("{prefix}{child_prefix}")
        };
        tracker.display_steps_list(steps_prefix, "  ", project, config);

        if let Some(children) = children {
            display_nested_projecs(tracker, children, subprojects, config, format!("{prefix}  │"));
        }

        is_first_project = false;
    }
}
