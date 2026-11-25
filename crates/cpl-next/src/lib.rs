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
mod modules;
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

/// Lines are reported in the user's file: the libraries are placed in front of it.
fn user_line(e: &str, std_lines: usize) -> String {
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
    // A method the program calls that no loaded module declares loads the std modules that
    // do, and the program is read again (R11.3.4); each round loads at least one more module.
    let mut wanted: Vec<String> = Vec::new();
    loop {
        let loaded = modules::load(STD, user_src, &wanted);
        let src = loaded.src;
        let std_lines = src[..loaded.program_start as usize].matches('\n').count();
        // A program the grammar does not derive is rejected there; what was parsed is not checked.
        if !loaded.errors.is_empty() {
            return Err((loaded.errors.iter())
                .map(|e| Diag {
                    stage: Stage::Syntax,
                    text: user_line(e, std_lines),
                })
                .collect());
        }
        let mut out: Vec<Diag> = Vec::new();
        let mut add = |stage, text: &str| {
            out.push(Diag {
                stage,
                text: user_line(text, std_lines),
            })
        };
        let (d, main) = decls::collect(&loaded.modules, &loaded.prelude);
        for e in &d.errors {
            add(Stage::Decls, e);
        }
        let t = check::check(&d, &main, &src);
        let more = modules::declaring(&t.missing, &loaded.modules);
        if !more.is_empty() {
            wanted.extend(more);
            continue;
        }
        for e in &t.errors {
            add(Stage::Check, e);
        }
        let owned = if t.errors.is_empty() && d.errors.is_empty() {
            own::check(&d, &t, &src)
        } else {
            own::Owned::default()
        };
        for e in &owned.errors {
            add(Stage::Own, e);
        }
        let g = lower::lower(&d, &t, &owned, &src, std_lines as u32);
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
                    text: user_line(e, std_lines),
                })
                .collect());
        }
        if events {
            inst::add_events(&mut prog, &t, &d, loaded.program_start);
        }
        let canon = inst::canonicalize(&prog).map_err(|es| {
            es.iter()
                .map(|e| Diag {
                    stage: Stage::Link,
                    text: user_line(e, std_lines),
                })
                .collect::<Vec<_>>()
        })?;
        return Ok(Compiled { prog, canon });
    }
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
