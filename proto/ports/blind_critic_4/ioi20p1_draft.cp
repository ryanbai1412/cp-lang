// Blind Critic #4 — First Draft (written from docs/one-page-cheat-sheet.md v2.1 ONLY)
// Problem: ioi20p1 (Plants)
// Original: corpus/submissions/5725518.cpp
// Exercises: LazySegTree custom Action, Set<V> circular neighbor queries, Euclidean %, binary lifting with ?Int, Map.first_gt, @grader struct

struct MaxPos<V>:
    mx: Int = -1_000_000_000_000
    pos: ?V = none

fn merge_max<V>(a: MaxPos<V>, b: MaxPos<V>) -> MaxPos<V>:
    if a.mx > b.mx: a else: b

struct ActiveRing<V>:
    n: Int
    k: Int
    active: Set<V> = Set<V>()
    ready: Set<V> = Set<V>()

    fn is_ready(self, x: V) -> Bool:
        if self.active.len == 1:
            return true
        let prv = self.active.last_lt(x) ?? self.active.last!
        // Euclidean % guarantees positive cyclic distance in 0..n!
        return (x - prv) % self.n >= self.k

    fn refresh(mut self, cand: ?V):
        if let x = cand:
            if self.is_ready(x):
                self.ready.add(x)
            else:
                self.ready.remove(x)

    fn insert(mut self, x: V):
        self.active.add(x)
        self.refresh(x)
        self.refresh(self.active.first_gt(x) ?? self.active.first)

    fn erase(mut self, x: V):
        self.active.remove(x)
        self.ready.remove(x)
        self.refresh(self.active.first_gt(x) ?? self.active.first)

fn solve_layers(k: Int, r: List<Int>) -> List<Int>:
    let n = r.len
    domain V = 0..n
    let init_nodes: [V]MaxPos<V> = [V] v => MaxPos<V>(mx: r[v.to_idx], pos: v)

    // BLIND-GUESS(H10): exact argument order of Action(id, compose, apply, over:) is not documented in cheat sheet §11!
    let act = Action(
        0,
        (a, b) => a + b,
        (node, tag) => MaxPos<V>(mx: node.mx + tag, pos: node.pos),
        over: Monoid(MaxPos<V>(), merge_max),
    )
    var st = LazySegTree<V, MaxPos<V>>(act, init_nodes)
    var ring = ActiveRing<V>(n: n, k: k)

    var layer: [V]Int = 0
    var done = 0
    var cur_layer = 0

    while done < n:
        while true:
            // BLIND-GUESS(H10b): full-range fold — does `st.fold(V.first..=V.last)` or `st.fold(..)` work?
            let top = st.fold(V.first..=V.last)
            if top.mx == k - 1:
                let u = top.pos!
                ring.insert(u)
                st.apply(u..=u, -1_000_000_000)
            else:
                break

        let batch = [u for u in ring.ready]
        for u in batch:
            done += 1
            layer[u] = cur_layer
            if let lo = u - (k - 1):
                st.apply(lo..=u, 1)
            else:
                st.apply(..=u, 1)
                st.apply(u.cyc(-(k - 1)).., 1)
            ring.erase(u)
        cur_layer += 1

    return [layer[v] for v in V]

struct JumpTable:
    n: Int
    logn: Int
    layers: List<Int>
    lift: List<List<?Int>>

    fn build(k: Int, base_layers: List<Int>) -> JumpTable:
        let n = base_layers.len
        let logn = bit_len(n) + 2
        let dbl = base_layers + base_layers
        var lift = List.filled(logn + 1, List.filled(2 * n, none))
        var win = Map<Int, Int>()

        for i in (0..2 * n).rev:
            if i + k < 2 * n:
                win.remove(dbl[i + k])
            win[dbl[i]] = i
            if let (_, nxt) = win.first_gt(dbl[i]):
                lift[0][i] = nxt

        for p in 1..=logn:
            for j in 0..2 * n:
                if let mid = lift[p - 1][j]:
                    lift[p][j] = lift[p - 1][mid]

        return JumpTable(n: n, logn: logn, layers: dbl, lift: lift)

    fn reaches_lt(self, u0: Int, v0: Int) -> Bool:
        if self.layers[u0] >= self.layers[v0]:
            return false
        var u = u0
        let v = if v0 < u0: v0 + self.n else: v0
        for p in (0..=self.logn).rev:
            if let nu = self.lift[p][u] and self.layers[nu] <= self.layers[v]:
                u = nu
        return v <= u

// BLIND-GUESS(H12): `@grader struct` syntax has zero example in cheat sheet §8!
@grader
struct Plants:
    n: Int = 0
    fwd: ?JumpTable = none
    rev: ?JumpTable = none

    fn init(mut self, k: Int, r: List<Int>):
        self.n = r.len
        let layers = solve_layers(k, r)
        var rev_layers = layers.copy()
        reverse(mut rev_layers)
        self.fwd = JumpTable.build(k, layers)
        self.rev = JumpTable.build(k, rev_layers)

    fn compare_plants(self, x: Int, y: Int) -> Int:
        let f = self.fwd!
        let rv = self.rev!
        fn lt(u: Int, v: Int) -> Bool:
            f.reaches_lt(u, v) or rv.reaches_lt(self.n - 1 - u, self.n - 1 - v)
        if lt(x, y): -1
        elif lt(y, x): 1
        else: 0
