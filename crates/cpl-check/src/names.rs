//! `NameMap`: the checker's map keyed by a name. Its entries sit in one
//! `Vec`, scanned while short; past `SCAN` entries one `HashMap<String, u32>`
//! indexes them. Every `NameMap<V>` shares that one index type, so the hash
//! table's code is built once however many value types the checker keys by
//! name, where a `HashMap<String, V>` is built again for every `V`.

use crate::fx::HashMap;

const SCAN: usize = 8;

#[derive(Clone)]
pub struct NameMap<V> {
    entries: Vec<(String, V)>,
    index: Index,
}

/// Where each name's entry is; kept only while there are more than `SCAN`.
#[derive(Clone, Default)]
struct Index(Option<Box<HashMap<String, u32>>>);

impl Index {
    #[inline(never)]
    fn get(&self, name: &str) -> Option<usize> {
        self.0.as_ref()?.get(name).map(|&i| i as usize)
    }

    #[inline(never)]
    fn put(&mut self, name: &str, at: usize) {
        if let Some(map) = &mut self.0 {
            map.insert(name.to_string(), at as u32);
        }
    }

    #[inline(never)]
    fn take(&mut self, name: &str) {
        if let Some(map) = &mut self.0 {
            map.remove(name);
        }
    }

    #[inline(never)]
    fn rebuild(&mut self, names: &mut dyn Iterator<Item = &str>, len: usize) {
        self.0 = (len > SCAN).then(|| {
            Box::new(
                names
                    .enumerate()
                    .map(|(i, n)| (n.to_string(), i as u32))
                    .collect(),
            )
        });
    }
}

impl<V> Default for NameMap<V> {
    fn default() -> Self {
        NameMap {
            entries: Vec::new(),
            index: Index::default(),
        }
    }
}

impl<V> NameMap<V> {
    pub fn new() -> Self {
        Self::default()
    }

    fn find(&self, name: &str) -> Option<usize> {
        if self.entries.len() > SCAN {
            self.index.get(name)
        } else {
            self.entries.iter().position(|(n, _)| n == name)
        }
    }

    fn reindex(&mut self) {
        let len = self.entries.len();
        self.index
            .rebuild(&mut self.entries.iter().map(|(n, _)| n.as_str()), len);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.index = Index::default();
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.find(name).is_some()
    }

    pub fn get(&self, name: &str) -> Option<&V> {
        self.find(name).map(|i| &self.entries[i].1)
    }

    pub fn get_key_value(&self, name: &str) -> Option<(&String, &V)> {
        self.find(name)
            .map(|i| (&self.entries[i].0, &self.entries[i].1))
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut V> {
        self.find(name).map(|i| &mut self.entries[i].1)
    }

    fn push(&mut self, name: String, value: V) -> usize {
        let at = self.entries.len();
        self.entries.push((name, value));
        if at == SCAN {
            self.reindex();
        } else if at > SCAN {
            self.index.put(&self.entries[at].0, at);
        }
        at
    }

    pub fn insert(&mut self, name: String, value: V) -> Option<V> {
        match self.find(&name) {
            Some(i) => Some(std::mem::replace(&mut self.entries[i].1, value)),
            None => {
                self.push(name, value);
                None
            }
        }
    }

    pub fn remove(&mut self, name: &str) -> Option<V> {
        let at = self.find(name)?;
        let (_, value) = self.entries.swap_remove(at);
        if self.entries.len() <= SCAN {
            self.index = Index::default();
        } else {
            self.index.take(name);
            if at < self.entries.len() {
                self.index.put(&self.entries[at].0, at);
            }
        }
        Some(value)
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&String, &mut V) -> bool) {
        self.entries.retain_mut(|(n, v)| keep(n, v));
        self.reindex();
    }

