//! A diagnostic as a layer reports it: a place inside an item (contract 1) and what it
//! says, a refusal as its [`Unbuilt`] feature.
use crate::ids::SourceId;
use crate::unbuilt::Unbuilt;
use cpl_diag::Code;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Msg {
    /// `None`: the layer does not know where yet.
    pub at: Option<SourceId>,
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
    /// Other places that explain it.
    pub labels: Vec<(SourceId, String)>,
    /// Edits that would make the program what was likely meant.
    pub fixes: Vec<Fix>,
}

/// An edit set that makes the program what was likely meant: apply them all.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fix {
    pub title: String,
    pub edits: Vec<(SourceId, String)>,
}

impl Msg {
    pub fn at(at: SourceId, m: impl Into<Msg>) -> Msg {
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
            ..Msg::internal(text)
        }
    }
}

impl Msg {
    /// The pipeline is at fault, not the program: no rule names it.
    pub fn internal(text: impl Into<String>) -> Msg {
        Msg {
            at: None,
            module: None,
            layer: None,
            refusal: None,
            code: cpl_diag::codes::INTERNAL,
            text: text.into(),
            help: None,
            labels: Vec::new(),
            fixes: Vec::new(),
        }
    }
}

impl From<Unbuilt> for Msg {
    fn from(u: Unbuilt) -> Msg {
        Msg {
            refusal: Some(u),
            ..Msg::internal("")
        }
    }
}

/// Which layer reported a diagnostic.
/// In pipeline order: a program's diagnostics are listed layer by layer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

/// The diagnostics of each library fn reached only through a trait, each naming the layer that
/// found it: the build says them if link makes an instance of the fn.
#[derive(Clone, Debug, Default)]
pub struct Deferred(std::collections::HashMap<crate::ty::FnId, Vec<Msg>>);

impl Deferred {
    pub fn add(&mut self, f: crate::ty::FnId, msgs: impl IntoIterator<Item = Msg>) {
        self.0.entry(f).or_default().extend(msgs);
    }
    pub fn of(&self, f: crate::ty::FnId) -> &[Msg] {
        self.0.get(&f).map_or(&[], Vec::as_slice)
    }
}
