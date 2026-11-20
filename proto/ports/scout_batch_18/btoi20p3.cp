// Baltic OI '20 P3 - Joker (DMOJ: btoi20p3, 25pt, TL 1.0s, ML 256MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Undirected graph with n vertices, m edges, and q queries (l, r): after
// removing edges l..=r, does the remaining graph contain an odd cycle?
// Print YES if an odd cycle exists, NO if bipartite.
//
// Let limit[l] be the smallest r >= l such that removing l..=r leaves a
// bipartite graph (or m + 1 if even edges 1..l-1 already have an odd cycle).
// limit[l] is monotone non-decreasing in l, so divide-and-conquer over l
// coupled with a rollback parity-DSU computes all limit[l] in O((n + m) log^2 m).
// Query (l, r) is YES iff r < limit[l].

let (n, m, q): (1..=2e5, 1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 1..=m
domain Q = 0..q
let edges: [E](u: V, v: V) = read()
let queries: [Q](l: E, r: E) = read() where all(x.l <= x.r for x in queries)

// WISH W12: bare-core fallback — `stdlib/dsu.cp` has path-halving DSU only,
// with neither parity/potential weights nor rollback (`Journaled` / `Rollback`).
struct RollbackParityDsu<V>:
    up: [V]V
    sz: [V]Int
    par: [V]Bool
    hist: List<(child: V, parent: V, old_sz: Int)>
    odd_cycle: Int

    fn new() -> RollbackParityDsu<V>:
        RollbackParityDsu([V] v => v, [V] _ => 1, [V] _ => false, [], 0)

    fn find(self, v: V) -> (root: V, flip: Bool):
        var x = v
        var f = false
        while up[x] != x:
            f = f != par[x]
            x = up[x]
        (root: x, flip: f)

    fn mark(self) -> (h: Int, odd: Int) = (h: hist.len, odd: odd_cycle)

    fn rollback(mut self, snap: (h: Int, odd: Int)):
        odd_cycle = snap.odd
        while hist.len > snap.h:
            let rec = hist.pop()
            up[rec.child] = rec.child
            par[rec.child] = false
            sz[rec.parent] = rec.old_sz

    fn add_edge(mut self, u: V, v: V):
        let (ru, fu) = self.find(u)
        let (rv, fv) = self.find(v)
        if ru == rv:
            if fu == fv: odd_cycle += 1
            return
        var (a, b) = (ru, rv)
        if sz[a] < sz[b]: swap(mut a, mut b)
        hist.push((child: b, parent: a, old_sz: sz[a]))
        up[b] = a
        par[b] = (fu == fv)
        sz[a] += sz[b]

fn solve() -> [Q]Bool:
    var dsu = RollbackParityDsu<V>.new()
    var limit: [E]Int = m + 1

    // Invariant: dsu currently holds edges 1..(l_lo - 1) and (r_hi + 1)..m.
    mut fn dnc(l_lo: Int, l_hi: Int, r_lo: Int, r_hi: Int):
        if l_lo > l_hi: return
        let mid = (l_lo + l_hi) / 2
        let snap0 = dsu.mark()
        // WISH W13: `scoped(dsu)` / `defer dsu.rollback(snap0)` would prevent
        // mismatched mark/rollback pairs across the 3 rollback regions here.
        for i in l_lo..mid:
            let e = edges[E(i)]
            dsu.add_edge(e.u, e.v)

        var opt = m + 1
        if dsu.odd_cycle == 0:
            opt = r_hi
            var cur = r_hi
            while cur >= max(r_lo, mid):
                let e = edges[E(cur)]
                dsu.add_edge(e.u, e.v)
                if dsu.odd_cycle > 0: break
                opt = cur - 1
                cur -= 1
            // Note: opt is the smallest r such that [1..mid-1] U [r+1..m] is bipartite.
            // If even cur == max(r_lo, mid) stayed bipartite, opt may be mid - 1,
            // meaning removing nothing (or just l=mid) is already bipartite.
        limit[E(mid)] = opt
        dsu.rollback(snap0)

        // Left half: [l_lo, mid - 1], optimal r in [r_lo, min(opt, r_hi)]
        let left_r_hi = min(opt, r_hi)
        if l_lo <= mid - 1:
            let snap_l = dsu.mark()
            for i in (left_r_hi + 1)..=r_hi:
                let e = edges[E(i)]
                dsu.add_edge(e.u, e.v)
            dnc(l_lo, mid - 1, r_lo, left_r_hi)
            dsu.rollback(snap_l)

        // Right half: [mid + 1, l_hi], optimal r in [max(r_lo, opt), r_hi]
        let right_r_lo = max(r_lo, min(opt, m))
        if mid + 1 <= l_hi:
            let snap_r = dsu.mark()
            for i in l_lo..=mid:
                let e = edges[E(i)]
                dsu.add_edge(e.u, e.v)
            dnc(mid + 1, l_hi, right_r_lo, r_hi)
            dsu.rollback(snap_r)

    dnc(1, m, 1, m)
    [Q] qi => queries[qi].r.to_idx + 1 < limit[queries[qi].l]

@brute(solve)
fn solve_brute() -> [Q]Bool
    where n <= 50 and m <= 80 and q <= 80:
    var out: [Q]Bool = false
    for qi in Q:
        let (ql, qr) = (queries[qi].l, queries[qi].r)
        var adj: [V]List<V> = []
        for e in E:
            if e < ql or e > qr:
                adj[edges[e].u].push(edges[e].v)
                adj[edges[e].v].push(edges[e].u)
        var color: [V]?Bool = none
        var has_odd = false
        for s in V:
            if color[s] != none: continue
            color[s] = false
            var queue = [s]
            var head = 0
            while head < queue.len and not has_odd:
                let u = queue[head]
                head += 1
                for v in adj[u]:
                    if color[v] == none:
                        color[v] = not color[u]!
                        queue.push(v)
                    elif color[v] == color[u]:
                        has_odd = true
        out[qi] = has_odd
    out

for has_odd in solve():
    print(if has_odd: "YES" else: "NO")
