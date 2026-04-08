use std::hash::Hash;

use indexmap::IndexSet;

use crate::id::HashedId;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LinkType {
    FinishToStart,
    StartToStart,
    FinishToFinish,
    StartToFinish,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DependencyType {
    Before,
    After,
    Blocks,
    IsBlockedBy,
    Contains,
    IsContainedIn,
    RelatesTo,
    AssociatedWith,
    Other(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Relation {
    pub link: LinkType,
    pub dependency: DependencyType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IssueRelation<ID> {
    pub to_id: ID,
    pub relation: Relation,
}

#[derive(Debug, Clone)]
pub struct Issue<ID> {
    pub id: ID,
    pub parent_id: Option<ID>,
    pub name: String,
    pub content: String,
    pub subissues: IndexSet<ID>,
    pub relations: Vec<IssueRelation<ID>>,
}

impl<ID: HashedId + PartialEq> PartialEq for Issue<ID> {
    fn eq(&self, other: &Self) -> bool {
        let Issue {
            id,
            parent_id,
            name,
            content,
            subissues,
            relations,
        } = self;

        *id == other.id
            && *parent_id == other.parent_id
            && *name == other.name
            && *content == other.content
            && *subissues == other.subissues
            && *relations == other.relations
    }
}

impl<ID: HashedId> Eq for Issue<ID> {}

impl<ID> Issue<ID> {
    pub fn new(id: ID, name: impl Into<String>) -> Self {
        Self {
            id,
            parent_id: None,
            name: name.into(),
            content: Default::default(),
            subissues: Default::default(),
            relations: Default::default(),
        }
    }

    pub fn with_parent_id(mut self, parent_id: ID) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }
}

impl<ID: HashedId> Issue<ID> {
    pub fn with_subissue(mut self, subissue_id: ID) -> Self {
        self.subissues.insert(subissue_id);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Milestone<ID> {
    pub id: ID,
    pub name: String,
    pub needed_issues: IndexSet<ID>,
}

impl<ID: HashedId + PartialEq> PartialEq for Milestone<ID> {
    fn eq(&self, other: &Self) -> bool {
        let Milestone {
            id,
            name,
            needed_issues,
        } = self;

        *id == other.id && *name == other.name && *needed_issues == other.needed_issues
    }
}

impl<ID: HashedId> Eq for Milestone<ID> {}

impl<ID> Milestone<ID> {
    pub fn new(id: ID, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            needed_issues: Default::default(),
        }
    }
}

impl<ID: HashedId> Milestone<ID> {
    pub fn with_needed_issue(mut self, issue_id: ID) -> Self {
        self.needed_issues.insert(issue_id);
        self
    }
}
