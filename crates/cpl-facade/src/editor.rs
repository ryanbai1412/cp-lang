//! What an editor asks about a place in a file (contract 9's typing path).

pub use cpl_contract::{Definition, Hover, Typed};

use crate::session::Session;

fn of(def: cpl_check::Definition) -> Definition {
    let cpl_check::Definition { path, span } = def;
    Definition { path, span }
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
