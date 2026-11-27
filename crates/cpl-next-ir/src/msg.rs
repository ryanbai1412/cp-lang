//! A diagnostic as a layer reports it: a span of the joined source (contract 1) and its text.
use cpl_diag::Span;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Msg {
    /// `None`: the layer does not know where yet.
    pub at: Option<Span>,
    /// As `kind_of` reads it.
    pub text: String,
    /// What to write instead, when the layer knows.
    pub help: Option<String>,
}

impl Msg {
    pub fn at(at: Span, text: impl Into<String>) -> Msg {
        Msg {
            at: Some(at),
            text: text.into(),
            help: None,
        }
    }
    /// A library's message with no place yet names the module it is from.
    pub fn in_module(self, module: &str) -> Msg {
        match self.at {
            Some(_) => self,
            None => Msg {
                text: format!("{module}: {}", self.text),
                ..self
            },
        }
    }
    pub fn prefixed(self, prefix: &str) -> Msg {
        Msg {
            text: format!("{prefix}: {}", self.text),
            ..self
        }
    }
}

impl From<String> for Msg {
    fn from(text: String) -> Msg {
        Msg {
            at: None,
            text,
            help: None,
        }
    }
}

impl From<&str> for Msg {
    fn from(text: &str) -> Msg {
        Msg::from(text.to_string())
    }
}
