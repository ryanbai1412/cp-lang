//! Disjoint sets over a domain.

/// Union-find with path halving and union by size.
/// `find`, `unite`, `same` and `size` are O(alpha(n)) amortized; memory is two `[V]` arrays.
///
/// It can also step back in time: `let snap = d.mark()` remembers the present and
/// `d.undo_to(snap)` returns to it. From the first
/// `mark` on, paths are no longer shortened (a shortened path cannot be restored), so every
/// operation is O(log n) and each `unite` costs one entry of memory until it is undone.
pub struct Dsu<V>:
    up: [V]V
    sz: [V]Int
    parts: Int
    /// The roots that were attached since the first `mark`, in order.
    joined: List<V>
    marked: Bool

    /// Every position starts in its own set. O(n).
    pub fn new() -> Dsu<V>:
        Dsu([V] v => v, [V] v => 1, V.K, [], false)

    /// The representative of `v`'s set.
    pub fn find(mut self, v: V) -> V:
        var x = v
        while up[x] != x:
            if not marked: up[x] = up[up[x]]
            x = up[x]
        x

    /// The present, to return to with `undo_to`. O(1).
    pub fn mark(mut self) -> Int:
        marked = true
        joined.len

    /// Undo every `unite` made since `snap = mark()`. O(1) per undone `unite`.
    pub fn undo_to(mut self, snap: Int):
        while joined.len > snap:
            let y = joined.pop()!
            sz[up[y]] -= sz[y]
            up[y] = y
            parts += 1

    /// Merge the sets of `a` and `b`. Returns false if they were already one set.
    pub fn unite(mut self, a: V, b: V) -> Bool:
        var x = self.find(a)
        var y = self.find(b)
        if x == y: return false
        if sz[x] < sz[y]: swap(mut x, mut y)
        up[y] = x
        sz[x] += sz[y]
        parts -= 1
        if marked: joined.push(y)
        true

    /// As `unite`, but says which root stayed and which went under it, so data kept per
    /// set can follow: `if let (kept, merged) = d.merge(a, b): sum[kept] += sum[merged]`.
    /// `none` if `a` and `b` were already one set.
    pub fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V):
        var x = self.find(a)
        var y = self.find(b)
        if x == y: return none
        if sz[x] < sz[y]: swap(mut x, mut y)
        self.unite(x, y)
        some((kept: x, merged: y))

    pub fn same(mut self, a: V, b: V) -> Bool = self.find(a) == self.find(b)

    /// The number of positions in `v`'s set.
    pub fn size(mut self, v: V) -> Int = sz[self.find(v)]

    /// The number of sets.
    pub @get fn components(self) -> Int = parts

    /// Every connected component as a list of its vertices, each vertex appearing once. O(n alpha(n)).
    pub fn groups(mut self) -> List<List<V>>:
        let root_of = [V] v => self.find(v)
        var buckets: [V]List<V> = []
        for v in V: buckets[root_of[v]].push(v)
        [buckets[v] for v in V if not buckets[v].is_empty]

@test
fn unite_and_find():
    domain V = 1..=6
    var d = Dsu<V>()
    assert d.components == 6
    assert d.unite(V(1), V(2))
    assert d.unite(V(3), V(4))
    assert not d.unite(V(2), V(1))
    assert d.same(V(1), V(2)) and not d.same(V(1), V(3))
    assert d.unite(V(2), V(4))
    assert d.size(V(3)) == 4 and d.size(V(5)) == 1
    assert d.components == 3

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

@test
fn undo_to_returns_to_every_mark():
    domain V = 0..12
    var d = Dsu<V>()
    d.unite(V(0), V(1))
    var snaps: List<(Int, List<V>, Int)> = []
    for round in 0..40:
        if rand(0, 2) > 0 or snaps.is_empty:
            snaps.push((d.mark(), [d.find(v) for v in V], d.components))
            d.unite(V.nth(rand(0, 11)), V.nth(rand(0, 11)))
        else:
            let (snap, roots, parts) = snaps.pop()!
            d.undo_to(snap)
            assert [d.find(v) for v in V] == roots and d.components == parts
            for v in V: assert d.size(v) == count(roots[u.to_idx] == roots[v.to_idx] for u in V)
    d.undo_to(0)
    assert d.same(V(0), V(1)) and d.components == 11

@test
fn groups_partitions_all_vertices_by_component():
    domain V = 1..=7
    var d = Dsu<V>()
    assert d.groups().len == 7
    assert [g.len for g in d.groups()] == [1, 1, 1, 1, 1, 1, 1]
    d.unite(V(1), V(3))
    d.unite(V(3), V(6))
    let snap = d.mark()
    d.unite(V(2), V(5))
    d.unite(V(5), V(7))
    let gs = d.groups()
    assert gs.len == d.components and gs.len == 3
    var flat: List<V> = []
    for g in gs:
        assert not g.is_empty
        for v in g:
            assert d.same(v, g[0])
            flat.push(v)
        assert g.len == d.size(g[0])
    assert sorted(flat) == [v for v in V]
    d.undo_to(snap)
    assert d.groups().len == 5 and d.components == 5
    for v in V: d.unite(V(1), v)
    assert d.groups() == [[v for v in V]]

@test
fn merge_says_which_root_stayed():
    domain V = 0..6
    var d = Dsu<V>()
    var total = [V] v => v.idx
    for (a, b) in [(0, 1), (2, 3), (1, 3), (0, 2), (4, 5)]:
        if let (kept, merged) = d.merge(V(a), V(b)):
            assert d.find(kept) == kept and d.find(merged) == kept
            total[kept] += total[merged]
        else:
            assert (a, b) == (0, 2)
    assert total[d.find(V(0))] == 6 and total[d.find(V(5))] == 9 and d.components == 2
