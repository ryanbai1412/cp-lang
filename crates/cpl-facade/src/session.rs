//! Contract 9: a session over a snapshot, and its check and build queries.

use std::sync::{Arc, Mutex, MutexGuard};

use cpl_diag::Diagnostic;

use crate::snapshot::{text_of, FileId, Files, Snapshot, Text, World};
use crate::today::Analysis;

/// How far a file is taken before its diagnostics are reported.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    /// Syntax, names and types: what one file says on its own.
    Typed,
    /// Everything `run` and `build` would reject: the typed layer, then the
    /// whole program each of the file's roles (solution, `@interactor`,
    /// `@alt`, `@gen`, `@check`) makes.
    Program,
    /// The program layer of a file taken as a program even when it only
    /// declares (a library module, which `Program` leaves to the programs
    /// that use it), and with what is not built yet named: what judges an
    /// automatic fix, which must not break a file for any use.
    Built,
}

/// Which program of a file to build.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Root {
    /// The file's statements, top to bottom.
    Program,
    /// One `@test`, by name.
    Test(String),
    /// The `@interactor` fn, by name: the judge's side of a conversation.
    Interactor(String),
    /// The `@alt(program)` fn, by name, where the solution is called.
    Alt(String),
    /// The `@check` fn, by name.
    Checker(String),
    /// The `@gen` fn, by name, at a size.
    Generator(String, i64),
    /// The `@gen` fn, by name, reading its parameters from the input; the
    /// number is the line its call is reported at.
    GeneratorRead(String, u32),
    /// The file's `law`s, each sampled.
    Laws,
}

/// What to build from a file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Job {
    pub root: Root,
    /// Start every `@cached` field empty, as a first call finds it.
    pub cold_caches: bool,
}

impl Job {
    pub fn of(root: Root) -> Job {
        Job {
            root,
            cold_caches: false,
        }
    }

    /// The file with every `@cached` field starting empty.
    pub fn cold() -> Job {
        Job {
            root: Root::Program,
            cold_caches: true,
        }
    }

    /// The file as `cpl run` runs it.
    pub fn program() -> Job {
        Job::of(Root::Program)
    }
}

/// The runtime checks a build or a run keeps.
pub type Checks = cpl_ir::checks::Checks;

/// A program built from a file, ready to run or to emit.
#[derive(Clone, PartialEq, Debug)]
pub struct Program {
    pub(crate) ir: cpl_ir::Program,
    pub(crate) text: Text,
    warnings: Arc<[Diagnostic]>,
}

impl Program {
    /// The text of the file it was built from.
    pub fn text(&self) -> &Text {
        &self.text
    }

    /// Whether it talks to a judge: it has an `@interactor`.
    pub fn interactive(&self) -> bool {
        self.ir.interactive
    }

    /// What building it warned of, after the typed layer's own warnings.
    pub fn warnings(&self) -> &[Diagnostic] {
        &self.warnings
    }
}

/// The fns of a file that make it more than one program, by name.
pub struct Roles(pub(crate) Arc<Analysis>);

impl Roles {
    /// The `@interactor` fn: the judge's side of a conversation.
    pub fn interactor(&self) -> Option<String> {
        cpl_ir::interactor(&self.0)
    }

    /// The `@alt(program)` fn and how many parameters it takes.
    pub fn alt(&self) -> Option<(String, u32)> {
        cpl_ir::brute(&self.0)
    }

    /// The `@gen` fn and how many parameters it takes.
    pub fn generator(&self) -> Option<(String, u32)> {
        cpl_ir::generator(&self.0)
    }

    /// The `@check` fn and how many parameters it takes.
    pub fn checker(&self) -> Option<(String, u32)> {
        cpl_ir::checker(&self.0)
    }

    /// The `@test`s, in order.
    pub fn tests(&self) -> Vec<String> {
        cpl_ir::list_tests(&self.0)
    }

    /// Whether a struct has a `@cached` field.
    pub fn cached_fields(&self) -> bool {
        cpl_ir::has_cached_fields(&self.0)
    }
}

/// [`Session::diagnostics`] of a file already checked.
pub(crate) fn diagnostics_of(analysis: &Analysis, upto: Layer) -> Vec<Diagnostic> {
    match upto {
        Layer::Typed => analysis.diagnostics.clone(),
        Layer::Program => cpl_ir::reported(analysis),
        Layer::Built => cpl_ir::check(analysis),
    }
}

