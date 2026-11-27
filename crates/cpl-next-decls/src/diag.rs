//! A diagnostic as a layer reports it: a span of the offset space every loaded module is placed in (contract 1) and what
//! it says, and which diagnostics are verdicts on the program and which are refusals. What this
//! file calls a verdict is what `next_wrong` counts against a case that expects acceptance.

/// What a diagnostic says about the program. The set is closed: every diagnostic is one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Kind {
    /// The program breaks the rule the code names.
    Verdict(cpl_diag::Code),
    /// The program uses a feature the spec has and this pipeline has not built yet.
    /// It says nothing about the program.
    Refusal(Unbuilt),
}

/// A refusal is its feature; a verdict's text is `<code>: <text>`, and text without a code is
/// the pipeline's bug: an internal error.
pub fn kind_of(m: &Msg) -> Kind {
    if let Some(u) = &m.refusal {
        return Kind::Refusal(u.clone());
    }
    let code = m
        .text
        .split_once(": ")
        .and_then(|(c, _)| cpl_diag::lookup(c));
    Kind::Verdict(code.map_or(cpl_diag::codes::INTERNAL, |i| i.code))
}

pub use cpl_next_ir::msg::Msg;
pub use cpl_next_ir::unbuilt::Unbuilt;
