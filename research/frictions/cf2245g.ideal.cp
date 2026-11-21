// Ideal CP Lang program for cf2245g (G. NPC Challenge — interactive tree recovery).
// Not gated: upstream ecnerwala/contest-programming CF/2245/G/G.cpp is an unrelated
// unfinished 2D-grid draft, and gate.sh has no interactor harness.

@!multitest
let n: 2..=1000 = read()
domain V = 1..=n

fn query(vs: List<V>) -> List<Bool>:
    print("?", vs.len, vs)
    flush()
    let s: String = read()
    if s == "-1": exit(0)
    return [c == '1' for c in s]

// Maintain the discovered forest with 2-coloring (each component is bipartite,
// so vertices of the same color in different components form an independent set
// unless there is an undiscovered edge between them).
var dsu = Dsu<V>()
var edges = List<(u: V, v: V)>()
var color: [V]Bool = false

// Find all edges from `u` into an independent set `pool` using binary search on
// greedy independent-set queries `[pool[..mid], u]`: `u` is rejected iff `pool[..mid]`
// contains a neighbor of `u`.
fn find_neighbors(u: V, pool: List<V>) -> List<V>:
    var res = List<V>()
    var lo = 0
    while lo < pool.len:
        let probe = pool[lo..] + [u]
        if query(probe).last!:
            break
        let idx = bsearch(lo..pool.len, mid => !query(pool[lo..=mid] + [u]).last!)
        res.push(pool[idx])
        lo = idx + 1
    return res

// Repeatedly peel a maximal independent set of remaining components via a single
// query on all vertices, then connect each rejected vertex to its neighbors in the
// independent set.
var active = [v for v in V]
while edges.len < n - 1:
    let got = query(active)
    var indep = List<V>()
    var rest = List<V>()
    for i in 0..active.len:
        if got[i]: indep.push(active[i]) else: rest.push(active[i])
    for u in rest:
        for v in find_neighbors(u, indep):
            if dsu.unite(u, v):
                edges.push((u: u, v: v))
    active = indep

print("!")
for e in edges:
    print(e.u, e.v)
flush()
