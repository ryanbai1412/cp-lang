// BOI 2017 Cat in a Tree: most marks with pairwise distance ≥ d. Bottom-up greedy: each subtree
// reports how many marks it holds and how far down its shallowest one is; children merge
// deepest first, keeping a child's mark only if it is far enough from the shallowest kept.
let (n, d) = read (Int, Int)
domain V = [n]
let t = Rooted.from_parents(read [V.drop(1)]V)
let best = t.up((v, kids) =>
    var (count, near) = (0, inf)
    for k in kids.map(k => (count: k.count, dist: k.near + 1)).sorted(by: desc(.dist)):
        count += k.count - 1
        if k.dist + near >= d: (count, near) = (count + 1, k.dist)
    if near >= d: (count: count + 1, near: 0) else: (count:, near:))
print(best[t.root].count)
