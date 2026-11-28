//! Contract 9 over the new pipeline (`handbook/design/next/facade.md`): the only way a tool
//! checks, builds and runs a program, over an immutable [`Snapshot`]. Each answer is computed
//! once, on the first query that needs it, and never changes.

pub use cpl_contract::{FileId, Text, World};
use cpl_next_ir::build::Judge;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};

pub use cpl_next_ir::build::Checks;

use crate::{
    compile, dumped, items, std_unchecked, types, units, Compiled, Diag, Form, Front, IrStage,
    Item, Outcome, Roots, Stage, Std, Type, TypedAt, Unit,
};

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

    /// Where the libraries `file` uses are: the `cp.toml` at or above it on disk, or none
    /// past std in a closed world.
    fn roots(&self, file: &FileId) -> Roots {
        match self.0.world {
            World::Disk => {
                let dir = Path::new(file.path())
                    .parent()
                    .filter(|d| !d.as_os_str().is_empty());
                Roots::from(dir.unwrap_or(Path::new(".")))
            }
            World::Closed => Roots::std(),
        }
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
pub struct Program(Compiled);

impl Program {
    /// Runs on the engine, on the calling thread.
    pub fn run(&self, input: &str) -> Outcome {
        self.0.run(input)
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
    built: OnceLock<Result<Program, Vec<Diag>>>,
    units: OnceLock<(Vec<Unit>, Vec<Diag>)>,
    front: OnceLock<Front>,
    items: OnceLock<HashMap<String, Item>>,
    types: OnceLock<Vec<TypedAt>>,
}

/// The answers for one snapshot, per file. A file the snapshot does not hold is no program and
/// says nothing.
pub struct Session {
    snapshot: Snapshot,
    /// In the order of the snapshot's files.
    files: Vec<PerFile>,
}

impl Session {
    pub fn open(snapshot: Snapshot) -> Session {
        let files = snapshot
            .0
            .files
            .iter()
            .map(|_| PerFile::default())
            .collect();
        Session { snapshot, files }
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
    pub fn program(&self, file: &str) -> Result<&Program, &[Diag]> {
        let Some((id, text, f)) = self.file(file) else {
            return Err(&[]);
        };
        let built = f.built.get_or_init(|| {
            let h = &self.snapshot.0;
            let roots = self.snapshot.roots(id);
            let mut c = compile(
                &h.std,
                text,
                &roots,
                h.events,
                h.judge.as_ref(),
                Checks::CHECKED,
            )?;
            if h.bodies {
                c.without_fast();
            }
            Ok(Program(c))
        });
        built.as_ref().map_err(Vec::as_slice)
    }

    /// `file`'s IR at `stage`, for tracing a layer by eye (`cpl-next dump-ir`): lowering's
    /// fns the program instantiates, by name, and each later stage's in instance order, which
    /// is what a call names.
    pub fn ir(&self, file: &str, stage: IrStage) -> Result<String, &[Diag]> {
        let c = &self.program(file)?.0;
        let Some(front) = self.front(file) else {
            return Err(&[]);
        };
        Ok(match stage {
            IrStage::Raw => {
                let reached: std::collections::HashSet<_> =
                    c.prog.insts.iter().map(|f| &f.key).collect();
                let mut fs: Vec<_> = (front.g.funcs.values())
                    .filter(|f| reached.contains(&f.key))
                    .collect();
                fs.sort_by_cached_key(|f| (f.name.clone(), format!("{:?}", f.key)));
                dumped(fs)
            }
            IrStage::Owned => dumped(&c.prog.insts),
            IrStage::Canonical => dumped(&c.canon),
            IrStage::Optimized => dumped(&c.opt),
        })
    }

    /// Every diagnostic of `file`, up to `upto`.
    pub fn diagnostics(&self, file: &str, upto: Layer) -> Vec<&Diag> {
        let all = self.program(file).err().unwrap_or_default();
        let whole = |d: &&Diag| matches!(d.stage, Stage::Own | Stage::Link);
        (all.iter())
            .filter(|d| upto == Layer::Program || !whole(d))
            .collect()
    }

    /// Each fn `file` declares and each of its statements, with its diagnostics, and every
    /// diagnostic of it: the editor's view of a program that need not compile.
    pub fn units(&self, file: &str) -> (&[Unit], &[Diag]) {
        let Some((id, text, f)) = self.file(file) else {
            return (&[], &[]);
        };
        let (units, diags) = f.units.get_or_init(|| {
            let h = &self.snapshot.0;
            let roots = self.snapshot.roots(id);
            units(&h.std, text, &roots, h.judge.as_ref(), Checks::CHECKED)
        });
        (units, diags)
    }

    /// `file`'s checked front half, which hover, definition and types read.
    fn front(&self, file: &str) -> Option<&Front> {
        let (id, text, f) = self.file(file)?;
        Some(f.front.get_or_init(|| {
            let h = &self.snapshot.0;
            let roots = self.snapshot.roots(id);
            crate::front(&h.std, text, &roots, h.judge.as_ref(), Checks::CHECKED)
        }))
    }

    /// The item the name at byte `at` of `file` names, if it names one.
    fn item_at(&self, file: &str, at: u32) -> Option<&Item> {
        let (_, text, f) = self.file(file)?;
        let name = word_at(text, at as usize)?;
        let front = self.front(file)?;
        f.items.get_or_init(|| items(front)).get(name)
    }

    /// How the item named at byte `at` of `file` is declared: its declaration's first line.
    pub fn hover(&self, file: &str, at: u32) -> Option<&str> {
        self.item_at(file, at).map(|i| i.head.as_str())
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
    pub fn definition(&self, file: &str, at: u32) -> Option<&Item> {
        self.item_at(file, at)
    }

    /// Each std module and the diagnostics checking every one of its fns gives, not only those
    /// a program reaches (suite 4).
    pub fn std_unchecked(&self) -> Vec<(String, Vec<Diag>)> {
        let h = &self.snapshot.0;
        std_unchecked(&h.std, h.judge.as_ref())
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
fn word_at(text: &str, at: usize) -> Option<&str> {
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let at = at.min(text.len());
    let start = text[..at].rfind(|c| !word(c)).map_or(0, |i| i + 1);
    let end = text[at..].find(|c| !word(c)).map_or(text.len(), |i| at + i);
    let w = &text[start..end];
    (w.chars().next()?.is_ascii_alphabetic() || w.starts_with('_')).then_some(w)
}
