use indexmap::IndexSet;

#[derive(Debug, Clone)]
pub struct Project<ID> {
    id: ID,
    parent_id: Option<ID>,
    name: String,
    subprojects: IndexSet<ID>,
}

impl<ID> Project<ID> {
    pub fn new(id: ID, name: impl Into<String>) -> Self {
        Self {
            id,
            parent_id: None,
            name: name.into(),
            subprojects: Default::default(),
        }
    }

    pub fn with_parent(mut self, parent_id: ID) -> Self {
        self.set_parent(parent_id);
        self
    }

    pub fn with_subprojects(mut self, subprojects: impl Into<IndexSet<ID>>) -> Self {
        self.subprojects = subprojects.into();
        self
    }

    pub fn set_parent(&mut self, parent_id: ID) -> Option<ID> {
        self.parent_id.replace(parent_id)
    }

    pub fn id(&self) -> &ID {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parent_id(&self) -> Option<&ID> {
        self.parent_id.as_ref()
    }
}
