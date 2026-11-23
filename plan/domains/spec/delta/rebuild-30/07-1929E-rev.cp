// cf1929e Sasha and the Happy Tree Cutting: the edge above v lies on pair j's
// path iff exactly one end of j is below v. Only O(k) distinct edge masks occur;
// dp over subsets of pairs: cheapest edges whose masks cover the subset.
// Revision: each edge has a colouring cost c (given after its endpoints).
@!multitest
let n: 2..=1e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V, c: 1..=1e9)>())
let k: 1..=20 = read()
domain B = 0..k
let pairs = read<[B](a: V, b: V)>()
let t = Rooted(g, root: V.first!)

var cheapest: [B.subsets]?Int = None
for v in V if v != V.first!:
    var e = Subset<B>.empty
    for j in B if t.is_anc(v, pairs[j].a) != t.is_anc(v, pairs[j].b): e = e.with(j)
    cheapest[e] min= t.up_arc(v)!.c
let masks = [(e, c) for e in B.subsets if let c = cheapest[e], e != Subset<B>.empty]

var dp: [B.subsets]?Int = None
dp[Subset<B>.empty] = 0
for s in B.subsets:
    let d = dp[s] else: continue
    for (e, c) in masks: dp[s | e] min= d + c
print(dp[Subset<B>.full]!)
