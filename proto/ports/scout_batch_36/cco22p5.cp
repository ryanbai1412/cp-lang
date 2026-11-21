// CCO '22 P5 — Phone Plans / Two-Graph Connectivity (DMOJ: cco22p5)
// original: corpus/submissions/4801885.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Given two weighted undirected graphs A and B on n vertices and a target k,
// minimize wa + wb such that at least k unordered vertex pairs are connected
// in A (using edges <= wa) OR in B (using edges <= wb), or print -1.
//
// Inclusion-exclusion: pairs(A U B) = pairs(A) + pairs(B) - pairs(A & B).
// Build B completely using small-to-large component lists (recording per-edge
// rep snapshots for O(n log n) rollback), then sweep A-edges in sorted weight
// order while rolling B back as far as pairs(A U B) >= k allows.
let (n, ma, mb, k): (1..=2e5, 0..=2e5, 0..=2e5, 0..=2e10) = read()
domain V = 1..=n
type Edge = (u: V, v: V, w: Int)

var ea: [0..ma]Edge = read()
var eb: [0..mb]Edge = read()
ea.sort(by: e => e.w)
eb.sort(by: e => e.w)

fn c2(x: Int) -> Int: x * (x - 1) / 2

struct SmallToLarge:
    pairs: Int
    rep: [V]V
    members: [V]List<V>

    fn merge(mut self, u0: V, v0: V) -> List<V>:
        var (u, v) = (self.rep[u0], self.rep[v0])
        if u == v: return []
        if self.members[u].len > self.members[v].len: (u, v) = (v, u)
        self.pairs -= c2(self.members[u].len) + c2(self.members[v].len)
        var moved: List<V> = []
        for x in self.members[u]:
            self.rep[x] = v
            self.members[v].push(x)
            moved.push(x)
        self.members[u].clear()
        self.pairs += c2(self.members[v].len)
        moved

var dsu_a = SmallToLarge(0, [V] v => v, [V] v => [v])
var dsu_b = SmallToLarge(0, [V] v => v, [V] v => [v])

// Record every B-edge's vertex rep changes so we can roll B back in O(n log n).
var hist: List<(prev_pairs: Int, changes: List<(v: V, old_rep: V)>)> = []
var rep_b: [V]V = [V] v => v
for i in 0..mb:
    let prev = dsu_b.pairs
    let moved = dsu_b.merge(eb[i].u, eb[i].v)
    var ch: List<(v: V, old_rep: V)> = []
    for x in moved:
        ch.push((x, rep_b[x]))
        rep_b[x] = dsu_b.rep[x]
    hist.push((prev, ch))

var pairs_b = dsu_b.pairs
var dup = 0
var pair_rep: [V](V, V) = [V] v => (v, rep_b[v])
var freq = HashMap<(V, V), Int>()
for v in V:
    let c = freq.get(pair_rep[v]) ?? 0
    dup += c
    freq[pair_rep[v]] = c + 1

fn relabel(moved: List<V>):
    for x in moved:
        let old = pair_rep[x]
        let c_old = freq[old] - 1
        freq[old] = c_old
        dup -= c_old
        let nxt = (dsu_a.rep[x], rep_b[x])
        pair_rep[x] = nxt
        let c_new = freq.get(nxt) ?? 0
        dup += c_new
        freq[nxt] = c_new + 1

fn total_pairs() -> Int: dsu_a.pairs + pairs_b - dup

var j = mb - 1
var ans: ?Int = none

fn relax(wa: Int):
    while j >= 0 and total_pairs() >= k:
        let rec = hist.pop()!
        pairs_b = rec.prev_pairs
        var touched: List<V> = []
        for ch in rec.changes:
            rep_b[ch.v] = ch.old_rep
            touched.push(ch.v)
        relabel(touched)
        j -= 1
    if total_pairs() >= k: ans min= wa
    if j + 1 < mb: ans min= wa + eb[j + 1].w

relax(0)
for i in 0..ma:
    relabel(dsu_a.merge(ea[i].u, ea[i].v))
    relax(ea[i].w)

print(ans, none: -1)
