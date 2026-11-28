//! Contract 9 over the new pipeline (`handbook/design/next/facade.md`): the only way a tool
//! checks, builds and runs a program, over an immutable [`Snapshot`]. Each answer is computed
//! once, on the first query that needs it, and never changes.

pub use cpl_contract::{Definition, FileId, Text, World};
use cpl_contract::{Hover, Typed};
use cpl_next_decls::modules;
use cpl_next_ir::build::Judge;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

pub use cpl_next_ir::build::Checks;

use crate::linked::{Built, Job, Linked, Shared, StdLib};
use crate::{
    bound_line, dumped, items, std_unchecked, types, units, Compiled, Diag, Form, Front, IrStage,
    Item, Outcome, Roots, Stage, Std, Type, TypedAt, Unit,
};
use cpl_next_db::db::{Db, DbError};
use cpl_next_db::items::{Libraries, ModuleFile, Source};

/// The only input of every query (contract 0): the files, where the libraries they use are
/// found, and std. Cheap to clone; equal snapshots give equal answers.
#[derive(Clone)]
pub struct Snapshot(Arc<Held>);

#[derive(Clone)]
struct Held {
    /// In the order of their ids, each once.
    files: Vec<(FileId, Text)>,
    world: World,
    std: Arc<Std>,
    events: bool,
    bodies: bool,
    judge: Option<Judge>,
    checks: Checks,
}

impl Snapshot {
    /// No files yet; the libraries they use are found in `world`, and std is the one built in.
    pub fn new(world: World) -> Snapshot {
        Snapshot(Arc::new(Held {
            files: Vec::new(),
            world,
            std: Arc::new(Std::read()),
            events: false,
            bodies: false,
            judge: None,
            checks: Checks::CHECKED,
        }))
    }

    /// This snapshot with `file` holding `text`.
    pub fn with_file(&self, file: impl AsRef<str>, text: impl AsRef<str>) -> Snapshot {
        let (file, text) = (FileId::new(file.as_ref()), Text::from(text.as_ref()));
        self.changed(|h| match h.files.binary_search_by(|(f, _)| f.cmp(&file)) {
            Ok(at) => h.files[at].1 = text,
            Err(at) => h.files.insert(at, (file, text)),
        })
    }

    /// This snapshot without `file`.
    pub fn without_file(&self, file: &FileId) -> Snapshot {
        self.changed(|h| h.files.retain(|(f, _)| f != file))
    }

    /// This snapshot checked against `std`, which whoever builds the snapshot read.
    pub fn with_std_lib(&self, std: Arc<Std>) -> Snapshot {
        self.changed(|h| h.std = std)
    }

    pub fn text(&self, file: &FileId) -> Option<&Text> {
        let files = &self.0.files;
        (files.binary_search_by(|(f, _)| f.cmp(file)))
            .ok()
            .map(|at| &files[at].1)
    }

    pub fn files(&self) -> impl Iterator<Item = (&FileId, &Text)> {
        self.0.files.iter().map(|(f, t)| (f, t))
    }

    pub fn world(&self) -> World {
        self.0.world
    }

    /// Its runs report the semantic events, for trace.
    pub fn traced(&self) -> Snapshot {
        self.changed(|h| h.events = true)
    }

    /// Every `@fast` fn runs its CP body, not its op.
    pub fn with_bodies(&self) -> Snapshot {
        self.changed(|h| h.bodies = true)
    }

    /// Built for the judge `name`, whose memory limit is `memory_mb`. The tool resolves which
    /// judge that is (a `// judge:` line, a project config, a flag); the compiler only reads it.
    pub fn with_judge(&self, name: &str, memory_mb: u64) -> Snapshot {
        let judge = Judge {
            name: name.into(),
            memory_mb,
        };
        self.changed(|h| h.judge = Some(judge))
    }

    /// This snapshot built keeping `checks`: a checked build keeps all, release what
    /// `with checks(..: on)` and the judge's `release_checks:` keep (contract 0).
    pub fn with_checks(&self, checks: Checks) -> Snapshot {
        self.changed(|h| h.checks = checks)
    }

