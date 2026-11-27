//! The one table that turns a name's text into an id. Every table after it is keyed by `Sym`,
//! so a lookup that starts from text asks here first.
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Sym(pub u32);

#[derive(Default, Clone)]
pub struct Syms {
    ids: HashMap<String, Sym>,
    texts: Vec<String>,
}

impl Syms {
    pub fn intern(&mut self, text: &str) -> Sym {
        if let Some(&s) = self.ids.get(text) {
            return s;
        }
        let s = Sym(self.texts.len() as u32);
        self.texts.push(text.to_string());
        self.ids.insert(text.to_string(), s);
        s
    }
    /// The id of a name some declaration gave, or `None` when none did.
    pub fn get(&self, text: &str) -> Option<Sym> {
        self.ids.get(text).copied()
    }
    pub fn text(&self, s: Sym) -> &str {
        &self.texts[s.0 as usize]
    }
}
