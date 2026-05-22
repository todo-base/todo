use std::hash::Hash;
use std::path::PathBuf;

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

#[derive(Debug, Clone, Default, PartialEq)]
pub enum IssueContent {
    #[default]
    Empty,
    Inline(String),
    Linked {
        file: PathBuf,
        note: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Issue<ID> {
    pub id: ID,
    pub parent_id: Option<ID>,
    pub name: String,
    pub content: IssueContent,
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
        self.content = IssueContent::Inline(content.into());
        self
    }

    pub fn with_content_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.content = IssueContent::Linked {
            file: path.into(),
            note: None,
        };
        self
    }

    pub fn with_content_file_and_note(mut self, path: impl Into<PathBuf>, note: impl Into<String>) -> Self {
        self.content = IssueContent::Linked {
            file: path.into(),
            note: Some(note.into()),
        };
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
pub struct IssueSet<ID> {
    pub name: String,
    pub issues: IndexSet<ID>,
}

impl<ID: HashedId + PartialEq> PartialEq for IssueSet<ID> {
    fn eq(&self, other: &Self) -> bool {
        let IssueSet { name, issues } = self;

        *name == other.name && *issues == other.issues
    }
}

impl<ID: HashedId> Eq for IssueSet<ID> {}

impl<ID> IssueSet<ID> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            issues: Default::default(),
        }
    }
}

impl<ID: HashedId> IssueSet<ID> {
    pub fn with_issue(mut self, issue_id: ID) -> Self {
        self.add(issue_id);
        self
    }

    pub fn add(&mut self, issue_id: ID) -> &mut Self {
        self.issues.insert(issue_id);
        self
    }
}
