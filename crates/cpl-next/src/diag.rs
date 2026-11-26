//! A diagnostic as a layer reports it: a span of the joined source (contract 1) and its
//! text, and which diagnostics are verdicts on the program and which are refusals. What this
//! file calls a verdict is what `next_wrong` counts against a case that expects acceptance.
use cpl_diag::Span;

/// What a diagnostic says about the program. The set is closed: every diagnostic is one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// The program breaks the rule the code names.
    Verdict(cpl_diag::Code),
    /// The program uses a feature the spec has and this pipeline has not built yet (`slice:`).
    /// It says nothing about the program.
    Refusal,
}

/// A stage's message is `[<module>: ][own: ]<code or slice or internal>: <text>`.
/// Anything else is a message without a code, which is the pipeline's bug: an internal error.
pub(crate) fn kind_of(e: &str) -> Kind {
    for part in e.split_inclusive(": ") {
        if part == "slice: " {
            return Kind::Refusal;
        }
        let part = part.strip_suffix(": ").unwrap_or(part);
        if let Some(info) = cpl_diag::lookup(part) {
            return Kind::Verdict(info.code);
        }
        if part.contains(' ') {
            break;
        }
    }
    Kind::Verdict(cpl_diag::codes::INTERNAL)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Msg {
    /// `None`: the layer does not know where yet.
    pub at: Option<Span>,
    /// As `kind_of` reads it.
    pub text: String,
}

impl Msg {
    pub fn at(at: Span, text: impl Into<String>) -> Msg {
        Msg {
            at: Some(at),
            text: text.into(),
        }
    }
    /// A library's message with no place yet names the module it is from.
    pub fn in_module(self, module: &str) -> Msg {
        match self.at {
            Some(_) => self,
            None => Msg {
                at: None,
                text: format!("{module}: {}", self.text),
            },
        }
    }
    pub fn prefixed(self, prefix: &str) -> Msg {
        Msg {
            at: self.at,
            text: format!("{prefix}: {}", self.text),
        }
    }
}

impl From<String> for Msg {
    fn from(text: String) -> Msg {
        Msg { at: None, text }
    }
}

impl From<&str> for Msg {
    fn from(text: &str) -> Msg {
        Msg::from(text.to_string())
    }
}
