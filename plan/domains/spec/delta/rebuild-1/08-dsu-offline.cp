// cpl 0.3.0
// Problem: offline connectivity under a weight limit.
// Input: n m q; m edges "u v w"; q queries "u v w": are u and v connected
// using only edges of weight <= w? Output YES/NO per query, input order.
// Guesses: H-TABLE-VS-FILL (`[qs.keys] _ => false`), H-DSU-CTOR
// (`Dsu<V>()` calls `Dsu.new`), H-DROPPED-BOOL (`d.unite` result unused),
// H-YES-WORDS (the profile's answers are "YES"/"NO").

let n = read<1..=2e5>()
let m = read<0..=2e5>()
let q = read<1..=2e5>()
domain V = 1..=n
let edges = read<[m](u: V, v: V, w: 0..=1e9)>().sorted(by: e => e.w)
let qs = read<[q](u: V, v: V, w: 0..=1e9)>()
var d = Dsu<V>()
var ans = [qs.keys] _ => false
var k = 0
for i in qs.keys.sorted(by: i => qs[i].w):
    while k < edges.len and edges[k].w <= qs[i].w:
        d.unite(edges[k].u, edges[k].v)
        k += 1
    ans[i] = d.same(qs[i].u, qs[i].v)
for ok in ans: print(yes(ok))
