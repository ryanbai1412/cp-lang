//! The one table that turns a name's text into an id. Every table after it is keyed by `Sym`,
//! so a lookup that starts from text asks here first.
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Sym(pub u32);

#[derive(Default, Clone)]
pub struct Syms {
    ids: Arc<HashMap<String, Sym>>,
    texts: Arc<Vec<String>>,
}

impl Syms {
    pub fn intern(&mut self, text: &str) -> Sym {
        if let Some(&s) = self.ids.get(text) {
            return s;
        }
        let s = Sym(self.texts.len() as u32);
        Arc::make_mut(&mut self.texts).push(text.to_string());
        Arc::make_mut(&mut self.ids).insert(text.to_string(), s);
        s
    }
    /// How many names are interned; the next fresh `Sym` is this one.
    pub fn len(&self) -> usize {
        self.texts.len()
    }
    pub fn is_empty(&self) -> bool {
        self.texts.is_empty()
    }
    /// The id of a name some declaration gave, or `None` when none did.
    pub fn get(&self, text: &str) -> Option<Sym> {
        self.ids.get(text).copied()
    }
    pub fn text(&self, s: Sym) -> &str {
        &self.texts[s.0 as usize]
    }
}

/// Where a type's names find their ids: the declarations', and in a body also its own names,
/// such as a domain it declares.
pub trait SymLookup {
    fn get(&self, text: &str) -> Option<Sym>;
}

impl SymLookup for Syms {
    fn get(&self, text: &str) -> Option<Sym> {
        Syms::get(self, text)
    }
}
