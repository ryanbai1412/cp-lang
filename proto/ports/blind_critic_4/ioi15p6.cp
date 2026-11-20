// IOI '15 P6 — Towns (runnable proto/cpc port of ioi15p6_draft.cp)
// Prototype deltas vs v2.1 draft: no bodyless `@grader fn`, no lexically scoped `@memo`,
// embeds an exact weighted-tree oracle reading `(n, edges)` from stdin and enforcing query budget.

let n: 3..=110 = read()
let m: 2..=300 = read()

// Read weighted tree with `m` internal+leaf vertices (leaves are 0..n-1).
domain T = 0..m
var adj: [T]List<(Int, Int)> = []
for _ in 0..m - 1:
    let u: Int = read()
    let v: Int = read()
    let w: Int = read()
    adj[T.from_idx(u)].push((v, w))
    adj[T.from_idx(v)].push((u, w))

// Oracle: exact shortest-path distances between leaves 0..n-1, cached with query counter.
domain V = 0..n
var true_dist: [V, V]Int = 0
for s in 0..n:
    var dist: [T]Int = -1
    var qn: List<Int> = [s]
    dist[T.from_idx(s)] = 0
    var head = 0
    while head < qn.len:
        let u = qn[head]
        head += 1
        let du = dist[T.from_idx(u)]
        for e in adj[T.from_idx(u)]:
            let tv = T.from_idx(e.0)
            if dist[tv] == -1:
                dist[tv] = du + e.1
                qn.push(e.0)
    for t in 0..n:
        true_dist[V.from_idx(s), V.from_idx(t)] = dist[T.from_idx(t)]

var memo_dist: [V, V]Int = -1
var queries = 0

fn ask_dist(a: Int, b: Int) -> Int:
    if a == b:
        return 0
    let va = V.from_idx(a)
    let vb = V.from_idx(b)
    if memo_dist[va, vb] != -1:
        return memo_dist[va, vb]
    queries += 1
    let d = true_dist[va, vb]
    memo_dist[va, vb] = d
    memo_dist[vb, va] = d
    return d

// Step 1: 2 sweeps to find diameter endpoints (u, w) from vertex 0.
var d0: [V]Int = 0
var u = 0
for i in 1..n:
    let d = ask_dist(0, i)
    d0[V.from_idx(i)] = d
    if d > d0[V.from_idx(u)]:
        u = i

var du: [V]Int = 0
var w = u
for i in 0..n:
    let d = ask_dist(u, i)
    du[V.from_idx(i)] = d
    if d > du[V.from_idx(w)]:
        w = i

let diam = du[V.from_idx(w)]

// Step 2: Project every vertex onto path (u, 0):
// dist_from_u[i] = dist(u, lca(0, u, i)) = (du[i] + du0 - d0[i]) / 2,
// drop[i]        = dist(i, lca(0, u, i)) = (du[i] + d0[i] - du0) / 2.
var dist_from_u: [V]Int = 0
var drop: [V]Int = 0
let du0 = d0[V.from_idx(u)]
for i in 0..n:
    let vi = V.from_idx(i)
    let d_u = du[vi]
    let d_0 = d0[vi]
    dist_from_u[vi] = (d_u + du0 - d_0) / 2
    drop[vi] = (d_u + d_0 - du0) / 2

// Group all vertices by their coordinate along the diameter path from u.
var coords: List<Int> = []
for i in 0..n:
    coords.push(dist_from_u[V.from_idx(i)])
coords.sort()

var uniq: List<Int> = []
var cnts: List<Int> = []
for x in coords:
    if uniq.len == 0 or uniq[uniq.len - 1] != x:
        uniq.push(x)
        cnts.push(1)
    else:
        cnts[cnts.len - 1] += 1

var radius = diam
for ci in 0..uniq.len:
    let x = uniq[ci]
    let r = if x > diam - x: x else: diam - x
    if r < radius:
        radius = r

// Find a hub coordinate `hub_x` with radius R and both prefix/suffix sides <= n/2.
var pref = 0
var hub_found = false
var hub_x = 0
var hub_cnt = 0
for ci in 0..uniq.len:
    let x = uniq[ci]
    let c = cnts[ci]
    let r = if x > diam - x: x else: diam - x
    let suf = n - pref - c
    if r == radius and pref <= n / 2 and suf <= n / 2:
        hub_found = true
        hub_x = x
        hub_cnt = c
        break
    pref += c

if not hub_found:
    print(-radius)
else:
    if hub_cnt <= n / 2:
        print(radius)
    else:
        // Boyer-Moore majority vote among vertices attached at hub_x!
        // Two leaves a, b attached at hub_x share a branch iff ask_dist(a, b) < drop[a] + drop[b].
        fn same_branch(a: Int, b: Int) -> Bool:
            return ask_dist(a, b) < drop[V.from_idx(a)] + drop[V.from_idx(b)]

        var hub_nodes: List<Int> = []
        for i in 0..n:
            if dist_from_u[V.from_idx(i)] == hub_x:
                hub_nodes.push(i)

        // Stack-based Boyer-Moore preserving each candidate's bucket size (<= (7n+1)/2 queries).
        var cand = -1
        var cand_bi = -1
        var lead = 0
        var buckets: List<Int> = []
        var bsz: List<Int> = []

        for v in hub_nodes:
            if lead == 0:
                cand = v
                lead = 1
                cand_bi = buckets.len
                buckets.push(v)
                bsz.push(1)
            elif same_branch(cand, v):
                lead += 1
                bsz[cand_bi] += 1
            else:
                lead -= 1
                buckets.push(v)
                bsz.push(1)

        if lead == 0:
            print(radius)
        else:
            // Count total vertices in `cand`'s branch across all buckets.
            var total = 0
            for bi in 0..buckets.len:
                let rep = buckets[bi]
                if rep == cand or same_branch(cand, rep):
                    total += bsz[bi]
            if total > n / 2:
                print(-radius)
            else:
                print(radius)
