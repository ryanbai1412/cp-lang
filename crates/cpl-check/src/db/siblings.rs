//! `use a.f` inside `mod b:`: an inline module names its siblings, the other
//! inline modules of the file that holds it (R11.1.3, R11.2.2).

use super::*;

impl Database {
    /// The path and text of `parts` seen from `from`: a sibling inline
    /// module first, then whatever the loader finds.
    pub(super) fn find_module(&self, from: &str, parts: &[String]) -> Option<(String, String)> {
        if let ([name], Some((file, _))) = (parts, from.rsplit_once("::mod::")) {
            let path = format!("{file}::mod::{name}");
            if let Some(state) = self.files.get(&path) {
                return Some((path, state.text.to_string()));
            }
        }
        self.loader.load(from, parts)
    }
}
