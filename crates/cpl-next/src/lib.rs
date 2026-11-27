//! The new pipeline: declarations, typed tree, ownership, generic IR, whole-program link,
//! canonical IR, a reference interpreter and a C++ emitter.
//!
//! It accepts a small part of the language and refuses the rest: a diagnostic with no
//! code from `cpl-diag` is a refusal, not a verdict on the program. Nothing shipped depends on this crate: it is run by its own tests
//! until it accepts every program, and the landing that switches `cpl` to it deletes
//! what it replaces.

use cpl_next_ir::{ir, ops, ty};
mod check;
mod cpp;
mod cycles;
mod decls;
mod diag;
use diag::kind_of;
pub use diag::Kind;
mod inst;
mod interp;
mod lower;
mod modules;
pub use modules::{Roots, Std};
mod own;
mod records;
mod synth;

pub use interp::{Outcome, Trap};

/// Which layer reported a diagnostic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Syntax,
    Decls,
    Check,
    Own,
    Lower,
    Link,
}

impl Stage {
    pub fn name(self) -> &'static str {
        match self {
            Stage::Syntax => "syntax",
            Stage::Decls => "decls",
            Stage::Check => "check",
            Stage::Own => "own",
            Stage::Lower => "lower",
            Stage::Link => "link",
        }
    }
}

/// Where a diagnostic is: a file of the program and a span of its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct At {
    /// `program` for the user's file, else the library module (`std.graph`).
    pub file: String,
    /// Byte offsets into that file's text.
    pub span: cpl_diag::Span,
    /// Of the span's start, from 1; the column counts bytes.
    pub line: u32,
    pub col: u32,
}

#[derive(Clone, Debug)]
pub struct Diag {
    pub stage: Stage,
    pub kind: Kind,
    /// `None`: the layer that reported it does not know where yet.
    pub at: Option<At>,
    /// The text without a verdict's code.
    pub message: String,
    /// What to write instead, when the layer knows.
    pub help: Option<String>,
}

impl Diag {
    /// The code this diagnostic carries, when it is a verdict.
    pub fn code(&self) -> Option<&'static str> {
        match self.kind {
            Kind::Verdict(c) => Some(c.0),
            Kind::Refusal => None,
        }
    }
    pub fn is_refusal(&self) -> bool {
        self.kind == Kind::Refusal
    }

    /// `m` placed in its file.
    fn new(stage: Stage, m: &diag::Msg, loaded: &modules::Loaded) -> Diag {
        let at = m.at.map(|span| {
            let module = loaded.text_at(span.start);
            let (line, col) = module.line_col(span.start);
            At {
                file: module.name.clone(),
                span: cpl_diag::Span::new(span.start - module.start, span.end - module.start),
                line,
                col,
            }
        });
        let kind = kind_of(&m.text);
        let message = match kind {
            Kind::Verdict(c) => m.text.replacen(&format!("{}: ", c.0), "", 1),
            Kind::Refusal => m.text.clone(),
        };
        Diag {
            stage,
            kind,
            at,
            message,
            help: m.help.clone(),
        }
    }
}

/// Whether two spans share a byte; an empty span is the byte it is at.
fn overlaps(a: cpl_diag::Span, b: cpl_diag::Span) -> bool {
    a.start < b.end.max(b.start + 1) && b.start < a.end.max(a.start + 1)
}

/// The one printer of a diagnostic: `check: program:3:5: E0301: text`, and its help on the
/// next line.
impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.stage.name())?;
        if let Some(a) = &self.at {
            write!(f, "{}:{}:{}: ", a.file, a.line, a.col)?;
        }
        if let Some(c) = self.code() {
            write!(f, "{c}: ")?;
        }
        f.write_str(&self.message)?;
        match &self.help {
            Some(h) => write!(f, "\n  = help: {h}"),
            None => Ok(()),
        }
    }
}

/// Which form of the IR the reference interpreter runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    /// Instances as linked.
    Raw,
    /// After function values are made explicit.
    Canonical,
    /// Canonical IR on the fast engine.
    Engine,
}

