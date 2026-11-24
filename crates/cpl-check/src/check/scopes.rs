//! The checker's lexical scopes, innermost last. The root scope is always
//! there, so the innermost one needs no check.

use super::Local;

pub(crate) type Scope = crate::names::NameMap<Local>;

#[derive(Default)]
pub(crate) struct Scopes {
    outer: Vec<Scope>,
    inner: Scope,
}

impl Scopes {
    pub(crate) fn push(&mut self) {
        self.outer.push(std::mem::take(&mut self.inner));
    }

    /// Leaves the innermost scope; leaving the root one empties it.
    pub(crate) fn pop(&mut self) {
        self.inner = self.outer.pop().unwrap_or_default();
    }

    /// How many scopes are open, the root one included.
    pub(crate) fn len(&self) -> usize {
        self.outer.len() + 1
    }

    pub(crate) fn innermost(&mut self) -> &mut Scope {
        &mut self.inner
    }

    /// Each scope and its depth (the root is 0), innermost first.
    pub(crate) fn inner_first(&self) -> impl Iterator<Item = (usize, &Scope)> {
        std::iter::once((self.outer.len(), &self.inner)).chain(self.outer.iter().enumerate().rev())
    }

    /// Each scope, root first.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.outer.iter().chain(std::iter::once(&self.inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local() -> Local {
        Local {
            ty: crate::ty::Ty::Int,
            mutable: false,
            span: cpl_diag::Span::new(0, 0),
            keyword: "let",
        }
    }

    #[test]
    fn popping_past_the_root_leaves_an_empty_root() {
        let mut scopes = Scopes::default();
        scopes.innermost().insert("a".into(), local());
        scopes.push();
        scopes.innermost().insert("b".into(), local());
        let depths: Vec<_> = scopes.inner_first().map(|(d, s)| (d, s.len())).collect();
        assert_eq!(depths, [(1, 1), (0, 1)]);
        scopes.pop();
        assert!(scopes.innermost().contains_key("a") && scopes.len() == 1);
        scopes.pop();
        scopes.innermost().insert("c".into(), local());
        assert_eq!(scopes.iter().map(|s| s.len()).collect::<Vec<_>>(), [1]);
    }
}
