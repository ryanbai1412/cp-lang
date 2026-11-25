//! Contract 9: a session over a snapshot, and its check and build queries.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use cpl_diag::Diagnostic;

use crate::snapshot::{FileId, Snapshot, Text, World};
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

/// What the C++ is written for: the judge it is submitted to.
#[derive(Clone, Debug)]
pub struct Profile {
    /// A line for the header that names the judge.
    pub stamp: Option<String>,
    /// The stack the program runs with on a thread of its own; `None` on a
    /// judge that allows no threads.
    pub thread_stack: Option<u64>,
    /// Whether the judge runs Windows.
    pub windows: bool,
    pub checks: Checks,
    /// The judge's CPU features (`avx2`).
    pub cpu: Vec<String>,
    /// The source limit, when the judge has one.
    pub size: Option<SizeBar>,
    /// Whether the source opens the file as a comment. A file built to be
    /// run here, not submitted, leaves it out.
    pub source_comment: bool,
}

/// A judge's source limit: the file gives way (its source comment, then
/// its names) until it is at most `aim` bytes, and is refused over `limit`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SizeBar {
    pub aim: usize,
    pub limit: usize,
}

impl Default for Profile {
    fn default() -> Profile {
        let target = cpl_codegen::Target::default();
        Profile {
            stamp: target.stamp,
            thread_stack: target.thread_stack,
            windows: target.windows,
            checks: target.checks,
            cpu: target.cpu,
            size: None,
            source_comment: true,
        }
    }
}

impl Profile {
    pub(crate) fn target(&self) -> cpl_codegen::Target {
        cpl_codegen::Target {
            stamp: self.stamp.clone(),
            thread_stack: self.thread_stack,
            windows: self.windows,
            checks: self.checks,
            cpu: self.cpu.clone(),
        }
    }
}

/// The C++ file of a program.
pub struct CppFile {
    pub text: String,
    /// What the file left out to fit the profile's size, in the order it
    /// gave way. Empty with no size bar.
    pub left_out: Vec<&'static str>,
    /// How long it is as written, with nothing left out.
    pub full: usize,
    /// Whether it is still over the profile's limit.
    pub over_limit: bool,
    fitted: Option<cpl_codegen::Fitted>,
}

/// Where the bytes of a C++ file go.
pub type Size = cpl_codegen::Size;
/// What a [`Size`] measures.
pub type Part = cpl_codegen::Part;

/// A program built from a file, ready to run or to emit.
#[derive(Clone, PartialEq, Debug)]
pub struct Program {
    pub(crate) ir: Arc<cpl_ir::Program>,
    text: Text,
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
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Roles {
    /// The `@interactor` fn: the judge's side of a conversation.
    pub interactor: Option<String>,
    /// The `@alt(program)` fn and how many parameters it takes.
    pub alt: Option<(String, u32)>,
    /// The `@gen` fn and how many parameters it takes.
    pub generator: Option<(String, u32)>,
    /// The `@check` fn and how many parameters it takes.
    pub checker: Option<(String, u32)>,
    /// The `@test`s, in order.
    pub tests: Vec<String>,
    /// Whether a struct has a `@cached` field.
    pub cached_fields: bool,
}

/// [`Session::diagnostics`] of a file already checked.
pub(crate) fn diagnostics_of(analysis: &Analysis, upto: Layer) -> Vec<Diagnostic> {
    match upto {
        Layer::Typed => analysis.diagnostics.clone(),
        Layer::Program => cpl_ir::reported(analysis),
        Layer::Built => cpl_ir::check(analysis),
    }
}

/// [`Session::roles`] of a file already checked.
pub(crate) fn roles_of(analysis: &Analysis) -> Roles {
    Roles {
        interactor: cpl_ir::interactor(analysis),
        alt: cpl_ir::brute(analysis),
        generator: cpl_ir::generator(analysis),
        checker: cpl_ir::checker(analysis),
        tests: cpl_ir::list_tests(analysis),
        cached_fields: cpl_ir::has_cached_fields(analysis),
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
        ir: Arc::new(ir),
    })
}

pub(crate) struct Inner {
    pub(crate) db: cpl_check::Session,
    /// The files `db` was told are open, as it has them.
    programs: BTreeMap<FileId, Text>,
    /// In a closed world, the files `db` finds modules among.
    modules: BTreeMap<FileId, Text>,
    /// The snapshot and the file the last query synced `db` to.
    last: Option<(Snapshot, Option<FileId>)>,
}