    fn changed(&self, change: impl FnOnce(&mut Held)) -> Snapshot {
        let mut held = Held::clone(&self.0);
        change(&mut held);
        Snapshot(Arc::new(held))
    }
}

/// How far down the layers a diagnostic query looks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    /// Syntax, declarations, the typed tree and lowering: what each unit says on its own.
    Typed,
    /// And what ownership and link find in the whole program.
    Program,
}

/// What a program's diagnostics say about it, the one exit rule every tool reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    Accepted,
    /// A layer found an error in the program.
    Rejected,
    /// The pipeline does not build something the program uses; that says nothing about the
    /// program. A syntax error rejects it whatever a later layer refused.
    Refused,
}

/// A checked and built program: what it runs and what it prints as C++.
pub struct Program(pub(crate) Compiled);

impl Program {
    /// Runs on the engine, on the calling thread.
    pub fn run(&self, input: &str) -> Outcome {
        self.0.run(input)
    }

    /// Runs `input` on the engine, on the calling thread.
    pub fn run_on(&self, input: cpl_next_ir::run::RunInput) -> Outcome {
        self.0.run_on(input)
    }

    /// A run that reads `from` as the program reads and writes `to` as it flushes (R10.4).
    pub fn streamed(
        &self,
        from: Box<dyn std::io::Read + Send>,
        to: Box<dyn std::io::Write + Send>,
    ) -> cpl_next_ir::run::RunInput {
        self.0.streamed(from, to)
    }

    /// What the reference interpreter runs in `form`: the tests' oracle, which nothing else runs.
    pub fn form(
        &self,
        form: Form,
    ) -> (
        &cpl_next_link::Program,
        &[cpl_next_ir::ir::Func<cpl_next_ir::prog::CTy>],
    ) {
        self.0.form(form)
    }

    pub fn cpp(&self) -> String {
        self.0.cpp()
    }

    /// A run of all of `input` under the judge's limits, for any engine.
    pub fn input(&self, input: &str) -> cpl_next_ir::run::RunInput {
        self.0.input(input)
    }

    /// What trap `t` of a run writes to stderr, its lines found from the source.
    pub fn said(&self, t: &crate::Trap) -> String {
        self.0.said(t)
    }

    pub fn proved(&self) -> &[cpl_next_ir::facts::Proved] {
        self.0.proved()
    }

    /// The instances holding an error type, which the checker should have refused or resolved.
    pub fn error_typed(&self) -> Vec<String> {
        self.0.error_typed()
    }
}

/// What a session answers for one file of its snapshot, each computed once.
#[derive(Default)]
struct PerFile {
    units: OnceLock<(Vec<Unit>, Vec<Diag>)>,
    front: OnceLock<Front>,
    items: OnceLock<HashMap<String, Item>>,
    types: OnceLock<Vec<TypedAt>>,
}

/// Where the libraries `file` uses are: the `cp.toml` at or above it on disk, or none past
/// std in a closed world.
fn roots(world: World, file: &FileId) -> Roots {
    match world {
        World::Disk => {
            let dir = Path::new(file.path())
                .parent()
                .filter(|d| !d.as_os_str().is_empty());
            Roots::from(dir.unwrap_or(Path::new(".")))
        }
        World::Closed => Roots::std(),
    }
}

/// What a session and every session [`Session::at`] makes from it share: one database, the
/// snapshot files its inputs hold now, and the library files it read from disk.
#[derive(Default)]
struct Work {
    db: Db,
    files: Vec<(FileId, Text)>,
    read: Vec<FileId>,
}

/// The answers for one snapshot, per file. A file the snapshot does not hold is no program and
/// says nothing.
pub struct Session {
    snapshot: Snapshot,
    /// In the order of the snapshot's files.
    files: Vec<PerFile>,
    work: Arc<Mutex<Work>>,
}

impl Session {
    pub fn open(snapshot: Snapshot) -> Session {
        let files = snapshot
            .0
            .files
            .iter()
            .map(|_| PerFile::default())
            .collect();
        Session {
            snapshot,
            files,
            work: Arc::default(),
        }
    }

