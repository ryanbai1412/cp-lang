//! Union-find with potentials: every vertex carries a number, and the structure knows the
//! difference between any two vertices of one component.
//!
//! `unite(a, b, d)` records `pot(b) - pot(a) == d`. It answers "are these measurements
//! consistent?" and "what is `x[b] - x[a]`?" for systems of difference equations.

/// Components of `V` with an `Int` potential per vertex. `unite` and `diff` are
/// O(alpha(n)) amortized; memory is three `[V]` arrays.
pub struct PotentialDsu<V>:
    up: [V]V
    size: [V]Int
    /// The potential of a vertex minus the potential of `up` of it.
    gap: [V]Int

    pub fn new() -> PotentialDsu<V>:
        PotentialDsu([V] v => v, [V] v => 1, [V] v => 0)

    /// The root of `v`'s component and `pot(v) - pot(root)`.
    pub fn find(mut self, v: V) -> (V, Int):
        var path: List<V> = []
        var x = v
        while up[x] != x:
            path.push(x)
            x = up[x]
        // walk back from the root, so each vertex is re-hung with its full gap
        var acc = 0
        for i in (0..path.len).rev:
            let y = path[i]
            acc += gap[y]
            gap[y] = acc
            up[y] = x
        (x, gap[v])

    pub fn same(mut self, a: V, b: V) -> Bool = self.find(a).0 == self.find(b).0

    /// `pot(b) - pot(a)`, or `none` when nothing relates them yet.
    pub fn diff(mut self, a: V, b: V) -> ?Int:
        let (ra, pa) = self.find(a)
        let (rb, pb) = self.find(b)
        if ra != rb: none else: pb - pa

    /// Record `pot(b) - pot(a) == d`. `false` if that contradicts what is already known
    /// (nothing changes then); `true` if it is new or agrees.
    pub fn unite(mut self, a: V, b: V, d: Int) -> Bool:
        let (ra, pa) = self.find(a)
        let (rb, pb) = self.find(b)
        if ra == rb: return pb - pa == d
        // pot(rb) - pot(ra) == pa + d - pb
        let w = pa + d - pb
        if size[ra] >= size[rb]:
            up[rb] = ra
            gap[rb] = w
            size[ra] += size[rb]
        else:
            up[ra] = rb
            gap[ra] = -w
            size[rb] += size[ra]
        true

@test
fn differences_match_hidden_values():
    domain V = 0..12
    for _ in 0..30:
        let hidden = [V] v => rand(-50, 50)
        var d = PotentialDsu<V>()
        var comp = [V] v => v.to_idx
        for _ in 0..14:
            let (a, b) = (V.nth(rand(0, 11)), V.nth(rand(0, 11)))
            assert d.unite(a, b, hidden[b] - hidden[a])
            let (old, new) = (comp[a], comp[b])
            for v in V:
                if comp[v] == old: comp[v] = new
            for u in V:
                for v in V:
                    assert d.diff(u, v) == (if comp[u] == comp[v]: hidden[v] - hidden[u] else: none)

@test
fn a_contradiction_is_refused():
    domain V = 1..=3
    var d = PotentialDsu<V>()
    assert d.unite(V(1), V(2), 5) and d.unite(V(2), V(3), -2)
    assert d.diff(V(1), V(3)) == 3 and d.diff(V(3), V(1)) == -3
    assert not d.unite(V(1), V(3), 4)
    assert d.unite(V(3), V(1), -3) and d.diff(V(1), V(3)) == 3