/// [`Session::program`] of a file already checked: `Err` is what the build
/// itself rejects, with the typed layer's diagnostics left to the caller.
pub(crate) fn build_of(analysis: &Analysis, job: &Job) -> Result<Program, Vec<Diagnostic>> {
    let lowered = match &job.root {
        Root::Program if job.cold_caches => cpl_ir::lower_cold_cache(analysis),
        Root::Program => cpl_ir::lower(analysis),
        Root::Laws => cpl_ir::lower_sampling_laws(analysis),
        Root::Test(name) => cpl_ir::lower_test(analysis, name),
        Root::Interactor(name) => cpl_ir::lower_judge(analysis, name),
        Root::Alt(name) => cpl_ir::lower_brute(analysis, name),
        Root::Checker(name) => cpl_ir::lower_checker(analysis, name),
        Root::Generator(name, size) => cpl_ir::lower_gen(analysis, name, *size),
        Root::GeneratorRead(name, line) => cpl_ir::lower_gen_read(analysis, name, *line),
    };
    lowered.map(|mut ir| Program {
        warnings: std::mem::take(&mut ir.warnings).into(),
        text: analysis.text.clone(),
        ir,
    })
}

pub(crate) struct Inner {
    pub(crate) db: cpl_check::Session,
    /// The files `db` was told are open, as it has them.
    programs: Files,
    /// In a closed world, the files `db` finds modules among.
    modules: Files,
    /// The snapshot and the file the last query synced `db` to.
    last: Option<(Snapshot, Option<FileId>)>,
}

fn database(world: World, modules: &Files) -> cpl_check::Session {
    match world {
        World::Disk => cpl_check::Session::new(),
        World::Closed => {
            let given = modules
                .iter()
                .map(|(file, text)| (file.path().to_string(), text.to_string()));
            cpl_check::Session::with_loader(Box::new(cpl_check::GivenFiles::new(given.collect())))
        }
    }
}

/// A snapshot and the answers already worked out for it and its
/// neighbours. Cheap to clone; [`Session::with_file`] shares the work.
#[derive(Clone)]
pub struct Session {
    snapshot: Snapshot,
    inner: Arc<Mutex<Inner>>,
}

/// The session of no files, with imports found on disk.
impl Default for Session {
    fn default() -> Session {
        Session::open(Snapshot::default())
    }
}