    /// A session for `snapshot` sharing this one's database: an answer whose inputs are the
    /// same in both is computed once. This session still answers for its own snapshot.
    pub fn at(&self, snapshot: Snapshot) -> Session {
        Session {
            work: self.work.clone(),
            ..Session::open(snapshot)
        }
    }

    /// The shared database with its inputs this session's snapshot, holding every file the
    /// program `root` uses that can be read. Only the inputs that differ from what it held
    /// change: a file's text that changed, a file that went, a library file that changed on
    /// disk.
    fn work(&self, root: &FileId) -> Result<MutexGuard<'_, Work>, DbError> {
        let mut w = self.work.lock().unwrap_or_else(|e| e.into_inner());
        let h = &self.snapshot.0;
        if w.files != h.files {
            let held = |f: &FileId| h.files.binary_search_by(|(g, _)| g.cmp(f)).is_ok();
            let mut read = Vec::new();
            for f in std::mem::take(&mut w.read).into_iter().filter(|f| !held(f)) {
                match std::fs::read_to_string(f.path()) {
                    Ok(text) => w.db.set::<Source>(f.clone(), text.into())?,
                    Err(_) => w.db.remove::<Source>(&f)?,
                }
                read.push(f);
            }
            for (f, _) in w.files.iter().filter(|(f, _)| !held(f)) {
                w.db.remove::<Source>(f)?;
                w.db.remove::<Libraries>(f)?;
            }
            for (f, t) in &h.files {
                w.db.set::<Source>(f.clone(), t.clone())?;
                (w.db).set::<Libraries>(f.clone(), roots(h.world, f).libraries())?;
            }
            (w.files, w.read) = (h.files.clone(), read);
        }
        w.db.set::<StdLib>((), Shared(h.std.clone()))?;
        let lacks = || {
            modules::load(&w.db, &h.std, root)
                .missing
                .into_iter()
                .map(|(_, p)| p)
        };
        let read = crate::read_missing(&w.db, || lacks().collect())?;
        w.read.extend(read);
        Ok(w)
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    /// `file`'s id, its text and its answers.
    fn file(&self, file: &str) -> Option<(&FileId, &str, &PerFile)> {
        let files = &self.snapshot.0.files;
        let at = files.binary_search_by(|(f, _)| f.path().cmp(file)).ok()?;
        Some((&files[at].0, &files[at].1, &self.files[at]))
    }

    /// `file` as a program, or every diagnostic that stops it, in layer order.
    pub fn program(&self, file: &str) -> Result<Arc<Program>, Arc<[Diag]>> {
        let Some((id, _, _)) = self.file(file) else {
            return Err(Arc::from(Vec::new()));
        };
        let h = &self.snapshot.0;
        let job = Job {
            root: id.clone(),
            events: h.events,
            bodies: h.bodies,
            judge: h.judge.clone(),
            checks: h.checks,
        };
        match self.work(id).and_then(|w| w.db.get::<Linked>(&job)) {
            Ok(Built(built)) => built,
            Err(e) => Err(Arc::from(vec![Diag::internal(&e)])),
        }
    }

