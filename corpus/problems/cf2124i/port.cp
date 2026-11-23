// Codeforces 2124 I: Lexicographic Partition (dream).
// Input `x[i]` is the 1-based depth of index `i` in the prefix-extremum tree:
// `x[0] = 1`, each `i >= 1` has `2 <= x[i] <= x[i - 1] + 1`, and its parent is
// the most recent vertex at depth `x[i] - 1`.
// Whenever `x[i] <= x[i - 1]`, the parent `p` starts a new branch after an
// earlier child, so `p` must precede its children in value order (`is_ext[p]`);
// otherwise `p` follows its children. A valid permutation exists iff no vertex
// and its parent are both `is_ext`.
// Because each vertex `u` independently chooses preorder (`is_ext[u]`) or
// postorder (`not is_ext[u]`) relative to its children (so neither `tree.Pre`
// nor `tree.Post` alone suffices), an iterative stack DFS emits this mixed
// Euler order safely at `n <= 2e5`, and `let pos = order.inverse()` converts
// value positions into 1-based ranks in O(n).

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
            for c in [c for c in tree.children(u)].rev: stack.push((c, false))

    let pos = order.inverse()
    let rank: [V]Int = [V] u => pos[u] + 1
    Some(rank)

if let perm = build_perm():
    print("YES")
    print(perm)
else:
    print("NO")
