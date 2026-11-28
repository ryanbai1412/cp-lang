//! The files diagnostics and every layer name.
use std::sync::Arc;

/// A file of a snapshot: its path, as the tool was given it. Stable for
/// the life of a session.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FileId(Arc<str>);

impl FileId {
    pub fn new(path: &str) -> FileId {
        FileId(Arc::from(path))
    }

    pub fn path(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FileId {
    fn from(path: &str) -> FileId {
        FileId::new(path)
    }
}

impl AsRef<str> for FileId {
    fn as_ref(&self) -> &str {
        self.path()
    }
}
