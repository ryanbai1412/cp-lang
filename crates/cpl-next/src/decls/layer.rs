//! An arena a program's declarations extend without copying std's: std's entries are shared, and
//! one the program changes is copied the first time it does. A copy of a layer shares its
//! entries too, until one of the two writes them.
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::sync::Arc;

pub struct Layer<T> {
    base: Arc<Vec<T>>,
    changed: HashMap<usize, T>,
    added: Arc<Vec<T>>,
}

impl<T> Default for Layer<T> {
    fn default() -> Self {
        Layer {
            base: Arc::new(Vec::new()),
            changed: HashMap::new(),
            added: Arc::new(Vec::new()),
        }
    }
}

impl<T: Clone> Clone for Layer<T> {
    fn clone(&self) -> Self {
        Layer {
            base: self.base.clone(),
            changed: self.changed.clone(),
            added: self.added.clone(),
        }
    }
}

impl<T> Layer<T> {
    pub fn len(&self) -> usize {
        self.base.len() + self.added.len()
    }
    pub fn push(&mut self, t: T)
    where
        T: Clone,
    {
        Arc::make_mut(&mut self.added).push(t);
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator + Clone {
        (0..self.len()).map(move |i| &self[i])
    }
    /// Every entry, shared by each layer made from this one.
    pub fn frozen(self) -> Layer<T>
    where
        T: Clone,
    {
        let Layer {
            base,
            mut changed,
            added,
        } = self;
        let take = |v: Arc<Vec<T>>| Arc::try_unwrap(v).unwrap_or_else(|v| (*v).clone());
        let mut all = take(base);
        for (i, t) in changed.drain() {
            all[i] = t;
        }
        all.extend(take(added));
        Layer {
            base: Arc::new(all),
            changed: HashMap::new(),
            added: Arc::new(Vec::new()),
        }
    }
}

impl<T> Index<usize> for Layer<T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        match i.checked_sub(self.base.len()) {
            Some(k) => &self.added[k],
            None if self.changed.is_empty() => &self.base[i],
            None => self.changed.get(&i).unwrap_or(&self.base[i]),
        }
    }
}

impl<T: Clone> IndexMut<usize> for Layer<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        match i.checked_sub(self.base.len()) {
            Some(k) => &mut Arc::make_mut(&mut self.added)[k],
            None => {
                let base = &self.base;
                self.changed.entry(i).or_insert_with(|| base[i].clone())
            }
        }
    }
}

impl<T> FromIterator<T> for Layer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(it: I) -> Self {
        Layer {
            base: Arc::new(Vec::new()),
            changed: HashMap::new(),
            added: Arc::new(it.into_iter().collect()),
        }
    }
}
