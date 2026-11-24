// Source: Codeforces 2124I, https://codeforces.com/contest/2124/problem/I
// limits: 2000 ms, 256 MB
// Problem: f(a) is the number of parts in a partition of a into subarrays whose sequence of
//   minimums is lexicographically largest. Given x_1..x_n (x_i <= i, n <= 2e5, multitest with
//   sum n <= 2e5), print a permutation a with f(a[1..i]) = x_i for all i, or NO.
// Solution: x_i is the depth of i in a tree: x_1 = 1, each later i needs 2 <= x_i <= x_{i-1} + 1
//   and hangs under the latest vertex at depth x_i - 1. A parent that gets a new child after an
//   earlier branch (x_i <= x_{i-1}) must come before its children in value order, otherwise
//   after them; it is impossible iff a vertex and its parent both must come first. Number the
//   vertices in that mixed pre/post order with an iterative DFS and print each one's rank. O(n).
// Bounds: n, x: 1..=200_000

@!multitest
let n: 1..=200_000 = read()
domain V = 0..n
let x: [V]1..=200_000 = read()

fn build_perm() -> ?[V]Int:
    var last_at_depth: [0..=n]?V = None
    last_at_depth[x[V.first!]] = V.first
    var par: [V]?V = None
    var is_ext: [V]Bool = false

    for u in V if let prev = u.prev:
        if x[u] > x[prev] + 1 or x[u] == 1: return None
        let p = last_at_depth[x[u] - 1]!
        par[u] = p
        if x[u] <= x[prev]: is_ext[p] = true
        last_at_depth[x[u]] = u

    for u in V:
        if let p = par[u], is_ext[u] and is_ext[p]: return None

    let tree = Rooted.from_parents(par)
    var order: List<V> = []
    var stack: List<(V, Bool)> = [(V.first!, false)]
    while let (u, post) = stack.pop():
        if post:
            order.push(u)
        else:
            if is_ext[u]: order.push(u) else: stack.push((u, true))
            for c in tree.children(u).collect().rev: stack.push((c, false))

    let pos = order.inverse()
    let rank: [V]Int = [V] u => pos[u] + 1
    Some(rank)

if let perm = build_perm():
    print("YES")
    print(perm)
else:
    print("NO")
