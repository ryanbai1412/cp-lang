// cf1929e Sasha and the Happy Tree Cutting: the edge above v lies on pair j's
// path iff exactly one end of j is below v. Only O(k) distinct edge masks occur;
// dp over subsets of pairs: fewest edges whose masks cover the subset.
@!multitest
let n: 2..=1e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let k: 1..=20 = read()
domain B = 0..k
let pairs = read<[B](a: V, b: V)>()
let t = Rooted(g, root: V.first!)

var seen: [B.subsets]Bool = false
for v in V if v != V.first!:
    var e = Subset<B>.empty
    for j in B if t.is_anc(v, pairs[j].a) != t.is_anc(v, pairs[j].b): e = e.with(j)
    seen[e] = true
let masks = [e for e in B.subsets if seen[e] and e != Subset<B>.empty]

var dp: [B.subsets]?Int = None
dp[Subset<B>.empty] = 0
for s in B.subsets:
    let d = dp[s] else: continue
    for e in masks: dp[s | e] min= d + 1
print(dp[Subset<B>.full]!)
