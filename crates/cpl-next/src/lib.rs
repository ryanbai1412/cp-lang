//! The new pipeline: declarations, typed tree, ownership, generic IR, whole-program link,
//! canonical IR, a reference interpreter and a C++ emitter.
//!
//! It accepts a small part of the language and refuses the rest: a diagnostic with no
//! code from `cpl-diag` is a refusal, not a verdict on the program. Nothing shipped depends on this crate: it is run by its own tests
//! until it accepts every program, and the landing that switches `cpl` to it deletes
//! what it replaces.

mod consts;

use cpl_next_check as check;
use cpl_next_cpp as cpp;
use cpl_next_decls::{decls, diag, modules};
use cpl_next_interp as interp;
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_ir::{ir, ty};
use cpl_next_link as inst;
use cpl_next_lower as lower;
use cpl_next_own as own;
use diag::kind_of;
pub use diag::Kind;
pub use diag::Unbuilt;
pub use modules::{Roots, Std};

pub use interp::{Outcome, Trap};

pub use cpl_next_ir::msg::Stage;

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
    /// The library module it is from, when it has no place.
    pub module: Option<String>,
    /// The layer that found it, when `stage` reports it for another.
    pub layer: Option<&'static str>,
    /// The text without a verdict's code, or the refused feature.
    pub message: String,
    /// What to write instead, when the layer knows.
    pub help: Option<String>,
    /// Spans are of `at.file`, as `at.span` is.
    pub labels: Vec<cpl_diag::Label>,
    pub fixes: Vec<cpl_diag::Fix>,
    /// The text of `at.file`.
    src: Option<std::sync::Arc<str>>,
}

impl Diag {
    /// The code this diagnostic carries, when it is a verdict.
    pub fn code(&self) -> Option<&'static str> {
        match self.kind {
            Kind::Verdict(c) => Some(c.0),
            Kind::Refusal(_) => None,
        }
    }
    pub fn is_refusal(&self) -> bool {
        matches!(self.kind, Kind::Refusal(_))
    }

    /// The diagnostic every tool reads: an editor, lint, the judge's output. A refusal is
    /// `E0501`, the code of what is not supported.
    pub fn diagnostic(&self) -> cpl_diag::Diagnostic {
        let code = self
            .code()
            .map_or(cpl_diag::codes::UNSUPPORTED, cpl_diag::Code);
        let span = self
            .at
            .as_ref()
            .map_or(cpl_diag::Span::new(0, 0), |a| a.span);
        cpl_diag::Diagnostic {
            labels: self.labels.clone(),
            help: self.help.clone(),
            fixes: self.fixes.clone(),
            ..cpl_diag::msgs::check::NEXT_SAID
                .at(&[&self.message])
                .with_code(code)
                .error(span)
        }
    }

    /// [`cpl_diag::json::diagnostic`], with the layer, the file, the module, the refused feature
    /// and `text`, the line `Display` prints.
    pub fn json(&self) -> cpl_json::Value {
        let lines = cpl_diag::LineIndex::new(self.src.as_deref().unwrap_or(""));
        let mut o = cpl_diag::json::diagnostic(&self.diagnostic(), &lines);
        let refusal = match &self.kind {
            Kind::Refusal(u) => cpl_json::Value::String(u.variant()),
            Kind::Verdict(_) => cpl_json::Value::Null,
        };
        let text = |t: Option<String>| t.map_or(cpl_json::Value::Null, cpl_json::Value::String);
        o.insert(
            "stage".into(),
            cpl_json::Value::String(self.stage.name().into()),
        );
        o.insert(
            "file".into(),
            text(self.at.as_ref().map(|a| a.file.clone())),
        );
        o.insert("module".into(), text(self.module.clone()));
        o.insert("text".into(), text(Some(self.to_string())));
        o.insert("refusal".into(), refusal);
        cpl_json::Value::Object(o)
    }

    /// `m` placed in its file.
    fn new(stage: Stage, m: &diag::Msg, loaded: &modules::Loaded) -> Diag {
        let local = |at: &SourceId| loaded.text_of(at).span(at);
        let src = m.at.as_ref().map(|at| loaded.text_of(at).text.clone());
        let at = m.at.as_ref().map(|at| {
            let (module, span) = (loaded.text_of(at), loaded.text_of(at).span(at));
            let (line, col) = module.line_col(span.start);
            At {
                file: module.name.clone(),
                span,
                line,
                col,
            }
        });
        let kind = kind_of(m);
        let message = match &kind {
            Kind::Verdict(_) => m.text.clone(),
            Kind::Refusal(u) => u.to_string(),
        };
        Diag {
            stage,
            kind,
            at,
            module: m.module.clone(),
            layer: m.layer,
            message,
            help: m.help.clone(),
            labels: (m.labels.iter())
                .map(|(at, message)| cpl_diag::Label {
                    span: local(at),
                    message: message.clone(),
                })
                .collect(),
            fixes: (m.fixes.iter())
                .map(|f| cpl_diag::Fix {
                    title: f.title.clone(),
                    edits: f
                        .edits
                        .iter()
                        .map(|(at, t)| (local(at), t.clone()))
                        .collect(),
                })
                .collect(),
            src,
        }
    }
}

