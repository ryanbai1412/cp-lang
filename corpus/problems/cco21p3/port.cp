// Source: CCO 2021 P3 (Through Another Maze Darkly), https://dmoj.ca/problem/cco21p3
// Problem: a tree of n <= 8e5 rooms, each listing its neighbours clockwise, with a pointer at the
//   first. A step turns the current room's pointer clockwise to the next corridor and walks it.
//   For each of q <= 8e5 independent queries k <= 1e15, print the room reached after k steps
//   starting from room 1.
// Solution: the walk is a sequence of passes from room 1 back to room 1, each a sub-tour of one
//   fixed Euler tour of the doubled tree; a step joins at the pass `born` given by the corridors
//   before the parent one in its room's order, and after n passes every step is in. One DFS
//   builds the tour and `born`; prefix sums of pass sizes binary-search a query's pass (past n,
//   mod 2(n - 1)). Offline by pass, a Fenwick over tour steps gets each newly born step and
//   `first_ge` finds the x-th active one: O((n + q) log n).
// original: corpus/submissions/4552873.cpp

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
    var p_idx: ?Int = None
    for i in 0..d:
        if parent != None and adj[u][i] == parent!:
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

walk(V(0), None, 0)

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
    let lo = (0..n + 1).first_true(mid => cum_steps[mid] >= x) ?? (n + 1)
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
            let pos = bit.first_ge(x)!.left!
            ans[qi] = tour_node[pos.to_idx].to_idx + 1

for a in ans: print(a)
