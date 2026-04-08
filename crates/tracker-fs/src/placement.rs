use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Placement<P> {
    WholeFile(P),
    CodeBlockInFile(P),
}

impl<P> Placement<P> {
    pub fn map<Q>(self, f: impl FnOnce(P) -> Q) -> Placement<Q> {
        match self {
            Self::WholeFile(p) => Placement::WholeFile(f(p)),
            Self::CodeBlockInFile(p) => Placement::CodeBlockInFile(f(p)),
        }
    }

    pub fn map_ref<Q>(&self, f: impl FnOnce(&P) -> Q) -> Placement<Q> {
        match self {
            Self::WholeFile(p) => Placement::WholeFile(f(p)),
            Self::CodeBlockInFile(p) => Placement::CodeBlockInFile(f(p)),
        }
    }
}

impl Placement<PathBuf> {
    pub fn set_root(&mut self, root: impl AsRef<Path>) {
        let path = self.as_mut();
        *path = root.as_ref().join(path.as_path())
    }
}

impl<P> AsRef<P> for Placement<P> {
    fn as_ref(&self) -> &P {
        match self {
            Self::WholeFile(p) => p,
            Self::CodeBlockInFile(p) => p,
        }
    }
}

impl<P> AsMut<P> for Placement<P> {
    fn as_mut(&mut self) -> &mut P {
        match self {
            Self::WholeFile(p) => p,
            Self::CodeBlockInFile(p) => p,
        }
    }
}
