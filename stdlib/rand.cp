//! Random numbers from a seed: the same seed gives the same numbers in `cpl run` and in the built C++.

/// A xorshift64* generator. Every draw is O(1); the state is one word.
pub struct Rng(s: U64):
    /// A generator started from `seed`; any seed works, also 0.
    pub fn new(seed: Int) -> Rng = Rng(U64(seed) ^ U64(-7046029254386353131))

    /// The next 64 random bits.
    pub fn next(mut self) -> U64:
        s ^= s >> 12
        s ^= s << 25
        s ^= s >> 27
        s * U64(2685821657736338717)

    /// A whole number in `lo..=hi`, each about equally likely.
    pub fn int(mut self, lo: Int, hi: Int) -> Int:
        assert lo <= hi, "`int` needs `lo <= hi`"
        // `hi - lo + 1` is up to 2 ** 64, past an `Int`, when `lo` is negative.
        if lo >= 0: return lo + Int(self.next() % (U64(hi - lo) + U64(1)))
        let width = I128(hi) - I128(lo) + I128(1)
        Int(I128(lo) + I128(self.next()) % width)

    /// A number in `0.0..1.0`.
    pub fn float(mut self) -> F64 = F64(Int(self.next() >> 11)) / 9007199254740992.0

    /// `a` in a random order, in place. Works on any `IndexedMut` (`List`, `[D]T`, `Str`). O(n).
    pub fn shuffle<S: IndexedMut>(mut self, mut a: S):
        for i in (1..a.len).rev: a.swap_idx(i, self.int(0, i))

/// A permutation of domain `V`: a bijection `[V]V`.
pub type Perm<V> = [V]V

/// A generated unweighted graph on `V` with both `edges` and adjacency `g[u]`.
pub struct GenGraph<V>:
    edges: List<(u: V, v: V)>
    adj: [V]List<V>

    pub fn get(self, u: V) -> List<V> = adj[u]

    pub fn out(self, u: V) -> yields V:
        for v in adj[u]:
            yield v

/// A generated weighted graph on `V` with both `edges` and adjacency `g[u]`.
pub struct GenWeightedGraph<V>:
    edges: List<(u: V, v: V, w: Int)>
    adj: [V]List<(to: V, w: Int)>

    pub fn get(self, u: V) -> List<(to: V, w: Int)> = adj[u]

    pub fn out(self, u: V) -> yields (to: V, w: Int):
        for a in adj[u]:
            yield a

/// A random string over `alphabet` of length in `len_lo..=len_hi`, choice-recorded for shrinking.
pub fn gen_str(alphabet: Str, len_lo: Int, len_hi: Int) -> Str:
    let n = rand(len_lo, len_hi)
    var s = ""
    let k = alphabet.len
    if k == 0 or n <= 0: return s
    for _ in 0..n:
        s.push(alphabet[rand(0, k - 1)])
    s

/// A random permutation of `V`, choice-recorded for shrinking.
pub fn gen_perm<V>() -> Perm<V>:
    var p: [V]V = [V] i => i
    for i in (1..V.K).rev:
        p.swap_idx(i, rand(0, i))
    p

/// A random parent array `[V]?V` for a rooted tree of `shape` (`random`, `path`, `star`, `caterpillar`, `binary`).
pub fn gen_parents<V>(shape: Str = "random") -> [V]?V:
    var par: [V]?V = [V] _ => none
    let n = V.K
    if n <= 1: return par
    if shape == "path":
        for i in 1..n:
            par[V.nth(i)] = V.nth(i - 1)
    elif shape == "star":
        for i in 1..n:
            par[V.nth(i)] = V.nth(0)
    elif shape == "binary":
        for i in 1..n:
            par[V.nth(i)] = V.nth((i - 1) / 2)
    elif shape == "caterpillar":
        let spine = rand(1, n)
        for i in 1..spine:
            par[V.nth(i)] = V.nth(i - 1)
        for i in spine..n:
            par[V.nth(i)] = V.nth(rand(0, spine - 1))
    else:
        for i in 1..n:
            par[V.nth(i)] = V.nth(rand(0, i - 1))
    par

