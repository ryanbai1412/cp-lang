// Codeforces 2124 I: Lexicographic Partition.
// Input `x[i]` is the 1-based depth of index `i` in the prefix-extremum tree:
// `x[0] = 1`, each `i >= 1` has `2 <= x[i] <= x[i - 1] + 1`, and its parent is
// the most recent vertex at depth `x[i] - 1`.
// Whenever `x[i] <= x[i - 1]`, the parent `p` starts a new branch after an
// earlier child, so `p` must precede its children in value order (`is_ext[p]`);
// otherwise `p` follows its children. A valid permutation exists iff no vertex
// and its parent are both `is_ext`.
// Because each subtree's value order concatenates its children in index order
// with `u` placed before (`is_ext[u]`) or after (`not is_ext[u]`), the value
// order is simply an Euler-style DFS emitting `u` in preorder when `is_ext[u]`
// and postorder otherwise.

@!multitest
let n: 1..=200_000 = read()
domain V = 0..n
let x: [V]1..=200_000 = read()

fn build_perm() -> ?[V]Int:
    var last_at_depth: [0..=n]?V = none
    last_at_depth[x[V(0)]] = V(0)
    var par: [V]?V = none
    var is_ext: [V]Bool = false

    for i in 1..n:
        let u = V(i)
        let prev = V(i - 1)
        if x[u] > x[prev] + 1 or x[u] == 1: return none
        let p = last_at_depth[x[u] - 1]!
        par[u] = p
        if x[u] <= x[prev]: is_ext[p] = true
        last_at_depth[x[u]] = u

    for u in V:
        if let p = par[u]:
            if is_ext[u] and is_ext[p]: return none

    let tree = Rooted.from_parents(par)
    var order: List<V> = []
    fn visit(u: V):
        if is_ext[u]: order.push(u)
        for c in tree.children(u): visit(c)
        if not is_ext[u]: order.push(u)
    visit(V(0))

    var perm: [V]Int = 0
    for (rank, u) in order.items: perm[u] = rank + 1
    some(perm)

if let perm = build_perm():
    print("YES")
    print(perm)
else:
    print("NO")
