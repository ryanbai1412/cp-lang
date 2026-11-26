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
mod own;
mod records;
mod synth;

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
            // The core starts the joined source, at 0.
            let i = (loaded.modules.partition_point(|md| md.start <= span.start)).max(1);
            let module = &loaded.modules[i - 1];
            let before = &loaded.src[module.start as usize..span.start as usize];
            let line_start = before.rfind('\n').map_or(0, |i| i + 1);
            At {
                file: module.name.clone(),
                span: cpl_diag::Span::new(span.start - module.start, span.end - module.start),
                line: before.matches('\n').count() as u32 + 1,
                col: (before.len() - line_start) as u32 + 1,
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
        }
    }
}

/// The one printer of a diagnostic: `check: program:3:5: E0301: text`.
impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.stage.name())?;
        if let Some(a) = &self.at {
            write!(f, "{}:{}:{}: ", a.file, a.line, a.col)?;
        }
        if let Some(c) = self.code() {
            write!(f, "{c}: ")?;
        }
        f.write_str(&self.message)
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

/// `events` adds the semantic events to the program, for both the interpreter and the C++.
pub fn compile(user_src: &str, events: bool) -> Result<Compiled, Vec<Diag>> {
    // A method the program calls that no loaded module declares loads the std modules that
    // do, and the program is read again (R11.3.4); each round loads at least one more module.
    let mut wanted: Vec<String> = Vec::new();
    loop {
        let loaded = modules::load(STD, user_src, &wanted);
        let src = &loaded.src;
        let std_lines = src[..loaded.program_start as usize].matches('\n').count();
        let diags = |stage, es: &[diag::Msg]| -> Vec<Diag> {
            es.iter().map(|e| Diag::new(stage, e, &loaded)).collect()
        };
        // A program the grammar does not derive is rejected there; what was parsed is not checked.
        if !loaded.errors.is_empty() {
            return Err(diags(Stage::Syntax, &loaded.errors));
        }
        let (d, main) = decls::collect(&loaded.modules, &loaded.prelude);
        let mut out = diags(Stage::Decls, &d.errors);
        let mut t = check::check(&d, &main, src);
        let more = modules::declaring(&t.missing, &loaded.modules);
        if !more.is_empty() {
            wanted.extend(more);
            continue;
        }
        out.extend(diags(Stage::Check, &t.errors));
        let owned = if t.errors.is_empty() && d.errors.is_empty() {
            own::check(&d, &t)
        } else {
            own::Owned::default()
        };
        out.extend(diags(Stage::Own, &owned.errors));
        for (f, es) in &owned.deferred {
            let own = es.iter().map(|e| e.clone().prefixed(Stage::Own.name()));
            t.deferred.entry(*f).or_default().extend(own);
        }
        let g = lower::lower(&d, &t, &owned, src, std_lines as u32);
        out.extend(diags(Stage::Lower, &g.errors));
        if !out.is_empty() {
            return Err(out);
        }
        let mut prog = inst::link(&d, &t, &g.funcs, &g.globals);
        if !prog.errors.is_empty() {
            return Err(diags(Stage::Link, &prog.errors));
        }
        if events {
            inst::add_events(&mut prog, &t, &d, loaded.program_start);
        }
        let canon = inst::canonicalize(&prog).map_err(|es| diags(Stage::Link, &es))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use modules::PROGRAM;

    fn diags(src: &str) -> Vec<Diag> {
        compile(src, false).err().expect("rejected")
    }

    /// A diagnostic's span is in the user's file, past the library text joined before it.
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
