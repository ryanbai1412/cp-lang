// BOI 2017 Cat in a Tree: most marks with pairwise distance >= d. Bottom-up greedy: each subtree
// reports how many marks it holds and how deep its shallowest one is; two subtrees hanging off
// one node keep both shallowest marks if they are d apart, else give up the shallower one.
let n: 1..=2e5 = read()
let d: 1..=2e5 = read()
domain V = 0..n
let parent = read<[1..n]V>()
let t = Rooted.from_parents([V] v => if Int(v) == 0: None else: Some(parent[Int(v)]))

type Pick = (count: Int, near: Int)
fn join(a: Pick, b: Pick) -> Pick:
    if a.near + b.near >= d: (a.count + b.count, min(a.near, b.near))
    else: (a.count + b.count - 1, max(a.near, b.near))

let best = t.up(
    Monoid<Pick>(id: (count: 0, near: 2 * d), op: join),
    lift: (p, _, _) => (count: p.count, near: p.near + 1),
    finish: (kids, _) => join(kids, (count: 1, near: 0)),
)
print(best[V.first!].count)