/// Whether two spans share a byte; an empty span is the byte it is at.
fn overlaps(a: cpl_diag::Span, b: cpl_diag::Span) -> bool {
    a.start < b.end.max(b.start + 1) && b.start < a.end.max(a.start + 1)
}

/// Whether two places share a byte of one item.
fn overlaps_in(a: &SourceId, b: &SourceId) -> bool {
    a.item == b.item && overlaps(a.span, b.span)
}

/// The one printer of a diagnostic: `check: program:3:5: E0301: text`, and its help on the
/// next line.
impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.stage.name())?;
        if let Some(a) = &self.at {
            write!(f, "{}:{}:{}: ", a.file, a.line, a.col)?;
        }
        for part in [self.module.as_deref(), self.layer].into_iter().flatten() {
            write!(f, "{part}: ")?;
        }
        match &self.kind {
            Kind::Verdict(c) => write!(f, "{}: ", c.0)?,
            Kind::Refusal(_) => f.write_str("slice: ")?,
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
    /// What the parser could not read; a later layer says nothing there.
    rejected: Vec<SourceId>,
    d: decls::Decls,
    t: check::Typed,
    g: lower::Generic,
    main: Vec<(ItemId, cpl_syntax::ast::Stmt)>,
    diags: Vec<Diag>,
}

/// `es`, said by `stage`, but for what lies where the parser could not read.
fn staged(
    stage: Stage,
    es: &[diag::Msg],
    loaded: &modules::Loaded,
    rejected: &[SourceId],
) -> Vec<Diag> {
    let unread = |m: &&diag::Msg| {
        (m.at.as_ref()).is_some_and(|a| rejected.iter().any(|p| overlaps_in(p, a)))
    };
    (es.iter())
        .filter(|m| stage == Stage::Syntax || !unread(m))
        .map(|e| Diag::new(stage, e, loaded))
        .collect()
}

/// Link, then ownership on the linked instances (contract 4), where the program checked clean:
/// the program, its copies settled, and ownership's diagnostics.
fn back(f: &mut Front) -> Option<(inst::Program, Vec<Diag>)> {
    if !(f.loaded.errors.is_empty() && f.t.errors.is_empty() && f.d.errors.is_empty()) {
        return None;
    }
    let names: Vec<(String, bool)> = (f.t.globals.iter())
        .map(|g| (g.name.clone(), g.mutable))
        .collect();
    let g = &mut f.g;
    let (prog, owned) = own::link(
        &f.d,
        &g.whole,
        &g.deferred,
        &mut g.funcs,
        &mut g.globals,
        &g.adds,
        &names,
    );
    let diags = staged(Stage::Own, &owned.errors, &f.loaded, &f.rejected);
    Some((prog, diags))
}

fn front(std: &Std, user_src: &str, roots: &Roots) -> Front {
    let loaded = modules::load(std, user_src, roots);
    // What the parser could not read is checked as error nodes; a later layer says nothing
    // on what the parser rejected, and every other unit is checked as if it were alone.
    let rejected: Vec<SourceId> = (loaded.poisoned.iter().cloned())
        .chain(loaded.errors.iter().filter_map(|m| m.at.clone()))
        .collect();
    let diags = |stage, es: &[diag::Msg]| staged(stage, es, &loaded, &rejected);
    let mut out = diags(Stage::Syntax, &loaded.errors);
    let (d, main) = decls::collect(std.decls(), &loaded.modules, &loaded.seen, &loaded.prelude);
    out.extend(diags(Stage::Decls, &d.errors));
    let values = consts::Evaluator::new(&d, &loaded, &main);
    let t = check::check(
        &d,
        &main,
        &loaded,
        &values,
        &[],
        judge_of(loaded.program()),
        &check::Ids::default(),
    );
    out.extend(diags(Stage::Check, &t.errors));
    let g = lower::lower(&d, &t, &loaded);
    out.extend(diags(Stage::Lower, &g.errors));
    Front {
        loaded,
        rejected,
        d,
        t,
        g,
        main,
        diags: out,
    }
}

