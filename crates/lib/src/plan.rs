use indexmap::{IndexMap, IndexSet};

use crate::id::HashedId;
use crate::issue::{Issue, Milestone};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Step<ID> {
    Issue(ID),
    Milestone(ID),
}

#[derive(Default)]
pub struct Plan<ID> {
    issues: IndexMap<ID, Issue<ID>>,
    milestones: IndexMap<ID, Milestone<ID>>,
    steps: IndexSet<Step<ID>>,
}

impl<ID> Plan<ID> {
    pub fn new() -> Self {
        Self {
            issues: IndexMap::new(),
            milestones: IndexMap::new(),
            steps: IndexSet::new(),
        }
    }
}

impl<ID: HashedId + PartialEq + Clone> Plan<ID> {
    pub fn is_empty(&self) -> bool {
        let Self {
            issues,
            milestones,
            steps,
        } = self;

        issues.is_empty() && milestones.is_empty() && steps.is_empty()
    }

    pub fn get_issue(&self, id: &ID) -> Option<&Issue<ID>> {
        self.issues.get(id)
    }

    pub fn get_milestone(&self, id: &ID) -> Option<&Milestone<ID>> {
        self.milestones.get(id)
    }

    pub fn steps(&self) -> &IndexSet<Step<ID>> {
        &self.steps
    }

    pub fn add_issue(&mut self, issue: Issue<ID>) {
        self.steps.insert(Step::Issue(issue.id.clone()));
        self.issues.insert(issue.id.clone(), issue);
    }

    pub fn add_milestone(&mut self, milestone: Milestone<ID>) {
        self.steps.insert(Step::Milestone(milestone.id.clone()));
        self.milestones.insert(milestone.id.clone(), milestone);
    }

    pub fn add_issues(&mut self, issues: impl IntoIterator<Item = Issue<ID>>) {
        for issue in issues {
            self.add_issue(issue);
        }
    }

    pub fn find_issue(&self, name: impl AsRef<str>) -> Option<&Issue<ID>> {
        self.issues
            .iter()
            .find_map(|(_, issue)| if issue.name == name.as_ref() { Some(issue) } else { None })
    }

    pub fn merge(mut self, other: Self) -> Self {
        let Self {
            issues,
            milestones,
            steps,
        } = &mut self;

        issues.extend(other.issues);
        milestones.extend(other.milestones);
        steps.extend(other.steps);

        self
    }
}