impl Session {
    pub fn open(snapshot: Snapshot) -> Session {
        let inner = Inner {
            db: database(snapshot.world(), &Files::new()),
            programs: Files::new(),
            modules: Files::new(),
            last: None,
        };
        Session {
            snapshot,
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// The session of this snapshot with `file` holding `text`. It shares
    /// everything else, and this session still answers for its own snapshot.
    #[inline(always)]
    pub fn with_file(&self, file: impl AsRef<str>, text: impl AsRef<str>) -> Session {
        self.at(self.snapshot.with_file(file, text))
    }

    /// This session's work, answering for `snapshot`.
    fn at(&self, snapshot: Snapshot) -> Session {
        Session {
            snapshot,
            inner: self.inner.clone(),
        }
    }

    /// The session of this snapshot without `file`.
    #[inline(always)]
    pub fn without_file(&self, file: impl AsRef<str>) -> Session {
        self.at(self.snapshot.lacking(file.as_ref()))
    }

    /// Moves a tool's handle on to [`Session::with_file`]'s session.
    #[inline(always)]
    pub fn set_file(&mut self, file: impl AsRef<str>, text: impl AsRef<str>) {
        *self = self.at(self.snapshot.with_file(file, text));
    }

    /// Moves a tool's handle on to [`Session::without_file`]'s session.
    #[inline(always)]
    pub fn remove_file(&mut self, file: impl AsRef<str>) {
        *self = self.at(self.snapshot.lacking(file.as_ref()));
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    /// Today's checker, holding exactly this snapshot's files, for a query
    /// about `asked`.
    pub(crate) fn synced(&self, asked: Option<&FileId>) -> MutexGuard<'_, Inner> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let world = self.snapshot.world();
        let asked = asked.filter(|_| world == World::Closed);
        if let Some((snapshot, file)) = &inner.last {
            if snapshot.same_files(&self.snapshot) && file.as_ref() == asked {
                return inner;
            }
        }
        let all = || {
            self.snapshot
                .files()
                .map(|(file, text)| (file.clone(), text.clone()))
        };
        // On disk every file of the snapshot is open. In a closed world the
        // file asked about is, and the rest are where its imports are found.
        let (programs, modules): (Files, Files) =
            all().partition(|(file, _)| world == World::Disk || Some(file) == asked);
        if modules != inner.modules {
            inner.db = database(world, &modules);
            inner.programs.clear();
            inner.modules = modules;
        }
        let Inner {
            db, programs: held, ..
        } = &mut *inner;
        for (file, _) in held.iter() {
            if text_of(&programs, file).is_none() {
                db.remove_file(file.path());
            }
        }
        for (file, text) in &programs {
            let held = text_of(held, file);
            if !held.is_some_and(|held| Arc::ptr_eq(held, text) || held == text) {
                db.set_file(file.path(), text.clone());
            }
        }
        inner.programs = programs;
        inner.last = Some((self.snapshot.clone(), asked.cloned()));
        inner
    }

    pub(crate) fn checked(&self, file: &str) -> Option<Arc<Analysis>> {
        let file = FileId::new(file);
        self.synced(Some(&file)).db.analysis(file.path())
    }

    /// Every diagnostic `file` has, up to `upto`. One answer for every
    /// tool: `cpl check`, the editor and the playground print this list.
    #[inline(always)]
    pub fn diagnostics(&self, file: impl AsRef<str>, upto: Layer) -> Vec<Diagnostic> {
        self.diagnostics_at(file.as_ref(), upto)
    }

    fn diagnostics_at(&self, file: &str, upto: Layer) -> Vec<Diagnostic> {
        let checked = self.checked(file);
        checked
            .map(|analysis| diagnostics_of(&analysis, upto))
            .unwrap_or_default()
    }

    /// The fns of `file` that make it more than one program.
    #[inline(always)]
    pub fn roles(&self, file: impl AsRef<str>) -> Option<Roles> {
        self.checked(file.as_ref()).map(Roles)
    }

    /// The program `job` asks for of `file`, or every diagnostic of the
    /// file that stops it: the typed layer's when it has an error, and
    /// otherwise those with what the build itself rejects. `Ok` leaves the
    /// typed layer's warnings to [`Session::diagnostics`].
    #[inline(always)]
    pub fn program(&self, file: impl AsRef<str>, job: &Job) -> Result<Program, Vec<Diagnostic>> {
        self.program_at(file.as_ref(), job)
    }

    fn program_at(&self, file: &str, job: &Job) -> Result<Program, Vec<Diagnostic>> {
        let Some(analysis) = self.checked(file) else {
            return Err(Vec::new());
        };
        if analysis.has_errors() {
            return Err(analysis.diagnostics.clone());
        }
        build_of(&analysis, job).map_err(|errors| {
            let mut diagnostics = analysis.diagnostics.clone();
            diagnostics.extend(errors);
            diagnostics
        })
    }
}

/// Fixes: the ones a diagnostic offers, tried against the checker.
impl Session {
    /// Lends today's checker to a fix search that tries texts of `file`
    /// in it, and afterwards forgets what it held for `file`, so the next
    /// query puts the snapshot's text back.
    fn trying<R>(&self, file: &str, search: impl FnOnce(&mut cpl_check::Session) -> R) -> R {
        let id = FileId::new(file);
        let mut inner = self.synced(Some(&id));
        let found = search(&mut inner.db);
        if self.snapshot.text(&id).is_none() {
            inner.db.remove_file(file);
        }
        inner.programs.retain(|(file, _)| *file != id);
        inner.last = None;
        found
    }

    /// The fix of `diagnostic` to offer for `text` as `file`: the first
    /// after which the file checks clean, else the first that leaves no
    /// error on or before its line, else its first.
    pub fn checked_fix<'d>(
        &self,
        file: &str,
        text: &str,
        diagnostic: &'d Diagnostic,
    ) -> Option<&'d cpl_diag::Fix> {
        self.trying(file, |db| {
            cpl_ir::fix::checked_fix(db, file, text, diagnostic)
        })
    }

    /// The errors of `text` as `file`, at [`Layer::Built`].
    pub fn errors_in(&self, file: &str, text: &str) -> Vec<Diagnostic> {
        self.trying(file, |db| cpl_ir::fix::errors_in(db, file, text))
    }

    /// `text` as `file` with the checked fix of each error applied, round
    /// after round until none applies; how many were applied; and the
    /// errors left.
    pub fn apply_error_fixes_in(&self, file: &str, text: &str) -> (String, usize, Vec<Diagnostic>) {
        self.trying(file, |db| cpl_ir::fix::apply_error_fixes_in(db, file, text))
    }
}

/// `text` as `file` with the checked fix of each error applied
/// ([`Session::apply_error_fixes_in`]), and how many were applied.
pub fn apply_error_fixes(file: &str, text: &str) -> (String, usize) {
    let (text, applied, _) = Session::default().apply_error_fixes_in(file, text);
    (text, applied)
}