/// A random unweighted graph on `V`, choice-recorded for shrinking.
pub fn gen_graph<V>(simple: Bool = true, connected: Bool = false, m_lo: Int = -1, m_hi: Int = -1) -> GenGraph<V>:
    let n = V.K
    var edges: List<(u: V, v: V)> = []
    var adj: [V]List<V> = [V] _ => []
    if n == 0: return GenGraph(edges, adj)
    var seen: Set<(Int, Int)> = Set()
    let min_m = if connected and n > 1: n - 1 else: 0
    let max_possible = if simple: n * (n - 1) / 2 else: n * n
    let lo = if m_lo >= 0: max(m_lo, min_m) else: min_m
    let hi_raw = if m_hi >= 0: m_hi else: min(max_possible, max(min_m, 2 * n))
    let hi = max(lo, min(hi_raw, max_possible))
    let target = if lo <= hi: rand(lo, hi) else: lo
    if connected and n > 1:
        for i in 1..n:
            let p = rand(0, i - 1)
            let a = min(p, i)
            let b = max(p, i)
            seen.insert((a, b))
            let (u, v) = (V.nth(p), V.nth(i))
            edges.push((u:, v:))
            adj[u].push(v)
            adj[v].push(u)
    var tries = 0
    let max_tries = (target + 1) * 16 + 32
    while edges.len < target and tries < max_tries:
        tries += 1
        let x = rand(0, n - 1)
        let y = rand(0, n - 1)
        if simple:
            if x == y: continue
            let a = min(x, y)
            let b = max(x, y)
            if (a, b) in seen: continue
            seen.insert((a, b))
        let (u, v) = (V.nth(x), V.nth(y))
        edges.push((u:, v:))
        adj[u].push(v)
        if x != y: adj[v].push(u)
    GenGraph(edges, adj)

/// A random weighted graph on `V` with edge weights in `w_lo..=w_hi`, choice-recorded for shrinking.
pub fn gen_weighted_graph<V>(
    w_lo: Int,
    w_hi: Int,
    simple: Bool = true,
    connected: Bool = false,
    m_lo: Int = -1,
    m_hi: Int = -1,
) -> GenWeightedGraph<V>:
    let base = gen_graph::<V>(simple:, connected:, m_lo:, m_hi:)
    var edges: List<(u: V, v: V, w: Int)> = []
    var adj: [V]List<(to: V, w: Int)> = [V] _ => []
    for e in base.edges:
        let w = rand(w_lo, w_hi)
        edges.push((u: e.u, v: e.v, w:))
        adj[e.u].push((to: e.v, w:))
        if e.u != e.v: adj[e.v].push((to: e.u, w:))
    GenWeightedGraph(edges, adj)

@test
fn the_same_seed_gives_the_same_numbers():
    var (a, b) = (Rng.new(7), Rng.new(7))
    assert all(a.next() == b.next() for _ in 0..10)
    var r = Rng.new(0)
    assert all(3 <= x and x <= 5 for x in [r.int(3, 5) for _ in 0..100])
    assert all(0.0 <= x and x < 1.0 for x in [r.float() for _ in 0..100])
    var xs = [i for i in 0..20]
    r.shuffle(mut xs)
    assert sorted(xs) == [i for i in 0..20] and xs != [i for i in 0..20]

@test
fn int_covers_the_widest_ranges():
    var rng = Rng.new(7)
    for _ in 0..200:
        assert rng.int(0, Int.MAX) >= 0
        assert rng.int(Int.MIN, -1) < 0
        let x = rng.int(Int.MIN, Int.MAX)
        assert Int.MIN <= x and x <= Int.MAX
        let y = rng.int(-3, 3)
        assert -3 <= y and y <= 3
    assert rng.int(Int.MAX, Int.MAX) == Int.MAX
    assert rng.int(Int.MIN, Int.MIN) == Int.MIN