    /// `file`'s IR at `stage`, for tracing a layer by eye (`cpl-next dump-ir`): lowering's
    /// fns the program instantiates, by name, and each later stage's in instance order, which
    /// is what a call names.
    pub fn ir(&self, file: &str, stage: IrStage) -> Result<String, Arc<[Diag]>> {
        let program = || self.program(file);
        Ok(match stage {
            IrStage::Raw => {
                let p = program()?;
                let Some(front) = self.front(file) else {
                    return Err(Arc::from(Vec::new()));
                };
                let reached: std::collections::HashSet<_> =
                    p.0.prog.insts.iter().map(|f| &f.key).collect();
                let mut fs: Vec<_> = (front.g.funcs.values())
                    .filter(|f| reached.contains(&f.key))
                    .collect();
                fs.sort_by_cached_key(|f| (f.name.clone(), format!("{:?}", f.key)));
                dumped(fs)
            }
            IrStage::Owned => dumped(&program()?.0.prog.insts),
            IrStage::Canonical => dumped(&program()?.0.canon),
            IrStage::Optimized => dumped(&program()?.0.opt),
            // Ownership's own record, so also for a program ownership rejects.
            IrStage::Borrows => {
                let Some((id, _, _)) = self.file(file) else {
                    return Err(Arc::from(Vec::new()));
                };
                let Ok(w) = self.work(id) else {
                    return Err(Arc::from(Vec::new()));
                };
                let h = &self.snapshot.0;
                let mut f = crate::front(&w.db, &h.std, id, h.judge.as_ref(), h.checks);
                match crate::back(&mut f, false) {
                    Some((prog, _, binds)) => {
                        let globals: Vec<String> =
                            f.t.globals.iter().map(|g| g.name.clone()).collect();
                        (binds.iter())
                            .map(|b| bound_line(&prog, &globals, b))
                            .collect()
                    }
                    None => return Err(Arc::from(f.diags)),
                }
            }
        })
    }

    /// Every diagnostic of `file`, up to `upto`.
    pub fn diagnostics(&self, file: &str, upto: Layer) -> Vec<Diag> {
        let Err(all) = self.program(file) else {
            return Vec::new();
        };
        let whole = |d: &&Diag| matches!(d.stage, Stage::Own | Stage::Link);
        (all.iter())
            .filter(|d| upto == Layer::Program || !whole(d))
            .cloned()
            .collect()
    }

    /// Each fn `file` declares and each of its statements, with its diagnostics, and every
    /// diagnostic of it: the editor's view of a program that need not compile.
    pub fn units(&self, file: &str) -> (&[Unit], &[Diag]) {
        let Some((id, _, f)) = self.file(file) else {
            return (&[], &[]);
        };
        let (units, diags) = f.units.get_or_init(|| {
            let h = &self.snapshot.0;
            match self.work(id) {
                Ok(w) => units(&w.db, &h.std, id, h.judge.as_ref(), h.checks),
                Err(e) => (Vec::new(), vec![Diag::internal(&e)]),
            }
        });
        (units, diags)
    }

    /// `file`'s checked front half, which hover, definition and types read.
    fn front(&self, file: &str) -> Option<&Front> {
        let (id, _, f) = self.file(file)?;
        if let Some(front) = f.front.get() {
            return Some(front);
        }
        let w = self.work(id).ok()?;
        Some(f.front.get_or_init(|| {
            let h = &self.snapshot.0;
            crate::front(&w.db, &h.std, id, h.judge.as_ref(), h.checks)
        }))
    }

    /// The item the name at byte `at` of `file` names, if it names one.
    fn item_at(&self, file: &str, at: u32) -> Option<&Item> {
        let (_, text, f) = self.file(file)?;
        let (_, name) = word_at(text, at as usize)?;
        let front = self.front(file)?;
        f.items.get_or_init(|| items(front)).get(name)
    }

    /// The item named at byte `offset` of `file`: its declaration's first line, its doc and
    /// the cost the doc states.
    pub fn hover(&self, file: &str, offset: u32) -> Option<Hover> {
        let (_, text, _) = self.file(file)?;
        let (start, name) = word_at(text, offset as usize)?;
        let item = self.item_at(file, offset)?;
        let typed = match item.function {
            true => Some(Typed::Fn),
            false => self.type_at(file, offset).map(|t| match t.ty {
                cpl_next_ir::ty::Ty::Int => Typed::Int,
                cpl_next_ir::ty::Ty::Fn(..) => Typed::Fn,
                _ => Typed::Other,
            }),
        };
        let cost = (item.doc.as_deref())
            .and_then(|d| cpl_next_syntax::doc::Facts::parse(d).cost)
            .map(|c| cpl_diag::plain::text(&c));
        Some(Hover {
            span: cpl_diag::Span::new(start as u32, (start + name.len()) as u32),
            text: item.head.clone(),
            doc: item.doc.clone(),
            cost,
            typed,
            signature: item.function,
        })
    }

