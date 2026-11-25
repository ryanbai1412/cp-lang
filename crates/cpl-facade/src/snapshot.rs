//! Contract 0: the world, as a snapshot.

use std::collections::BTreeMap;
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

impl From<&&str> for FileId {
    fn from(path: &&str) -> FileId {
        FileId::new(path)
    }
}

impl From<&String> for FileId {
    fn from(path: &String) -> FileId {
        FileId::new(path)
    }
}

impl From<String> for FileId {
    fn from(path: String) -> FileId {
        FileId::new(&path)
    }
}

impl From<&FileId> for FileId {
    fn from(file: &FileId) -> FileId {
        file.clone()
    }
}

/// A file's exact text.
pub type Text = Arc<str>;

/// Where a module the snapshot's files import comes from when the snapshot
/// does not hold it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum World {
    /// The folders `cp.toml` and `lib.toml` name, on disk. A file is read
    /// once, the first time a query needs it, and kept for the session.
    #[default]
    Disk,
    /// Nothing outside the snapshot: a host with no file system (the
    /// browser, a sandbox). `use kit.m` finds `kit/m.cp` among the files,
    /// through the `cp.toml` among them.
    Closed,
}

/// The only input of every query: the files, and where the rest is found.
/// Equal snapshots give equal answers.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Snapshot {
    files: Arc<BTreeMap<FileId, Text>>,
    world: World,
}

impl Snapshot {
    /// No files yet; imports are found in `world`.
    pub fn new(world: World) -> Snapshot {
        Snapshot {
            files: Arc::default(),
            world,
        }
    }

    /// This snapshot with `file` holding `text`. Everything else is shared.
    pub fn with_file(&self, file: impl Into<FileId>, text: impl Into<Text>) -> Snapshot {
        let mut files = (*self.files).clone();
        files.insert(file.into(), text.into());
        Snapshot {
            files: Arc::new(files),
            world: self.world,
        }
    }

    /// This snapshot without `file`.
    pub fn without_file(&self, file: &FileId) -> Snapshot {
        let mut files = (*self.files).clone();
        files.remove(file);
        Snapshot {
            files: Arc::new(files),
            world: self.world,
        }
    }

    pub fn text(&self, file: &FileId) -> Option<&Text> {
        self.files.get(file)
    }

    pub fn files(&self) -> impl Iterator<Item = (&FileId, &Text)> {
        self.files.iter()
    }

    /// Whether `other` has these very files: a cheap test, never wrong
    /// when it says yes.
    pub(crate) fn same_files(&self, other: &Snapshot) -> bool {
        Arc::ptr_eq(&self.files, &other.files) && self.world == other.world
    }

    pub fn world(&self) -> World {
        self.world
    }
}
