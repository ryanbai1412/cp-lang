//! Disjoint sets over a domain.

/// Union-find with path halving and union by size.
/// `find`, `unite`, `same` and `size` are O(alpha(n)) amortized; memory is two `[V]` arrays.
pub struct Dsu<V>:
    up: [V]V
    sz: [V]Int
    parts: Int

    /// Every position starts in its own set. O(n).
    pub fn new() -> Dsu<V>:
        Dsu([V] v => v, [V] v => 1, V.K)

    /// The representative of `v`'s set.
    pub fn find(mut self, v: V) -> V:
        var x = v
        while up[x] != x:
            up[x] = up[up[x]]
            x = up[x]
        x

    /// Merge the sets of `a` and `b`. Returns false if they were already one set.
    pub fn unite(mut self, a: V, b: V) -> Bool:
        var x = self.find(a)
        var y = self.find(b)
        if x == y: return false
        if sz[x] < sz[y]: swap(mut x, mut y)
        up[y] = x
        sz[x] += sz[y]
        parts -= 1
        true

    pub fn same(mut self, a: V, b: V) -> Bool = self.find(a) == self.find(b)

    /// The number of positions in `v`'s set.
    pub fn size(mut self, v: V) -> Int = sz[self.find(v)]

    /// The number of sets.
    pub @get fn count(self) -> Int = parts

@test
fn unite_and_find():
    domain V = 1..=6
    var d = Dsu<V>()
    assert d.count == 6
    assert d.unite(V(1), V(2))
    assert d.unite(V(3), V(4))
    assert not d.unite(V(2), V(1))
    assert d.same(V(1), V(2)) and not d.same(V(1), V(3))
    assert d.unite(V(2), V(4))
    assert d.size(V(3)) == 4 and d.size(V(5)) == 1
    assert d.count == 3

@test
fn agrees_with_labels_on_random_unions():
    domain V = 0..12
    var d = Dsu<V>()
    var label = [V] v => v.to_idx
    for _ in 0..40:
        let a = V.nth(rand(0, 11))
        let b = V.nth(rand(0, 11))
        let merged = label[a] != label[b]
        assert d.unite(a, b) == merged
        let (la, lb) = (label[a], label[b])
        for v in V:
            if label[v] == lb: label[v] = la
        for v in V:
            assert d.same(a, v) == (label[v] == la)
