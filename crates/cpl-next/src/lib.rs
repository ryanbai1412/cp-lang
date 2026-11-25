//! The new pipeline: declarations, typed tree, ownership, generic IR, whole-program link,
//! canonical IR, a reference interpreter and a C++ emitter.
//!
//! It accepts a small part of the language and refuses the rest: a diagnostic with no
//! code from `cpl-diag` is a refusal, not a verdict on the program. Nothing shipped depends on this crate: it is run by its own tests
//! until it accepts every program, and the landing that switches `cpl` to it deletes
//! what it replaces.

mod check;
mod cpp;
mod decls;
mod inst;
mod interp;
mod ir;
mod lower;
mod own;
mod ty;

pub use interp::{Outcome, Trap};

const STD: &str = include_str!("../std.cp");

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

#[derive(Clone, Debug)]
pub struct Diag {
    pub stage: Stage,
    /// `line N: ...` with N counted in the user's file.
    pub text: String,
}

impl Diag {
    /// The code this diagnostic carries, when it is one of the language's.
    pub fn code(&self) -> Option<&'static str> {
        let after_line = self
            .text
            .strip_prefix("line ")
            .and_then(|r| r.split_once(": "))
            .map_or(self.text.as_str(), |(_, r)| r);
        let code = after_line.split(':').next()?;
        cpl_diag::lookup(code).map(|info| info.code.0)
    }
    /// Not a verdict on the program: the pipeline does not implement something it uses, or
    /// reports something it has no code for. Only a diagnostic with a code is a verdict.
    pub fn is_refusal(&self) -> bool {
        self.code().is_none()
    }
}

/// Which form of the IR the reference interpreter runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    /// Instances as linked.
    Raw,
    /// After function values are made explicit.
    Canonical,
}

pub struct Compiled {
    prog: inst::Program,
    canon: Vec<ir::Func<inst::CTy>>,
}

/// Lines are reported in the user's file: std is placed in front of it.
fn user_line(e: &str) -> String {
    let std_lines = STD.matches('\n').count() + 1;
    match e.strip_prefix("line ").and_then(|r| r.split_once(':')) {
        Some((n, rest)) => match n.parse::<usize>() {
            Ok(n) if n > std_lines => format!("line {}:{rest}", n - std_lines),
            _ => format!("std {e}"),
        },
        None => e.to_string(),
    }
}

/// `events` adds the semantic events to the program, for both the interpreter and the C++.
pub fn compile(user_src: &str, events: bool) -> Result<Compiled, Vec<Diag>> {
    let src = format!("{STD}\n{user_src}");
    let mut out: Vec<Diag> = Vec::new();
    let mut add = |stage, text: &str| {
        out.push(Diag {
            stage,
            text: user_line(text),
        })
    };
    let p = cpl_syntax::parse_file(&src);
    for d in &p.diagnostics {
        add(Stage::Syntax, &d.message);
    }
    let items: Vec<(u32, Vec<cpl_syntax::ast::Stmt>)> = p
        .items
        .iter()
        .map(|(o, it)| (*o, it.stmts.clone()))
        .collect();
    let (d, main) = decls::collect(&items);
    for e in &d.errors {
        add(Stage::Decls, e);
    }
    let t = check::check(&d, &main, &src);
    for e in &t.errors {
        add(Stage::Check, e);
    }
    if t.errors.is_empty() && d.errors.is_empty() {
        for e in &own::check(&d, &t, &src) {
            add(Stage::Own, e);
        }
    }
    let g = lower::lower(&d, &t, &src, STD.matches('\n').count() as u32 + 1);
    for e in &g.errors {
        add(Stage::Lower, e);
    }
    if !out.is_empty() {
        return Err(out);
    }
    let mut prog = inst::link(&d, &t, &g.funcs, &g.globals);
    if !prog.errors.is_empty() {
        return Err(prog
            .errors
            .iter()
            .map(|e| Diag {
                stage: Stage::Link,
                text: user_line(e),
            })
            .collect());
    }
    if events {
        inst::add_events(&mut prog, &t, &d, STD.len() as u32);
    }
    let canon = inst::canonicalize(&prog).map_err(|es| {
        es.iter()
            .map(|e| Diag {
                stage: Stage::Link,
                text: user_line(e),
            })
            .collect::<Vec<_>>()
    })?;
    Ok(Compiled { prog, canon })
}

impl Compiled {
    /// Runs on the calling thread. The interpreter recurses once per call of the program, so
    /// a caller that wants the spec's 1 GB of program stack gives the thread more than that.
    pub fn run(&self, form: Form, input: &str) -> Outcome {
        let funcs = match form {
            Form::Raw => &self.prog.insts,
            Form::Canonical => &self.canon,
        };
        interp::run(&self.prog, funcs, input)
    }

    pub fn cpp(&self) -> String {
        cpp::emit(&self.prog, &self.canon)
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