pub struct Compiled {
    prog: inst::Program,
    canon: Vec<ir::Func<inst::CTy>>,
    /// What the fast engine and the C++ run: canonical IR less the checks the facts prove.
    opt: Vec<ir::Func<inst::CTy>>,
    proved: Vec<cpl_next_ir::facts::Proved>,
}

/// Every layer through lowering, each run whatever an earlier one found (contracts 1-3).
struct Front {
    loaded: modules::Loaded,
    d: decls::Decls,
    t: check::Typed,
    g: lower::Generic,
    main: Vec<(u32, cpl_syntax::ast::Stmt)>,
    diags: Vec<Diag>,
}

fn front(std: &Std, user_src: &str, roots: &Roots) -> Front {
    // A method the program calls that no loaded module declares loads the std modules that
    // do, and the program is read again (R11.3.4); each round loads at least one more module.
    let mut wanted: Vec<String> = Vec::new();
    loop {
        let loaded = modules::load(std, user_src, roots, &wanted);
        // What the parser could not read is checked as error nodes; a later layer says nothing
        // on what the parser rejected, and every other unit is checked as if it were alone.
        let rejected: Vec<cpl_diag::Span> = (loaded.poisoned.iter().copied())
            .chain(loaded.errors.iter().filter_map(|m| m.at))
            .collect();
        let unread =
            |m: &&diag::Msg| (m.at).is_some_and(|a| rejected.iter().any(|p| overlaps(*p, a)));
        let diags = |stage, es: &[diag::Msg]| -> Vec<Diag> {
            (es.iter())
                .filter(|m| stage == Stage::Syntax || !unread(m))
                .map(|e| Diag::new(stage, e, &loaded))
                .collect()
        };
        let mut out = diags(Stage::Syntax, &loaded.errors);
        let (d, main) = decls::collect(&loaded.modules, &loaded.prelude);
        out.extend(diags(Stage::Decls, &d.errors));
        let mut t = check::check(&d, &main, &loaded);
        let more = std.declaring(&t.missing, &loaded.modules);
        if !more.is_empty() {
            wanted.extend(more);
            continue;
        }
        out.extend(diags(Stage::Check, &t.errors));
        let owned = if loaded.errors.is_empty() && t.errors.is_empty() && d.errors.is_empty() {
            own::check(&d, &t)
        } else {
            own::Owned::default()
        };
        out.extend(diags(Stage::Own, &owned.errors));
        for (f, es) in &owned.deferred {
            let own = es.iter().map(|e| e.clone().prefixed(Stage::Own.name()));
            t.deferred.entry(*f).or_default().extend(own);
        }
        let g = lower::lower(&d, &t, &owned, &loaded);
        out.extend(diags(Stage::Lower, &g.errors));
        return Front {
            loaded,
            d,
            t,
            g,
            main,
            diags: out,
        };
    }
}

/// `events` adds the semantic events to the program, for both the interpreter and the C++.
pub fn compile(
    std: &Std,
    user_src: &str,
    roots: &Roots,
    events: bool,
) -> Result<Compiled, Vec<Diag>> {
    let Front {
        loaded,
        d,
        t,
        g,
        diags,
        ..
    } = front(std, user_src, roots);
    if !diags.is_empty() {
        return Err(diags);
    }
    let link = |es: &[diag::Msg]| -> Vec<Diag> {
        (es.iter())
            .map(|e| Diag::new(Stage::Link, e, &loaded))
            .collect()
    };
    let lines = (g.sites.iter())
        .map(|s| loaded.program_line(s.start))
        .collect();
    let mut prog = inst::link(&d, &t, &g.funcs, &g.globals, lines);
    if !prog.errors.is_empty() {
        return Err(link(&prog.errors));
    }
    if events {
        inst::add_events(&mut prog, &t, &d, loaded.program_start);
    }
    let canon = inst::canonicalize(&prog).map_err(|es| link(&es))?;
    let (opt, proved) = cpl_next_ir::opt::optimize(&canon, &prog.globals, prog.main);
    Ok(Compiled {
        prog,
        canon,
        opt,
        proved,
    })
}

