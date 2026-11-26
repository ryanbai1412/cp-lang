//! A diagnostic as a layer reports it: a span of the joined source (contract 1) and its text.
use cpl_diag::Span;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Msg {
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