    fn typed(&self, file: &str) -> Option<(&Front, &[TypedAt])> {
        let (_, _, f) = self.file(file)?;
        let front = self.front(file)?;
        Some((front, f.types.get_or_init(|| types(front))))
    }

    /// The type of the innermost expression of `file` at byte `at`.
    pub fn type_at(&self, file: &str, at: u32) -> Option<Type<'_>> {
        let (front, typed) = self.typed(file)?;
        (typed.iter())
            .filter(|e| e.span.start <= at && at < e.span.end)
            .min_by_key(|e| e.span.end - e.span.start)
            .map(|e| Type::new(front, e))
    }

    /// Each expression of `file` inside `within`, outermost first, and its type.
    pub fn types_in(&self, file: &str, within: cpl_diag::Span) -> Vec<(cpl_diag::Span, Type<'_>)> {
        let Some((front, typed)) = self.typed(file) else {
            return Vec::new();
        };
        let mut out: Vec<_> = (typed.iter())
            .filter(|e| within.start <= e.span.start && e.span.end <= within.end)
            .map(|e| (e.span, Type::new(front, e)))
            .collect();
        out.sort_by_key(|(span, _)| (span.start, std::cmp::Reverse(span.end)));
        out
    }

    /// The names `file`'s scope gives items, that start with the identifier ending at byte
    /// `at`, by name: what typing there could complete to.
    pub fn complete(&self, file: &str, at: u32) -> Vec<(&str, &Item)> {
        let (Some((_, program, f)), Some(front)) = (self.file(file), self.front(file)) else {
            return Vec::new();
        };
        let items = f.items.get_or_init(|| items(front));
        let at = (at as usize).min(program.len());
        let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
        let start = program[..at].rfind(|c| !word(c)).map_or(0, |i| i + 1);
        let prefix = &program[start..at];
        let mut out: Vec<(&str, &Item)> = (items.iter())
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(name, item)| (name.as_str(), item))
            .collect();
        out.sort_by_key(|(name, _)| *name);
        out
    }

    /// Where the item named at byte `at` of `file` is declared.
    pub fn definition(&self, file: &str, offset: u32) -> Option<Definition> {
        let item = self.item_at(file, offset)?;
        let (id, _, _) = self.file(file)?;
        let path = match item.module.as_str() {
            modules::PROGRAM => file.to_string(),
            m => {
                match (self.work(id)).and_then(|w| w.db.get::<ModuleFile>(&(id.clone(), m.into())))
                {
                    Ok(Some(f)) => f.path().to_string(),
                    _ => modules::std_path(m),
                }
            }
        };
        Some(Definition {
            path,
            span: item.span,
        })
    }

    /// Each std module and the diagnostics checking every one of its fns gives, not only those
    /// a program reaches (suite 4).
    pub fn std_unchecked(&self) -> Vec<(String, Vec<Diag>)> {
        let h = &self.snapshot.0;
        std_unchecked(&h.std, h.judge.as_ref(), h.checks)
    }

    pub fn verdict(&self, file: &str) -> Verdict {
        let Err(ds) = self.program(file) else {
            return Verdict::Accepted;
        };
        let syntax = |d: &Diag| d.stage == Stage::Syntax && !d.is_refusal();
        match ds.iter().any(Diag::is_refusal) && !ds.iter().any(syntax) {
            true => Verdict::Refused,
            false => Verdict::Rejected,
        }
    }
}

/// The identifier `at` is in, or ends at.
/// The name at byte `at`, and where it starts.
fn word_at(text: &str, at: usize) -> Option<(usize, &str)> {
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let at = at.min(text.len());
    let start = text[..at].rfind(|c| !word(c)).map_or(0, |i| i + 1);
    let end = text[at..].find(|c| !word(c)).map_or(text.len(), |i| at + i);
    let w = &text[start..end];
    (w.chars().next()?.is_ascii_alphabetic() || w.starts_with('_')).then_some((start, w))
}

#[cfg(test)]
mod tests;
