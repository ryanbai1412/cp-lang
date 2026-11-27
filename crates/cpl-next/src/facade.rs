//! Contract 9 over the new pipeline (`handbook/design/next/facade.md`): the only way a tool
//! checks, builds and runs a program, over an immutable [`Snapshot`]. Each answer is computed
//! once, on the first query that needs it, and never changes.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::{
    compile, items, std_unchecked, units, Compiled, Diag, Form, Item, Outcome, Roots, Stage, Std,
    Unit,
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
    items: OnceLock<HashMap<String, Item>>,
}

impl Session {
    pub fn open(snapshot: Snapshot) -> Session {
        Session {
            snapshot,
            built: OnceLock::new(),
            units: OnceLock::new(),
            items: OnceLock::new(),
        }
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    /// The program, or every diagnostic that stops it, in layer order.
    pub fn program(&self) -> Result<&Program, &[Diag]> {
        let built = self.built.get_or_init(|| {
            let h = &self.snapshot.0;
            let mut c = compile(&h.std, &h.program, &h.roots, h.events)?;
            if h.bodies {
                c.without_fast();
            }
            Ok(Program(c))
        });
        built.as_ref().map_err(Vec::as_slice)
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
            units(&h.std, &h.program, &h.roots)
        });
        (units, diags)
    }

    /// The item the name at byte `at` of the program names, if it names one.
    fn item_at(&self, at: u32) -> Option<&Item> {
        let h = &self.snapshot.0;
        let name = word_at(&h.program, at as usize)?;
        let items = (self.items).get_or_init(|| items(&h.std, &h.program, &h.roots));
        items.get(name)
    }

    /// How the item named at byte `at` is declared: its declaration's first line.
    pub fn hover(&self, at: u32) -> Option<&str> {
        self.item_at(at).map(|i| i.head.as_str())
    }

    /// Where the item named at byte `at` is declared.
    pub fn definition(&self, at: u32) -> Option<&Item> {
        self.item_at(at)
    }

    /// Each std module and how many diagnostics checking every one of its fns gives, not only
    /// those a program reaches (suite 4).
    pub fn std_unchecked(&self) -> Vec<(String, usize)> {
        std_unchecked(&self.snapshot.0.std)
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
