//! Contract 9: a session over a snapshot, and its check and build queries.

use std::sync::{Arc, Mutex, MutexGuard};

use cpl_diag::Diagnostic;

use crate::snapshot::{at, text_of, Files, Snapshot, Text, World};
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

/// The decorators that give `cpl stress` a judge: `@alt` (a whole program
/// or one part), the older `@brute`, `@check` and the older `@checker`,
/// and `@interactor`.
pub const JUDGE_DECORATORS: [&str; 5] = cpl_ir::STRESS_DECORATORS;

/// Whether `source` has a top-level fn `cpl stress` judges by: one of
/// [`JUDGE_DECORATORS`] on it, as the parser reads it (not a mention in a
/// comment or a string). Needs no check, so an editor asks it on each key.
pub fn has_judge(source: &str) -> bool {
    cpl_ir::has_stress_target(source)
}

/// The runtime checks a judge build or a run keeps (`cpl build --checks`, R09.3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Checks {
    /// No index, `/ 0` or `assert` traps: what hand-written C++ checks.
    None,
    /// Index, `/ 0` and `assert` stop the program with their message.
    #[default]
    Traps,
    /// [`Checks::Traps`], plus range stores, overflow and the other checks of the local
    /// build (`-DLOCAL`).
    All,
}

impl Checks {
    /// What every local run checks: `cpl test`, `cpl stress` and their builds of brutes and
    /// generators.
    pub const LOCAL: Checks = Checks::All;

    pub const NAMES: [&'static str; 3] = ["none", "traps", "all"];

    pub fn parse(name: &str) -> Option<Checks> {
        cpl_ir::checks::Checks::parse(name).map(Checks::from)
    }

    pub fn name(self) -> &'static str {
        cpl_ir::checks::Checks::from(self).name()
    }
}

impl From<Checks> for cpl_ir::checks::Checks {
    fn from(c: Checks) -> cpl_ir::checks::Checks {
        match c {
            Checks::None => cpl_ir::checks::Checks::None,
            Checks::Traps => cpl_ir::checks::Checks::Traps,
            Checks::All => cpl_ir::checks::Checks::All,
        }
    }
}

impl From<cpl_ir::checks::Checks> for Checks {
    fn from(c: cpl_ir::checks::Checks) -> Checks {
        match c {
            cpl_ir::checks::Checks::None => Checks::None,
            cpl_ir::checks::Checks::Traps => Checks::Traps,
            cpl_ir::checks::Checks::All => Checks::All,
        }
    }
}

/// An `if` of a program.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Branch {
    /// The `if` statement.
    pub test: cpl_diag::Span,
    /// The first statement of the branch it takes when true.
    pub taken: cpl_diag::Span,
}

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

    /// The fn the judge's grader calls first, when the file leaves `main`
    /// to a grader.
    pub fn grader_entry(&self) -> Option<&str> {
        let id = *self.ir.entry_fns.first()?;
        Some(&self.ir.fns[id].name)
    }

    /// Whether only its C++ build runs: it has an `@cpp` body.
    pub fn cpp_only(&self) -> bool {
        !self.ir.cpp_bodies.is_empty()
    }

    /// Where a value passed to a call is written back after it.
    pub fn written_back(&self) -> &[cpl_diag::Span] {
        &self.ir.written_back
    }

    /// The span of every statement the file's own code runs, imported
    /// modules' left out: what a profile or a coverage report counts.
    pub fn statements(&self) -> Vec<cpl_diag::Span> {
        cpl_ir::sites::user_spans(&self.ir)
    }

    /// Every `if` among [`Program::statements`], each once.
    pub fn branches(&self) -> Vec<Branch> {
        let counted = cpl_ir::sites::user_stmts(&self.ir);
        let mut blocks = vec![&self.ir.setup, &self.ir.main];
        blocks.extend(self.ir.fns.iter().map(|f| &f.body));
        blocks.extend(self.ir.closures.iter().map(|c| &c.body));
        let mut found = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for block in blocks {
            cpl_ir::walk::deep_stmts(block, &mut |stmt| {
                let cpl_ir::Stmt::If(_, then, _) = stmt else {
                    return;
                };
                let first = then.stmts.first().and_then(|first| first.span());
                let (Some(span), Some(first)) = (stmt.span(), first) else {
                    return;
                };
                if counted.contains(&(stmt as *const cpl_ir::Stmt)) && seen.insert(span) {
                    found.push(Branch {
                        test: span,
                        taken: first,
                    });
                }
            });
        }
        found
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
    /// The snapshot the last query synced `db` to, and which of its files
    /// the query was about.
    last: Option<(Snapshot, Option<usize>)>,
}

