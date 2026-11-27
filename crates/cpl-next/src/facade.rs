//! Contract 9 over the new pipeline (`handbook/design/next/facade.md`): the only way a tool
//! checks, builds and runs a program, over an immutable [`Snapshot`]. Each answer is computed
//! once, on the first query that needs it, and never changes.

use cpl_next_ir::build::Judge;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::{
    compile, dumped, items, std_unchecked, types, units, Compiled, Diag, Form, Front, IrStage,
    Item, Outcome, Roots, Stage, Std, Type, TypedAt, Unit,
};

/// The only input of every query: the program, the libraries it may use, and std.
/// Cheap to clone; equal snapshots give equal answers.
#[derive(Clone)]
pub struct Snapshot(Arc<Held>);

struct Held {
    program: Arc<str>,
    roots: Arc<Roots>,
    std: Arc<Std>,
    events: bool,
    bodies: bool,
    judge: Option<Judge>,
}

impl Snapshot {
    /// `roots` and `std` are read by whoever builds the snapshot, never by a query.
    pub fn new(std: Arc<Std>, roots: Roots, program: &str) -> Snapshot {
        Snapshot(Arc::new(Held {
            program: program.into(),
            roots: Arc::new(roots),
            std,
            events: false,
            bodies: false,
            judge: None,
        }))
    }

    /// This snapshot with the program's text `text`; std and the roots are shared.
    pub fn with_program(&self, text: &str) -> Snapshot {
        self.changed(|h| h.program = text.into())
    }

    /// This snapshot with the libraries at `roots`.
    pub fn with_roots(&self, roots: Roots) -> Snapshot {
        self.changed(|h| h.roots = Arc::new(roots))
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

    pub fn program(&self) -> &str {
        &self.0.program
    }

    fn changed(&self, change: impl FnOnce(&mut Held)) -> Snapshot {
        let h = &self.0;
        let mut held = Held {
            program: h.program.clone(),
            roots: h.roots.clone(),
            std: h.std.clone(),
            events: h.events,
            bodies: h.bodies,
            judge: h.judge.clone(),
        };
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

/// The answers for one snapshot.
pub struct Session {
    snapshot: Snapshot,
    built: OnceLock<Result<Program, Vec<Diag>>>,
    units: OnceLock<(Vec<Unit>, Vec<Diag>)>,
    front: OnceLock<Front>,
    items: OnceLock<HashMap<String, Item>>,
    types: OnceLock<Vec<TypedAt>>,
}

impl Session {
    pub fn open(snapshot: Snapshot) -> Session {
        Session {
            snapshot,
            built: OnceLock::new(),
            units: OnceLock::new(),
            front: OnceLock::new(),
            items: OnceLock::new(),
            types: OnceLock::new(),
        }
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    /// The program, or every diagnostic that stops it, in layer order.
    pub fn program(&self) -> Result<&Program, &[Diag]> {
        let built = self.built.get_or_init(|| {
            let h = &self.snapshot.0;
            let mut c = compile(&h.std, &h.program, &h.roots, h.events, h.judge.as_ref())?;
            if h.bodies {
                c.without_fast();
            }
            Ok(Program(c))
        });
        built.as_ref().map_err(Vec::as_slice)
    }

    /// The program's IR at `stage`, for tracing a layer by eye (`cpl-next dump-ir`): lowering's
    /// fns the program instantiates, by name, and each later stage's in instance order, which
    /// is what a call names.
    pub fn ir(&self, stage: IrStage) -> Result<String, &[Diag]> {
        let c = &self.program()?.0;
        Ok(match stage {
            IrStage::Raw => {
                let reached: std::collections::HashSet<_> =
                    c.prog.insts.iter().map(|f| &f.key).collect();
                let mut fs: Vec<_> = (self.front().g.funcs.values())
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

    /// Every diagnostic of the program, up to `upto`.
    pub fn diagnostics(&self, upto: Layer) -> Vec<&Diag> {
        let all = self.program().err().unwrap_or_default();
        let whole = |d: &&Diag| matches!(d.stage, Stage::Own | Stage::Link);
        (all.iter())
            .filter(|d| upto == Layer::Program || !whole(d))
            .collect()
    }

    /// Each fn the program declares and each of its statements, with its diagnostics, and
    /// every diagnostic of the program: the editor's view of a program that need not compile.
    pub fn units(&self) -> (&[Unit], &[Diag]) {
        let (units, diags) = self.units.get_or_init(|| {
            let h = &self.snapshot.0;
            units(&h.std, &h.program, &h.roots, h.judge.as_ref())
        });
        (units, diags)
    }

    /// The program's checked front half, which hover, definition and types read.
    fn front(&self) -> &Front {
        let h = &self.snapshot.0;
        (self.front).get_or_init(|| crate::front(&h.std, &h.program, &h.roots, h.judge.as_ref()))
    }

    /// The item the name at byte `at` of the program names, if it names one.
    fn item_at(&self, at: u32) -> Option<&Item> {
        let name = word_at(&self.snapshot.0.program, at as usize)?;
        (self.items).get_or_init(|| items(self.front())).get(name)
    }

    /// How the item named at byte `at` is declared: its declaration's first line.
    pub fn hover(&self, at: u32) -> Option<&str> {
        self.item_at(at).map(|i| i.head.as_str())
    }

    fn typed(&self) -> &[TypedAt] {
        (self.types).get_or_init(|| types(self.front()))
    }

    /// The type of the innermost expression of the program at byte `at`.
    pub fn type_at(&self, at: u32) -> Option<Type<'_>> {
        (self.typed().iter())
            .filter(|e| e.span.start <= at && at < e.span.end)
            .min_by_key(|e| e.span.end - e.span.start)
            .map(|e| Type::new(self.front(), e))
    }

    /// Each expression of the program inside `within`, outermost first, and its type.
    pub fn types_in(&self, within: cpl_diag::Span) -> Vec<(cpl_diag::Span, Type<'_>)> {
        let mut out: Vec<_> = (self.typed().iter())
            .filter(|e| within.start <= e.span.start && e.span.end <= within.end)
            .map(|e| (e.span, Type::new(self.front(), e)))
            .collect();
        out.sort_by_key(|(span, _)| (span.start, std::cmp::Reverse(span.end)));
        out
    }

    /// The names the program's scope gives items, that start with the identifier ending at
    /// byte `at`, by name: what typing there could complete to.
    pub fn complete(&self, at: u32) -> Vec<(&str, &Item)> {
        let program = &self.snapshot.0.program;
        let at = (at as usize).min(program.len());
        let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
        let start = program[..at].rfind(|c| !word(c)).map_or(0, |i| i + 1);
        let prefix = &program[start..at];
        let items = (self.items).get_or_init(|| items(self.front()));
        let mut out: Vec<(&str, &Item)> = (items.iter())
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(name, item)| (name.as_str(), item))
            .collect();
        out.sort_by_key(|(name, _)| *name);
        out
    }

    /// Where the item named at byte `at` is declared.
    pub fn definition(&self, at: u32) -> Option<&Item> {
        self.item_at(at)
    }

    /// Each std module and how many diagnostics checking every one of its fns gives, not only
    /// those a program reaches (suite 4).
    pub fn std_unchecked(&self) -> Vec<(String, usize)> {
        let h = &self.snapshot.0;
        std_unchecked(&h.std, h.judge.as_ref())
    }

    pub fn verdict(&self) -> Verdict {
        let Err(ds) = self.program() else {
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
