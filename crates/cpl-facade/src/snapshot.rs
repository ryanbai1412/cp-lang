//! Contract 0: the world, as a snapshot.

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

/// Files in the order of their ids, each once.
pub(crate) type Files = Vec<(FileId, Text)>;

/// Where `files` holds the file at `path`, or where it would go.
pub(crate) fn at(files: &Files, path: &str) -> Result<usize, usize> {
    files.binary_search_by(|(held, _)| held.path().cmp(path))
}

/// The text `files` holds for `file`.
pub(crate) fn text_of<'a>(files: &'a Files, path: &str) -> Option<&'a Text> {
    at(files, path).ok().map(|held| &files[held].1)
}

/// The only input of every query: the files, and where the rest is found.
/// Equal snapshots give equal answers. Cheap to clone.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Snapshot(Arc<Held>);

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct Held {
    /// In the order of their ids.
    files: Files,
    world: World,
    /// The standard library the files are checked against.
    std: Arc<cpl_check::GivenStd>,
}

impl Snapshot {
    /// No files yet; imports are found in `world`.
    pub fn new(world: World) -> Snapshot {
        Snapshot::of(Held {
            world,
            ..Held::default()
        })
    }

    fn of(held: Held) -> Snapshot {
        Snapshot(Arc::new(held))
    }

    /// A copy of what this holds, to change.
    fn held(&self) -> Held {
        Held::clone(&self.0)
    }

    /// This snapshot with `file` holding `text`.
    #[inline(always)]
    pub fn with_file(&self, file: impl AsRef<str>, text: impl AsRef<str>) -> Snapshot {
        self.holding(file.as_ref(), Some(text.as_ref()))
    }

    /// This snapshot with `file` holding `text`, or without it.
    fn holding(&self, file: &str, text: Option<&str>) -> Snapshot {
        let mut next = self.held();
        let files = &mut next.files;
        match (at(files, file), text) {
            (Ok(held), Some(text)) => files[held].1 = Text::from(text),
            (Err(place), Some(text)) => files.insert(place, (FileId::new(file), Text::from(text))),
            (Ok(held), None) => drop(files.remove(held)),
            (Err(_), None) => {}
        }
        Snapshot::of(next)
    }

    pub(crate) fn lacking(&self, file: &str) -> Snapshot {
        self.holding(file, None)
    }

    /// This snapshot without `file`.
    pub fn without_file(&self, file: &FileId) -> Snapshot {
        self.lacking(file.path())
    }

    /// This snapshot with `text` as the file `std/<module>.cp`
    /// (`"sparse"`, `"geo/hull"`): how a host whose build carries only a
    /// part of the standard library (the browser) adds a module to what
    /// its files are checked against. A module the build carries stays
    /// the build's, and the snapshot is then equal to this one; a name
    /// the standard library lacks is an error.
    pub fn with_std(&self, module: &str, text: &str) -> Result<Snapshot, String> {
        let mut std = self.0.std.clone();
        std.give(module, text)?;
        Ok(self.with_std_as(std))
    }

    /// This snapshot checked against only the modules a browser build
    /// carries, in any build, so a test can give it the rest as a page
    /// would. Modules given before are dropped.
    #[cfg(not(target_arch = "wasm32"))]
    #[doc(hidden)]
    pub fn with_embedded_std(&self) -> Snapshot {
        self.with_std_as(Arc::new(cpl_check::GivenStd::embedded()))
    }

    /// This snapshot checked against the standard library `other` is.
    pub fn with_std_of(&self, other: &Snapshot) -> Snapshot {
        self.with_std_as(other.0.std.clone())
    }

    fn with_std_as(&self, std: Arc<cpl_check::GivenStd>) -> Snapshot {
        Snapshot::of(Held { std, ..self.held() })
    }

    /// The text of the standard-library module `module` (`"geo/hull"`)
    /// the files are checked against: the build's, or the one given.
    pub fn std_text(&self, module: &str) -> Option<&str> {
        self.0.std.text(module)
    }

    /// The outline of the standard-library module `parts` names, or names
    /// a member of, when the snapshot lacks its text: how many of `parts`
    /// name the module, its path, and the outline.
    pub fn std_outline(&self, parts: &[String]) -> Option<(usize, String, &'static str)> {
        self.0.std.outline(parts)
    }

    pub fn text(&self, file: &FileId) -> Option<&Text> {
        self.text_at(file.path())
    }

    /// The text of the file at `path`, if the snapshot holds one.
    pub(crate) fn text_at(&self, path: &str) -> Option<&Text> {
        text_of(&self.0.files, path)
    }

    pub fn files(&self) -> impl Iterator<Item = (&FileId, &Text)> {
        self.0.files.iter().map(|(file, text)| (file, text))
    }

    /// Whether `other` is this very snapshot: a cheap test, never wrong
    /// when it says yes.
    pub(crate) fn same_files(&self, other: &Snapshot) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn world(&self) -> World {
        self.0.world
    }

    pub(crate) fn std(&self) -> &Arc<cpl_check::GivenStd> {
        &self.0.std
    }
}