/// One unit of the program (contract 3): a fn it declares, or one of its top-level statements.
#[derive(Debug)]
pub struct Unit {
    /// The fn's name; `None` for a statement.
    pub name: Option<String>,
    /// Where it is in the program's text.
    pub span: cpl_diag::Span,
    /// The diagnostics placed in it.
    pub diags: Vec<Diag>,
    /// How many of its expressions the checker could not type (error nodes).
    pub untyped: usize,
    /// Whether lowering made its code: a fn's own function, a statement's part of `main`.
    pub lowered: bool,
}

/// What `compile` checks and lowers, unit by unit, and every diagnostic: the editor's view of a
/// program that need not compile.
pub fn units(std: &Std, user_src: &str, roots: &Roots) -> (Vec<Unit>, Vec<Diag>) {
    let f = front(std, user_src, roots);
    let start = f.loaded.program_start;
    let fns = (0..f.d.fns.len()).map(ty::FnId).filter(|id| {
        let fd = f.d.f(*id);
        // A derived impl's methods are made, not written: they are the struct's.
        let derived = matches!(fd.owner, Some(decls::Owner::Impl(i)) if f.d.impls[i].derived);
        fd.module == f.d.program
            && !derived
            && fd.intrinsic.is_none()
            && !matches!(fd.body, cpl_syntax::ast::FnBody::None)
    });
    let fns = fns.map(|id| {
        let fd = f.d.f(id);
        let lowered = f.g.funcs.contains_key(&ir::FuncKey::Decl(id));
        (Some(fd.name.clone()), fd.span, lowered)
    });
    let statements = (f.main.iter()).map(|(off, s)| {
        let span = cpl_diag::Span::new(off + s.span.start, off + s.span.end);
        (None, span, f.g.funcs.contains_key(&ir::FuncKey::Main))
    });
    let units = (fns.chain(statements))
        .map(|(name, span, lowered)| {
            let local = cpl_diag::Span::new(span.start - start, span.end - start);
            let diags = (f.diags.iter())
                .filter(|d| {
                    (d.at.as_ref())
                        .is_some_and(|a| a.file == modules::PROGRAM && overlaps(a.span, local))
                })
                .cloned()
                .collect();
            Unit {
                name,
                span: local,
                diags,
                untyped: f.g.untyped.iter().filter(|u| overlaps(**u, span)).count(),
                lowered,
            }
        })
        .collect();
    (units, f.diags)
}

impl Compiled {
    /// Runs on the calling thread. The interpreter recurses once per call of the program, so
    /// a caller that wants the spec's 1 GB of program stack gives the thread more than that.
    pub fn run(&self, form: Form, input: &str) -> Outcome {
        match form {
            Form::Raw => interp::run(&self.prog, &self.prog.insts, input),
            Form::Canonical => interp::run(&self.prog, &self.canon, input),
            Form::Engine => cpl_next_engine::run(&self.prog, &self.opt, input),
        }
    }

    pub fn cpp(&self) -> String {
        cpp::emit(&self.prog, &self.opt)
    }

    /// The checks optimized IR leaves out, each with the facts' reason being its site.
    pub fn proved(&self) -> &[cpl_next_ir::facts::Proved] {
        &self.proved
    }
}