    pub fn entry(&mut self, name: String) -> Entry<'_, V> {
        Entry { map: self, name }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &V)> + Clone {
        self.entries.iter().map(|(n, v)| (n, v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut V)> {
        self.entries.iter_mut().map(|(n, v)| (&*n, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> + Clone {
        self.entries.iter().map(|(n, _)| n)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> + Clone {
        self.entries.iter().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.entries.iter_mut().map(|(_, v)| v)
    }

    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.entries.into_iter().map(|(_, v)| v)
    }

    pub fn into_keys(self) -> impl Iterator<Item = String> {
        self.entries.into_iter().map(|(n, _)| n)
    }

    pub fn drain(&mut self) -> impl Iterator<Item = (String, V)> + '_ {
        self.index = Index::default();
        self.entries.drain(..)
    }
}

pub struct Entry<'a, V> {
    map: &'a mut NameMap<V>,
    name: String,
}

impl<'a, V> Entry<'a, V> {
    pub fn or_insert_with(self, make: impl FnOnce() -> V) -> &'a mut V {
        let at = match self.map.find(&self.name) {
            Some(i) => i,
            None => self.map.push(self.name, make()),
        };
        &mut self.map.entries[at].1
    }

    pub fn or_insert(self, value: V) -> &'a mut V {
        self.or_insert_with(|| value)
    }

    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }

    pub fn and_modify(self, f: impl FnOnce(&mut V)) -> Self {
        if let Some(v) = self.map.get_mut(&self.name) {
            f(v);
        }
        self
    }
}

impl<V: PartialEq> PartialEq for NameMap<V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().all(|(n, v)| other.get(n) == Some(v))
    }
}

impl<V: Eq> Eq for NameMap<V> {}

impl<V: std::fmt::Debug> std::fmt::Debug for NameMap<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<V> Extend<(String, V)> for NameMap<V> {
    fn extend<I: IntoIterator<Item = (String, V)>>(&mut self, items: I) {
        for (n, v) in items {
            self.insert(n, v);
        }
    }
}

impl<V> FromIterator<(String, V)> for NameMap<V> {
    fn from_iter<I: IntoIterator<Item = (String, V)>>(items: I) -> Self {
        let mut map = NameMap::new();
        map.extend(items);
        map
    }
}

impl<V> IntoIterator for NameMap<V> {
    type Item = (String, V);
    type IntoIter = std::vec::IntoIter<(String, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a, V> IntoIterator for &'a NameMap<V> {
    type Item = (&'a String, &'a V);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (String, V)>,
        fn(&'a (String, V)) -> (&'a String, &'a V),
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().map(|(n, v)| (n, v))
    }
}

#[cfg(test)]
mod tests {
    use super::NameMap;

    #[test]
    fn a_name_map_finds_every_name_across_the_scan_and_index_sizes() {
        let mut m = NameMap::new();
        for i in 0..40 {
            assert_eq!(m.insert(format!("n{i}"), i), None);
            for j in 0..=i {
                assert_eq!(m.get(&format!("n{j}")), Some(&j));
            }
        }
        assert_eq!(m.insert("n3".into(), 103), Some(3));
        for i in (0..40).step_by(3) {
            assert!(m.remove(&format!("n{i}")).is_some());
            assert!(!m.contains_key(&format!("n{i}")));
        }
        for i in 0..40 {
            let want = (i % 3 != 0).then_some(if i == 3 { 103 } else { i });
            assert_eq!(m.get(&format!("n{i}")).copied(), want, "n{i}");
        }
        m.retain(|_, v| *v % 2 == 0);
        assert!(m.iter().all(|(n, v)| m.get(n) == Some(v) && v % 2 == 0));
        *m.entry("fresh".into()).or_default() += 5;
        assert_eq!(m.get("fresh").copied(), Some(5));
        loop {
            let first = m.keys().next().cloned();
            let Some(n) = first else { break };
            m.remove(&n);
            assert!(m.iter().all(|(k, v)| m.get(k) == Some(v)));
        }
        assert!(m.is_empty());
    }
}
