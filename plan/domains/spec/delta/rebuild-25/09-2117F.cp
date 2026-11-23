// Codeforces 2117F Wildflower: https://codeforces.com/problemset/problem/2117/F
// Three leaves force two equal sums (each leaf sum is 1 or 2). A path is free: 2^n. With two
// leaves, everything from their LCA up is free; below it the two chains must interleave
// (1,3,5,... against 2,4,6,...) while both run: 2 ways if they are equally long, else
// 2^d + 2^(d-1) ways for a length difference d.
type Z = Mod<1e9 + 7>

@!multitest
let n: 2..=2e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let t = Rooted(g, root: 1)
let leaves = [v for v in V if t.size(v) == 1]
if leaves.len >= 3:
    print(0)
    return
if leaves.len == 1:
    print(Z(2) ** n)
    return
let (a, b) = (leaves[0], leaves[1])
let top = t.lca(a, b)
let la = t.depth(a) - t.depth(top)
let lb = t.depth(b) - t.depth(top)
let d = abs(la - lb)
let branches: Z = if d == 0: 2 else: 3 * Z(2) ** (d - 1)
print(branches * Z(2) ** (n - la - lb))
