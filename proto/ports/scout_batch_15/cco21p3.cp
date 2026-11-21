// CCO 2021 Weird Tour: a tree whose vertices each give their neighbors in a
// cyclic order. A walk starts at vertex 1 and repeatedly takes the next
// incident edge cyclically, inducing an Eulerian tour of the doubled tree
// where edges activate on successive passes. Answer q queries asking which
// vertex is visited at step x (1-based; 0 means vertex 1).
//
// Running the cyclic-order DFS computes the Eulerian tour of length 2(n - 1)
// and the activation pass `born[step]` of each directed step. Prefix sums of
// pass sizes locate which pass a query falls into; answering the x-th active
// step offline uses `std.fenwick.Fenwick.lower_bound`, completely replacing
// the original's `__gnu_pbds::tree` order-statistics tree (and dead `Bit`).
// original: corpus/submissions/4552873.cpp
use std.fenwick.{Fenwick}

let (n, q): (Int, Int) = read()
domain V = 0..n

var adj: [V]List<V> = [V] _ => []
for u in V:
    let deg: Int = read()
    var nbrs = List<V>()
    for _ in 0..deg:
        let v: Int = read()
        nbrs.push(V(v - 1))
    // Cyclic shift left by 1: entering from the first listed neighbor exits via the second.
    if nbrs.len > 1:
        adj[u] = nbrs[1..nbrs.len] + [nbrs[0]]
    else:
        adj[u] = nbrs

var tour_node = List<V>()
var born = List<Int>()

fn walk(u: V, parent: ?V, start_pass: Int):
    let d = adj[u].len
    var t = start_pass
    var pass_at = List.filled(d, 0)
    var p_idx: ?Int = none
    for i in 0..d:
        if parent != none and adj[u][i] == parent!:
            t += 1
            p_idx = i
        else:
            pass_at[i] = t
    var order = [i for i in 0..d]
    if let pi = p_idx:
        order = order[pi + 1..d] + order[0..pi]
    for idx in order:
        let v = adj[u][idx]
        let pt = pass_at[idx]
        born.push(pt)
        tour_node.push(v)
        walk(v, u, pt)
        born.push(pt)
        tour_node.push(u)

walk(V(0), none, 0)

let total_steps = tour_node.len
domain Step = 0..total_steps
domain Pass = 0..n + 2

var added = List.filled(n + 1, 0)
for pt in born: added[pt] += 1

var active_by_pass = List.filled(n + 1, 0)
var cum_steps = List.filled(n + 1, 0)
for p in 0..=n:
    active_by_pass[p] = (if p > 0: active_by_pass[p - 1] else: 0) + added[p]
    cum_steps[p] = (if p > 0: cum_steps[p - 1] else: 0) + active_by_pass[p]

// Bucket queries by the pass they land in.
var buckets: [Pass]List<(Int, Int)> = [Pass] _ => []
for qi in 0..q:
    var x: Int = read()
    var (lo, hi) = (0, n + 1)
    while lo < hi:
        let mid = (lo + hi) / 2
        if cum_steps[mid] >= x: hi = mid
        else: lo = mid + 1
    let p = lo
    if p > 0: x -= cum_steps[p - 1]
    if p > n: x %= total_steps
    buckets[Pass(p)].push((x, qi))

var by_pass: [Pass]List<Step> = [Pass] _ => []
for s in Step: by_pass[Pass(born[s.to_idx])].push(s)

var bit = Fenwick<Step, Int>.new()
var ans = List.filled(q, 0)
for p in Pass:
    for s in by_pass[p]: bit.add(s, 1)
    for (x, qi) in buckets[p]:
        if x == 0:
            ans[qi] = 1
        else:
            let pos = bit.first_ge(x)!.left
            ans[qi] = tour_node[pos.to_idx].to_idx + 1

for a in ans: print(a)
