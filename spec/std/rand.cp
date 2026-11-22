//! Random numbers from a seed: the same seed gives the same numbers in `cpl run`
//! and in the built C++. `Rng(seed)` is a splitmix64 stream: O(1) per draw.

pub struct Rng(s: U64):
    /// A deterministic stream of random bits seeded by `seed`.
    ///
    /// The state is `U64(seed)` (two's complement for negative seeds) and each `next()` is one
    /// splitmix64 step, so equal seeds give equal streams on every build and target. There is no
    /// time-seeded constructor: tests and stress runs replay exactly.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var r = Rng(0); r.next() == 0xE220A8397B1DCDAF`
    /// example: `var a = Rng(7); var b = Rng(7); a.next() == b.next()`
    pub fn new(seed: Int) -> Rng = Rng(U64(seed) ^ U64(-7046029254386353131))
    /// The next 64 random bits: one splitmix64 step.
    ///
    /// The state advances by `0x9E3779B97F4A7C15` (wrapping) and the result is the splitmix64 mix of
    /// the new state: `z ^= z >> 30; z *= 0xBF58476D1CE4E5B9; z ^= z >> 27; z *= 0x94D049BB133111EB;
    /// z ^= z >> 31`, all wrapping. Every other `Rng` method draws its bits from `next`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var r = Rng(0); r.next(); r.next() == 0x6E789E6AA1B965F4`
    /// example: `var r = Rng(1); r.next() == 10451216379200822465`
    pub fn next(mut self) -> U64:
        s ^= s >> 12
        s ^= s << 25
        s ^= s >> 27
        s * U64(2685821657736338717)
    /// A uniformly random integer in `lo..=hi`, both ends inclusive, from one `next()`.
    ///
    /// With `n = hi - lo + 1`, the result is `lo + ((U128(next()) * n) >> 64)`: Lemire's
    /// multiply-high map, which uses exactly one draw (bias at most `n / 2^64`).
    ///
    /// cost: O(1)
    /// empty: traps
    /// traps: when `lo > hi`
    /// example: `var r = Rng(1); let x = r.int(1, 6); 1 <= x and x <= 6`
    /// example: `var r = Rng(5); r.int(4, 4) == 4`
    pub fn int(mut self, lo: Int, hi: Int) -> Int:
        assert lo <= hi, "`int` needs `lo <= hi`"
        // `hi - lo + 1` is up to 2 ** 64, past an `Int`, when `lo` is negative.
        if lo >= 0: return lo + Int(self.next() % (U64(hi - lo) + U64(1)))
        let width = I128(hi) - I128(lo) + I128(1)
        Int(I128(lo) + I128(self.next()) % width)
    /// A uniformly random `F64` in `[0, 1)`: the top 53 bits of `next()` times `2^-53`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var r = Rng(5); let x = r.float(); 0.0 <= x and x < 1.0`
    pub fn float(mut self) -> F64 = F64(Int(self.next() >> 11)) / 9007199254740992.0
    /// True with probability `p`: `float() < p`.
    ///
    /// `p <= 0` is always false and `p >= 1` always true; both still consume one draw so the
    /// stream stays aligned whatever `p` is.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var r = Rng(3); r.chance(1.0) == true`
    /// example: `var r = Rng(3); r.chance(0.0) == false`
    pub fn chance(mut self, p: F64) -> Bool = self.float() < p
    /// A uniformly random element of `xs`: `xs` at position `int(0..xs.len)`.
    ///
    /// The position is zero-based, so any key type works. An empty `xs` returns `None` without
    /// drawing.
    ///
    /// cost: O(1)
    /// empty: `None`, and no draw
    /// traps: never
    /// example: `var r = Rng(1); r.pick([7]) == Some(7)`
    /// example: `var r = Rng(1); let x = r.pick([4, 5, 6]).unwrap(); 4 <= x and x <= 6`
    /// example: `let e: List<Int> = []; var r = Rng(1); r.pick(e) == None`
    pub fn pick<S: Indexed, T>(mut self, xs: S) -> ?T:
        if xs.len == 0: return None
        Some(xs.at(self.int(0, xs.len - 1)))
    /// Shuffle `a` in place, uniformly (Fisher-Yates).
    ///
    /// For `i` from `a.len - 1` down to `1`, swap positions `i` and `int(0..=i)`. Positions are
    /// zero-based, so this works on any `IndexedMut`: a `List`, a `[D]T` over any domain, a `Str`.
    /// Keys stay where they are; only the values move.
    ///
    /// cost: O(n)
    /// empty: no change, and no draw
    /// traps: never
    /// example: `var r = Rng(2); var a = [3, 1, 2]; r.shuffle(mut a); a.sorted() == [1, 2, 3]`
    /// example: `var r = Rng(2); var a = [9]; r.shuffle(mut a); a == [9]`
    pub fn shuffle<S: IndexedMut>(mut self, mut a: S):
        for i in (1..a.len).rev: a.swap_idx(i, self.int(0, i))
    /// A uniformly random permutation of the domain `D`, as a `[D]D`.
    ///
    /// It is the identity `[D]D` shuffled with `shuffle`, so a seed fixes the result. Every element
    /// of `D` appears exactly once among the values.
    ///
    /// cost: O(n)
    /// order: keyed by `D`; the value at each key is uniformly random, each element of `D` once
    /// traps: never
    /// example: `domain V = 0..4; var r = Rng(4); let p = r.perm<V>(); p.distinct().len == 4`
    pub fn perm<D>(mut self) -> [D]D:
        var p: [D]D = [D] d => d
        self.shuffle(mut p)
        p
    /// `k` elements of `from` at distinct positions, chosen uniformly, in random order.
    ///
    /// It runs a partial Fisher-Yates over a copy of the positions: for `i` in `0..k`, swap
    /// positions `i` and `int(i..n)`, then return the values at the first `k` positions. Equal
    /// values at different positions can both be chosen.
    ///
    /// cost: O(|from|)
    /// empty: `[]` when `k == 0`
    /// order: the order drawn (random); a new `List`, keyed from 0
    /// traps: when `k` is not in `0..=from.len`
    /// example: `var r = Rng(9); r.sample(3, from: [5, 6, 7]).sorted() == [5, 6, 7]`
    /// example: `var r = Rng(9); r.sample(0, from: [5, 6, 7]) == []`
    /// example: `var r = Rng(9); r.sample(2, from: [1, 1, 1]) == [1, 1]`
    pub fn sample<S: Indexed, T>(mut self, k: Int, from: S) -> List<T>:
        let n = from.len
        assert 0 <= k and k <= n
        var pos = [i for i in 0..n]
        for i in 0..k: pos.swap_idx(i, self.int(i, n - 1))
        [from.at(pos[i]) for i in 0..k]

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
