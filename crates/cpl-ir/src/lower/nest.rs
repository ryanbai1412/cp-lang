//! A stack of scopes that always has its outermost level, so the
//! innermost one is there to name without an `Option`.

#[derive(Clone)]
pub(super) struct Nest<T> {
    outer: T,
    inner: Vec<T>,
}

impl<T: Default> Default for Nest<T> {
    fn default() -> Self {
        Nest {
            outer: T::default(),
            inner: Vec::new(),
        }
    }
}

impl<T> Nest<T> {
    pub(super) fn push(&mut self, level: T) {
        self.inner.push(level);
    }

    /// Closes the innermost level; the outermost one stays open.
    pub(super) fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    pub(super) fn len(&self) -> usize {
        1 + self.inner.len()
    }

    pub(super) fn outer(&self) -> &T {
        &self.outer
    }

    pub(super) fn outer_mut(&mut self) -> &mut T {
        &mut self.outer
    }

    pub(super) fn top(&self) -> &T {
        self.inner.last().unwrap_or(&self.outer)
    }

    pub(super) fn top_mut(&mut self) -> &mut T {
        match self.inner.last_mut() {
            Some(level) => level,
            None => &mut self.outer,
        }
    }

    /// Outermost first.
    pub(super) fn iter(&self) -> std::iter::Chain<std::iter::Once<&T>, std::slice::Iter<'_, T>> {
        std::iter::once(&self.outer).chain(&self.inner)
    }

    /// The depth (outermost 0) of the innermost level `f` holds for.
    pub(super) fn rposition(&self, mut f: impl FnMut(&T) -> bool) -> Option<usize> {
        match self.inner.iter().rposition(&mut f) {
            Some(i) => Some(i + 1),
            None => f(&self.outer).then_some(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Nest;

    #[test]
    fn the_outer_level_outlives_every_pop() {
        let mut nest: Nest<Vec<u8>> = Nest::default();
        nest.top_mut().push(1);
        nest.push(vec![2]);
        assert_eq!(nest.len(), 2);
        assert_eq!(nest.top(), &vec![2]);
        assert_eq!(nest.iter().next_back(), Some(&vec![2]));
        assert_eq!(nest.rposition(|l| l.contains(&1)), Some(0));
        assert_eq!(nest.rposition(|l| !l.is_empty()), Some(1));
        assert_eq!(nest.pop(), Some(vec![2]));
        assert_eq!(nest.pop(), None);
        assert_eq!(nest.len(), 1);
        nest.top_mut().push(3);
        assert_eq!(nest.outer(), &vec![1, 3]);
    }
}