fn database(snapshot: &Snapshot, modules: &Files) -> cpl_check::Session {
    let libs = match snapshot.world() {
        World::Disk => cpl_check::Libraries::Discover,
        World::Closed => {
            let given = modules
                .iter()
                .map(|(file, text)| (file.path().to_string(), text.to_string()));
            let given = cpl_check::GivenFiles::new(given.collect());
            cpl_check::Libraries::Loader(Box::new(given))
        }
    };
    cpl_check::Session::with_options(cpl_check::SessionOptions {
        libs,
        ..Default::default()
    })
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
            db: database(&snapshot, &Files::new()),
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
    pub fn at(&self, snapshot: Snapshot) -> Session {
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

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    /// The standard-library modules checking `file` asks for that the
    /// snapshot lacks, by name (`"sparse"`), sorted: what a host gives
    /// ([`Snapshot::with_std`]) for the file's diagnostics to be about the
    /// file. Empty when the build carries all of it.
    pub fn std_needs(&self, file: &str) -> Vec<String> {
        let mut inner = self.asked(file);
        inner.db.analysis(file);
        inner.db.std_needs(file)
    }

    /// [`Session::std_needs`], and with them the modules those would ask
    /// for in turn as far as their outlines say: fewer rounds of giving.
    /// Ask again after giving them, until it names none.
    pub fn std_needed(&self, file: &str) -> Vec<String> {
        let text = self.snapshot.text_at(file);
        text.map(|text| self.snapshot.std().needed(file, text))
            .unwrap_or_default()
    }

    /// Today's checker, holding exactly this snapshot's files, for a query
    /// about `asked`.
    pub(crate) fn synced(&self, asked: Option<&str>) -> MutexGuard<'_, Inner> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let world = self.snapshot.world();
        let asked = asked.filter(|_| world == World::Closed);
        let asked = asked.and_then(|path| self.snapshot.place(path));
        if let Some((snapshot, file)) = &inner.last {
            if *file == asked && snapshot.same_files(&self.snapshot) {
                return inner;
            }
        }
        // On disk every file of the snapshot is open. In a closed world the
        // file asked about is, and the rest are where its imports are found.
        let mut programs = Files::new();
        let mut modules = Files::new();
        for (place, (file, text)) in self.snapshot.files().enumerate() {
            let open = world == World::Disk || Some(place) == asked;
            let among = if open { &mut programs } else { &mut modules };
            among.push((file.clone(), text.clone()));
        }
        if modules != inner.modules {
            inner.db = database(&self.snapshot, &modules);
            inner.programs.clear();
            inner.modules = modules;
        }
        let Inner {
            db, programs: held, ..
        } = &mut *inner;
        db.set_std(self.snapshot.std());
        for (file, _) in held.iter() {
            if text_of(&programs, file.path()).is_none() {
                db.remove_file(file.path());
            }
        }
        for (file, text) in &programs {
            let held = text_of(held, file.path());
            if !held.is_some_and(|held| Arc::ptr_eq(held, text) || held == text) {
                db.set_file(file.path(), text.clone());
            }
        }
        inner.programs = programs;
        inner.last = Some((self.snapshot.clone(), asked));
        inner
    }

    /// [`Session::synced`] for a query about the file at `path`.
    #[inline(never)]
    pub(crate) fn asked(&self, path: &str) -> MutexGuard<'_, Inner> {
        self.synced(Some(path))
    }

    pub(crate) fn checked(&self, file: &str) -> Option<Arc<Analysis>> {
        self.asked(file).db.analysis(file)
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
        let mut inner = self.asked(file);
        let found = search(&mut inner.db);
        if self.snapshot.text_at(file).is_none() {
            inner.db.remove_file(file);
        }
        if let Ok(held) = at(&inner.programs, file) {
            inner.programs.remove(held);
        }
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
