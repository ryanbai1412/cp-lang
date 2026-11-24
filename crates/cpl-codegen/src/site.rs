//! The user line a library trap reports. A library fn is emitted once for
//! all its callers, so its asserts can't name the line that reached them;
//! a call from user code into a library fn that can fail one stores its
//! line in `cpl::site` first, as the interpreter shows the user's call
//! (R09.4.5).

use cpl_ir::{walk, Block, Expr, ExprKind, Program, Stmt};

use crate::emit::Emitter;
use crate::expr::string_literal;

/// Where each user source line starts, the line of the user statement
/// being emitted, and the library fns whose traps name it.
#[derive(Default)]
pub(crate) struct Sites {
    starts: Vec<u32>,
    line: Option<u32>,
    noted: Vec<bool>,
}

impl Sites {
    pub(crate) fn new(program: &Program) -> Sites {
        Sites {
            starts: program.user_line_starts.clone(),
            line: None,
            noted: noted_fns(program),
        }
    }
}

/// Per fn: a library fn that can fail an assert naming a library line,
/// itself or through the library fns and closures it calls.
fn noted_fns(program: &Program) -> Vec<bool> {
    let library = |id: usize| program.library_fns.get(id).copied().unwrap_or(false);
    let mut facts: Vec<(bool, Vec<usize>)> = program
        .fns
        .iter()
        .enumerate()
        .map(|(id, f)| match library(id) {
            true => {
                let mut fact = (false, Vec::new());
                scan(program, &f.body, &mut Vec::new(), &mut fact);
                fact
            }
            false => (false, Vec::new()),
        })
        .collect();
    loop {
        let grew: Vec<usize> = (0..facts.len())
            .filter(|&id| !facts[id].0 && facts[id].1.iter().any(|&c| facts[c].0))
            .collect();
        if grew.is_empty() {
            return facts.into_iter().map(|(noted, _)| noted).collect();
        }
        grew.into_iter().for_each(|id| facts[id].0 = true);
    }
}

fn scan(program: &Program, body: &Block, seen: &mut Vec<usize>, fact: &mut (bool, Vec<usize>)) {
    walk::deep_stmts(body, &mut |stmt| {
        if let Stmt::Assert { note: Some(_), .. } = stmt {
            fact.0 = true;
        }
    });
    let mut closures = Vec::new();
    walk::deep_exprs(body, &mut |e| match e.kind {
        ExprKind::Call(id, _) if program.library_fns.get(id).copied().unwrap_or(false) => {
            fact.1.push(id)
        }
        ExprKind::Closure(cid) if !seen.contains(&cid) => closures.push(cid),
        _ => {}
    });
    for cid in closures {
        if !seen.contains(&cid) {
            seen.push(cid);
            scan(program, &program.closures[cid].body, seen, fact);
        }
    }
}

impl Emitter<'_> {
    /// Notes the line of a user statement about to be emitted.
    pub(crate) fn note_site(&mut self, stmt: &Stmt) {
        if self.in_library() || self.site.starts.is_empty() {
            return;
        }
        if let Some(span) = stmt.span() {
            self.site.line = Some(self.site.starts.partition_point(|&at| at <= span.start) as u32);
        }
    }

    /// The store a call to fn `id` makes first, when user code calls a
    /// library fn whose trap would name the user's line.
    pub(crate) fn site_store(&self, id: usize) -> Option<String> {
        let line = self.site.line?;
        (!self.in_library() && self.site.noted.get(id).copied().unwrap_or(false))
            .then(|| format!("cpl::site = {line}"))
    }

    /// An assert: a library one names the user line that reached it
    /// through `cpl::site` and its own line in a note.
    pub(crate) fn assert(
        &mut self,
        out: &mut String,
        cond: &Expr,
        message: Option<&Expr>,
        text: &str,
        line: u32,
        note: Option<&str>,
    ) {
        self.need("fail");
        let cond = self.expr(cond).text;
        let library = note.is_some() && !text.is_empty();
        let report = match text {
            "" => String::new(),
            _ if library => cpl_ir::assert_head(text),
            _ => format!("{} (line {line}): ", cpl_ir::assert_head(text)),
        };
        let note = note.map_or(String::new(), |n| format!("\nnote: {n}"));
        let why = match message {
            _ if library => {
                let head = string_literal(report.as_bytes());
                let rest = match message {
                    Some(message) => {
                        let sep = string_literal(b": ");
                        format!(" + {sep} + {}", self.expr(message).text)
                    }
                    None => String::new(),
                };
                let note = string_literal(note.as_bytes());
                format!("(std::string({head}) + cpl::site_line(){rest} + {note}).c_str()")
            }
            None => string_literal(
                format!("{}{note}", report.strip_suffix(": ").unwrap_or(&report)).as_bytes(),
            ),
            Some(message) => {
                let message = self.expr(message).text;
                let report = string_literal(report.as_bytes());
                let note = string_literal(note.as_bytes());
                format!("(std::string({report}) + {message} + {note}).c_str()")
            }
        };
        self.line(out, &format!("if (!({cond})) cpl::fail({why});"));
    }

    fn in_library(&self) -> bool {
        self.fn_id
            .is_some_and(|id| self.program.library_fns.get(id).copied().unwrap_or(false))
    }
}