fn database(world: World, modules: &BTreeMap<FileId, Text>) -> cpl_check::Session {
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
            db: database(snapshot.world(), &BTreeMap::new()),
            programs: BTreeMap::new(),
            modules: BTreeMap::new(),
            last: None,
        };
        Session {
            snapshot,
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// The session of this snapshot with `file` holding `text`. It shares
    /// everything else, and this session still answers for its own snapshot.
    pub fn with_file(&self, file: impl Into<FileId>, text: impl Into<Text>) -> Session {
        Session {
            snapshot: self.snapshot.with_file(file, text),
            inner: self.inner.clone(),
        }
    }

    /// The session of this snapshot without `file`.
    pub fn without_file(&self, file: impl Into<FileId>) -> Session {
        Session {
            snapshot: self.snapshot.without_file(&file.into()),
            inner: self.inner.clone(),
        }
    }

    /// Moves a tool's handle on to [`Session::with_file`]'s session.
    pub fn set_file(&mut self, file: impl Into<FileId>, text: impl Into<Text>) {
        *self = self.with_file(file, text);
    }

    /// Moves a tool's handle on to [`Session::without_file`]'s session.
    pub fn remove_file(&mut self, file: impl Into<FileId>) {
        *self = self.without_file(file);
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
        let (programs, modules): (BTreeMap<FileId, Text>, BTreeMap<FileId, Text>) = match world {
            World::Disk => (all().collect(), BTreeMap::new()),
            World::Closed => all().partition(|(file, _)| Some(file) == asked),
        };
        if modules != inner.modules {
            inner.db = database(world, &modules);
            inner.programs.clear();
            inner.modules = modules;
        }
        let gone: Vec<FileId> = inner
            .programs
            .keys()
            .filter(|file| !programs.contains_key(*file))
            .cloned()
            .collect();
        for file in gone {
            inner.db.remove_file(file.path());
        }
        for (file, text) in &programs {
            let held = inner.programs.get(file);
            if !held.is_some_and(|held| Arc::ptr_eq(held, text) || held == text) {
                inner.db.set_file(file.path(), text.clone());
            }
        }
        inner.programs = programs;
        inner.last = Some((self.snapshot.clone(), asked.cloned()));
        inner
    }

    pub(crate) fn checked(&self, file: &FileId) -> Option<Arc<Analysis>> {
        self.synced(Some(file)).db.analysis(file.path())
    }

    /// Every diagnostic `file` has, up to `upto`. One answer for every
    /// tool: `cpl check`, the editor and the playground print this list.
    pub fn diagnostics(&self, file: impl Into<FileId>, upto: Layer) -> Vec<Diagnostic> {
        let checked = self.checked(&file.into());
        checked
            .map(|analysis| diagnostics_of(&analysis, upto))
            .unwrap_or_default()
    }

    /// The fns of `file` that make it more than one program.
    pub fn roles(&self, file: impl Into<FileId>) -> Roles {
        self.checked(&file.into())
            .map(|analysis| roles_of(&analysis))
            .unwrap_or_default()
    }

    /// The program `job` asks for of `file`, or every diagnostic of the
    /// file that stops it: the typed layer's when it has an error, and
    /// otherwise those with what the build itself rejects. `Ok` leaves the
    /// typed layer's warnings to [`Session::diagnostics`].
    pub fn program(&self, file: impl Into<FileId>, job: &Job) -> Result<Program, Vec<Diagnostic>> {
        let Some(analysis) = self.checked(&file.into()) else {
            return Err(Vec::new());
        };
        let mut diagnostics = analysis.diagnostics.clone();
        if analysis.has_errors() {
            return Err(diagnostics);
        }
        build_of(&analysis, job).map_err(|errors| {
            diagnostics.extend(errors);
            diagnostics
        })
    }

    /// The C++17 file of `program`, written from the file called `name`,
    /// for the judge `profile` describes. The source opens it as a comment.
    pub fn emit_cpp(&self, program: &Program, name: &str, profile: &Profile) -> CppFile {
        emit_cpp(program, name, profile)
    }
}

/// [`Session::emit_cpp`]: a built program holds all its C++ needs.
pub fn emit_cpp(program: &Program, name: &str, profile: &Profile) -> CppFile {
    let target = profile.target();
    let Some(bar) = profile.size else {
        let text = cpl_codegen::emit_for(
            &program.ir,
            name,
            profile.source_comment.then_some(&*program.text),
            &target,
        );
        return CppFile {
            full: text.len(),
            text,
            left_out: Vec::new(),
            over_limit: false,
            fitted: None,
        };
    };
    let fitted = cpl_codegen::emit_aiming(
        &program.ir,
        name,
        &program.text,
        (bar.aim, bar.limit),
        &target,
    );
    let over_limit = fitted.is_err();
    let fitted = fitted.unwrap_or_else(|over| over);
    CppFile {
        text: fitted.cpp.clone(),
        left_out: fitted.left_out(),
        full: fitted.full,
        over_limit,
        fitted: Some(fitted),
    }
}

impl CppFile {
    /// What was left out to fit, said to the contestant of `file` built
    /// for `judge`, whose size `bar` words; `None` when nothing was.
    pub fn note(&self, file: &str, bar: &str, judge: &str) -> Option<String> {
        self.fitted.as_ref()?.note(file, bar, judge)
    }

    /// Where the bytes of this file, fitted to `profile`'s size from
    /// `program`, go: every piece, adding up to its length.
    pub fn sizes(&self, program: &Program, name: &str, profile: &Profile) -> Vec<Size> {
        let Some(fitted) = &self.fitted else {
            return Vec::new();
        };
        let source = fitted.header.then_some(&*program.text);
        cpl_codegen::sizes(&program.ir, name, source, &profile.target(), fitted)
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
        inner.programs.remove(&id);
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

/// The C++ of `program` that counts how often each statement runs and
/// writes the counts to the file `tally` when it ends, and the span of
/// each statement counted, in the order of the counts.
pub fn emit_cpp_profiled(
    program: &Program,
    name: &str,
    profile: &Profile,
    tally: &str,
) -> (String, Vec<cpl_diag::Span>) {
    cpl_codegen::emit_profiled(&program.ir, name, &profile.target(), tally)
}
