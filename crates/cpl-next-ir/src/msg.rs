//! A diagnostic as a layer reports it: a span of the joined source (contract 1) and what it
//! says, a refusal as its [`Unbuilt`] feature.
use crate::unbuilt::Unbuilt;
use cpl_diag::{Code, Span};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Msg {
    /// `None`: the layer does not know where yet.
    pub at: Option<Span>,
    /// The library module it is from, when it has no place yet.
    pub module: Option<String>,
    /// The layer that found it, when another layer reports it.
    pub layer: Option<&'static str>,
    /// The feature not built, when it is a refusal; `text` is then empty.
    pub refusal: Option<Unbuilt>,
    /// The rule a verdict says the program breaks; `INTERNAL` when the pipeline is at fault.
    pub code: Code,
    /// What a verdict says.
    pub text: String,
    /// What to write instead, when the layer knows.
    pub help: Option<String>,
}

impl Msg {
    pub fn at(at: Span, m: impl Into<Msg>) -> Msg {
        Msg {
            at: Some(at),
            ..m.into()
        }
    }
    /// A library's message with no place yet names the module it is from.
    pub fn in_module(self, module: &str) -> Msg {
        match self.at {
            Some(_) => self,
            None => Msg {
                module: Some(module.to_string()),
                ..self
            },
        }
    }
    pub fn from_layer(self, layer: &'static str) -> Msg {
        Msg {
            layer: Some(layer),
            ..self
        }
    }
}

impl Msg {
    /// The program breaks the rule `code` names.
    pub fn verdict(code: Code, text: impl Into<String>) -> Msg {
        Msg {
            code,
            ..Msg::from(text.into())
        }
    }
}

impl From<String> for Msg {
    fn from(text: String) -> Msg {
        Msg {
            at: None,
            module: None,
            layer: None,
            refusal: None,
            code: cpl_diag::codes::INTERNAL,
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

impl From<Unbuilt> for Msg {
    fn from(u: Unbuilt) -> Msg {
        Msg {
            refusal: Some(u),
            ..Msg::from(String::new())
        }
    }
}
