use indexmap::IndexMap;

use crate::id::HashedId;
use crate::issue::{Issue, IssueSet};

#[derive(Default)]
pub struct Plan<ID> {
    issues: IndexMap<ID, Issue<ID>>,
    sets: IndexMap<String, IssueSet<ID>>,
}

impl<ID> Plan<ID> {
    pub fn new() -> Self {
        Self {
            issues: IndexMap::new(),
            sets: IndexMap::new(),
        }
    }
}

impl<ID: HashedId + PartialEq + Clone> Plan<ID> {
    pub fn is_empty(&self) -> bool {
        let Self { issues, sets } = self;

        issues.is_empty() && sets.is_empty()
    }

    pub fn get_issue(&self, id: &ID) -> Option<&Issue<ID>> {
        self.issues.get(id)
    }

    pub fn get_set(&self, name: impl AsRef<str>) -> Option<&IssueSet<ID>> {
        self.sets.get(name.as_ref())
    }

    pub fn issues(&self) -> &IndexMap<ID, Issue<ID>> {
        &self.issues
    }

    pub fn sets(&self) -> &IndexMap<String, IssueSet<ID>> {
        &self.sets
    }

    pub fn add_issue(&mut self, issue: Issue<ID>) {
        self.issues.insert(issue.id.clone(), issue);
    }

    pub fn add_set(&mut self, set: IssueSet<ID>) {
        self.sets.insert(set.name.clone(), set);
    }

    pub fn add_issue_to_set(&mut self, set_name: impl AsRef<str>, issue_id: ID) {
        if let Some(set) = self.sets.get_mut(set_name.as_ref()) {
            set.add(issue_id);
        }
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

    pub fn find_sets(&self, issue_id: &ID) -> impl Iterator<Item = &IssueSet<ID>> {
        self.sets.values().filter(|set| set.issues.contains(issue_id))
    }

    pub fn merge(mut self, other: Self) -> Self {
        let Self { issues, sets } = &mut self;

        issues.extend(other.issues);
        sets.extend(other.sets);

        self
    }
}
