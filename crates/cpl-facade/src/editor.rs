//! What an editor asks about a place in a file (contract 9's typing path).

use cpl_diag::Span;

use crate::session::Session;

/// Where a name is declared: a span of the file at `path`, which may be
/// another file of the snapshot or a module of std.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Definition {
    pub path: String,
    pub span: Span,
}

fn of(def: cpl_check::Definition) -> Definition {
    let cpl_check::Definition { path, span } = def;
    Definition { path, span }
}

/// What an editor shows for a place in a file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Hover {
    /// The name or expression asked about, in file positions.
    pub span: Span,
    pub text: String,
    /// The `///` comment above the declaration, wherever it is declared.
    pub doc: Option<String>,
    /// The cost the doc states, in the receiver's domains:
    /// `O(log |V|) amortized`.
    pub cost: Option<String>,
    /// What kind of value it is, when it has a type.
    pub typed: Option<Typed>,
    /// The text is a fn's signature, `fn name(..) -> ..`.
    pub signature: bool,
}

/// The kinds of value an editor treats apart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Typed {
    Int,
    Fn,
    Other,
}

fn all(mut found: Vec<cpl_check::Definition>) -> Vec<Definition> {
    let mut all = Vec::with_capacity(found.len());
    for def in &mut found {
        let path = std::mem::take(&mut def.path);
        all.push(Definition {
            path,
            span: def.span,
        });
    }
    all
}

impl Session {
    /// What to show for the place at `offset` of `file`: its type, its doc
    /// and the cost the doc states.
    pub fn hover(&self, file: &str, offset: u32) -> Option<Hover> {
        let found = self.asked(file).db.hover(file, offset)?;
        Some(Hover {
            span: found.span,
            text: found.text,
            doc: found.doc,
            cost: found.cost,
            typed: found.typed.map(|typed| match typed {
                cpl_check::Typed::Int => Typed::Int,
                cpl_check::Typed::Fn => Typed::Fn,
                cpl_check::Typed::Other => Typed::Other,
            }),
            signature: found.signature,
        })
    }

    /// Where the name at `offset` of `file` is declared, following a `use`
    /// into the module that declares it.
    pub fn definition(&self, file: &str, offset: u32) -> Option<Definition> {
        let mut inner = self.asked(file);
        inner.db.definition_location(file, offset).map(of)
    }

    /// Every declaration the name at `offset` may mean: the one
    /// [`Session::definition`] gives, or each candidate of a method call
    /// that is ambiguous (E0701), in the order they were weighed.
    pub fn definitions(&self, file: &str, offset: u32) -> Vec<Definition> {
        let mut inner = self.asked(file);
        all(inner.db.definitions(file, offset))
    }

    /// Where the type of the name at `offset` is declared.
    pub fn type_definition(&self, file: &str, offset: u32) -> Option<Definition> {
        let mut inner = self.asked(file);
        inner.db.type_definition(file, offset).map(of)
    }

    /// Every `impl` of the type or trait named at `offset`, in `file` and
    /// the modules it uses.
    pub fn implementations(&self, file: &str, offset: u32) -> Vec<Definition> {
        let mut inner = self.asked(file);
        all(inner.db.implementations(file, offset))
    }
}
