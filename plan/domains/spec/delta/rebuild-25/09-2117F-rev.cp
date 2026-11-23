// Codeforces 2117F Wildflower: https://codeforces.com/problemset/problem/2117/F
// Revision: print the answer for every root r = 1..n on one line, not only for root 1.
// Three leaves force two equal sums (each leaf sum is 1 or 2). A path is free: 2^n. With two
// leaves, everything from where their paths to the root meet is free; below it the two chains
// must interleave while both run: 2 ways if they are equally long, else 2^d + 2^(d-1) ways for a
// length difference d. Rooted at r, the leaves are the degree-1 vertices other than r, and the
// branch vertex is the meeting point of a, b and r.
type Z = Mod<1e9 + 7>

@!multitest
let n: 2..=2e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let t = Rooted(g, root: 1)
let ends = [v for v in V if g.deg(v) == 1]
fn count(r: V) -> Z:
    if ends.len - (if g.deg(r) == 1: 1 else: 0) >= 3: return 0
    let leaves = [v for v in ends if v != r]
    if leaves.len == 1: return Z(2) ** n
    let (a, b) = (leaves[0], leaves[1])
    let top = t.meet(a, b, r)
    let la = t.dist(a, top)
    let lb = t.dist(b, top)
    let d = abs(la - lb)
    let branches: Z = if d == 0: 2 else: 3 * Z(2) ** (d - 1)
    branches * Z(2) ** (n - la - lb)
print(V.map(count))