/// The judge a program names (`// judge:`, else `generic`), as the checker reads it.
fn judge_of(program: &str) -> Option<cpl_next_ir::build::Judge> {
    let p = cpl_judge::builtin(cpl_judge::named_in(program).unwrap_or("generic"))?;
    Some(cpl_next_ir::build::Judge {
        name: p.name,
        memory_mb: p.memory_mb,
    })
}

/// Each std module and how many diagnostics it has when every one of its fns is checked, not
/// only those a program reaches: std as a program (suite 4 of plan/end-state.toml).
pub fn std_unchecked(std: &Std) -> Vec<(String, usize)> {
    (std.module_names())
        .filter(|m| *m != "std.prelude")
        .map(|m| {
            let loaded = modules::load(std, &format!("use {m}\n"), &Roots::std());
            let (d, main) =
                decls::collect(std.decls(), &loaded.modules, &loaded.seen, &loaded.prelude);
            let at = (d.module_names.iter())
                .position(|n| n == m)
                .map(decls::ModId);
            let values = consts::Evaluator::new(&d, &loaded, &main);
            let t = check::check(
                &d,
                &main,
                &loaded,
                &values,
                &at.into_iter().collect::<Vec<_>>(),
                judge_of(""),
                &check::Ids::default(),
            );
            let ds = (staged(Stage::Decls, &d.errors, &loaded, &[]).into_iter()).chain(staged(
                Stage::Check,
                &t.errors,
                &loaded,
                &[],
            ));
            let own = |d: &Diag| {
                d.at.as_ref()
                    .map(|a| a.file.as_str())
                    .or(d.module.as_deref())
                    == Some(m)
            };
            (m.to_string(), ds.filter(own).count())
        })
        .collect()
}

/// `events` adds the semantic events to the program, for both the interpreter and the C++.
pub fn compile(
    std: &Std,
    user_src: &str,
    roots: &Roots,
    events: bool,
) -> Result<Compiled, Vec<Diag>> {
    let mut f = front(std, user_src, roots);
    let Some((mut prog, own)) = back(&mut f) else {
        return Err(f.diags);
    };
    f.diags.extend(own);
    f.diags.sort_by_key(|d| d.stage);
    if !f.diags.is_empty() {
        return Err(f.diags);
    }
    let link = |es: &[diag::Msg]| -> Vec<Diag> {
        (es.iter())
            .map(|e| Diag::new(Stage::Link, e, &f.loaded))
            .collect()
    };
    if !prog.errors.is_empty() {
        return Err(link(&prog.errors));
    }
    if events {
        inst::add_events(&mut prog, &f.g.whole);
    }
    let canon = inst::canonicalize(&mut prog).map_err(|es| link(&es))?;
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
    let mut f = front(std, user_src, roots);
    if let Some((_, own)) = back(&mut f) {
        f.diags.extend(own);
        f.diags.sort_by_key(|d| d.stage);
    }
    let fns = (0..f.d.fns.len()).map(ty::FnId).filter(|id| {
        let fd = f.d.f(*id);
        // A derived impl's methods are made, not written: they are the struct's.
        let derived = matches!(fd.owner, Some(decls::Owner::Impl(i)) if f.d.impls[i].derived);
        f.d.of_program(fd.module)
            && !derived
            && fd.intrinsic.is_none()
            && !matches!(fd.body, cpl_syntax::ast::FnBody::None)
    });
    let fns = fns.map(|id| {
        let fd = f.d.f(id);
        let lowered = f.g.funcs.contains_key(&ir::FuncKey::Decl(id));
        (Some(fd.name.clone()), fd.span.clone(), lowered)
    });
    let statements = (f.main.iter()).map(|(off, s)| {
        let span = off.at(s.span);
        (None, span, f.g.funcs.contains_key(&ir::FuncKey::Main))
    });
    let units = (fns.chain(statements))
        .map(|(name, span, lowered)| {
            let local = f.loaded.text_of(&span).span(&span);
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
                untyped: f.g.untyped.iter().filter(|u| overlaps_in(u, &span)).count(),
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

    /// Every `@fast` fn runs its CP body instead of its op, which is what the fn means; the
    /// op is only a faster way to compute it, and `fast_ops_agree_with_their_bodies` holds
    /// the two together.
    pub fn without_fast(&mut self) {
        for f in (self.prog.insts.iter_mut())
            .chain(&mut self.canon)
            .chain(&mut self.opt)
        {
            f.fast = None;
        }
    }

    /// The checks optimized IR leaves out, each with the facts' reason being its site.
    pub fn proved(&self) -> &[cpl_next_ir::facts::Proved] {
        &self.proved
    }
}

#[cfg(test)]
mod tests;