/// The name of a syntax node's kind, for saying which one is not implemented.
pub(crate) fn kind_name(kind: &impl std::fmt::Debug) -> String {
    let text = format!("{kind:?}");
    let end = text
        .find(|c: char| !c.is_alphanumeric())
        .unwrap_or(text.len());
    text[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use modules::PROGRAM;

    fn diags(src: &str) -> Vec<Diag> {
        compile(&Std::read(), src, &Roots::std(), false)
            .err()
            .expect("rejected")
    }

    /// What compiling `src` gives: its diagnostics, or its C++ and what its run prints.
    fn outcome(std: &Std, src: &str) -> String {
        match compile(std, src, &Roots::std(), true) {
            Ok(c) => {
                let o = c.run(Form::Raw, "");
                let events: String = o.events.iter().map(|e| format!("{e}\n")).collect();
                format!("{}\n{}{events}{}", c.cpp(), o.output, o.trap.is_some())
            }
            Err(ds) => ds.iter().map(|d| format!("{d}\n")).collect(),
        }
    }

    /// Std read once is a value no program changes: programs compiled one after another
    /// against it each get what they get against a std of their own.
    #[test]
    fn programs_against_one_std_are_as_if_each_were_alone() {
        let programs = [
            include_str!("../tests/programs/n_bound_02_for_walks_through_the_bound.cp"),
            include_str!("../tests/programs/a_check_08_wrong_arg_count.cp"),
            include_str!("../tests/programs/n_assoc_type.cp"),
            "let xs = [3, 1, 2]\nprint(xs.sorted())\n",
            "let a = 1\nlet = 2\n",
        ];
        let std = Std::read();
        let shared: Vec<String> = programs.iter().map(|p| outcome(&std, p)).collect();
        for (p, got) in programs.iter().zip(&shared) {
            assert_eq!(got, &outcome(&Std::read(), p), "{p}");
        }
    }

    /// The checks optimized IR leaves out in the program's own code, by kind and line.
    fn proved(src: &str) -> Vec<(cpl_next_ir::ir::CheckKind, u32)> {
        let c = compile(&Std::read(), src, &Roots::std(), false).expect("compiles");
        let main = c.prog.main;
        (c.proved().iter())
            .filter(|p| p.func == main)
            .map(|p| (p.kind, c.prog.lines[p.site as usize]))
            .collect()
    }

    /// A check is left out only where a fact proves it: `i` from `0..n` is a position of
    /// `[0..n]` and `n - i` is in `1..=n`, but past the end of either it stays.
    #[test]
    fn facts_leave_out_only_the_checks_they_prove() {
        use cpl_next_ir::ir::CheckKind::{Domain, RangeStore};
        let head = "let n: 1..=10 = read()\nvar walk: List<1..=n> = []\nvar nxt: [0..n]0..=n = 0\n";
        let at = |body: &str| proved(&format!("{head}{body}\n"));
        assert_eq!(
            at("for i in 0..n: walk.push(n - i)"),
            [
                (RangeStore, 3),
                (RangeStore, 3),
                (RangeStore, 4),
                (RangeStore, 4)
            ]
        );
        assert_eq!(
            at("for i in 0..=n: walk.push(n - i)"),
            [(RangeStore, 3), (RangeStore, 3), (RangeStore, 4)]
        );
        assert_eq!(
            at("for i in 0..n: nxt[i] = i + 1"),
            [
                (RangeStore, 3),
                (RangeStore, 3),
                (Domain, 4),
                (Domain, 4),
                (Domain, 4),
                (RangeStore, 4),
                (RangeStore, 4)
            ]
        );
        assert_eq!(
            at("for i in 0..n: nxt[i + 1] = i"),
            [
                (RangeStore, 3),
                (RangeStore, 3),
                (Domain, 4),
                (Domain, 4),
                (RangeStore, 4),
                (RangeStore, 4)
            ]
        );
    }

    /// A diagnostic's span is in the user's file, past the modules placed before it.
    #[test]
    fn diagnostic_is_placed_in_the_program() {
        let src = "let a = 1\nlet b = 2\nprint(a +  zz)\n";
        let d = &diags(src)[0];
        let at = d.at.as_ref().expect("placed");
        assert_eq!((at.file.as_str(), at.line, at.col), (PROGRAM, 3, 12));
        assert_eq!(&src[at.span.start as usize..at.span.end as usize], "zz");
        assert_eq!(
            d.to_string(),
            "check: program:3:12: E0201: unknown name `zz`"
        );
    }

    #[test]
    fn syntax_error_is_placed_in_the_program() {
        let d = &diags("let a = 1\nlet = 2\n")[0];
        let at = d.at.as_ref().expect("placed");
        assert_eq!(
            (d.stage, at.file.as_str(), at.line),
            (Stage::Syntax, PROGRAM, 2)
        );
        assert!(d.code().is_some(), "{d}");
    }
}
