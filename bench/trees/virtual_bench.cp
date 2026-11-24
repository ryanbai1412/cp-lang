// Matched benchmark for Rooted.virtual: per query, the sum of pairwise distances between k keys
// by one bottom-up pass over their virtual tree.
let n: 1..=1_000_000 = read()
domain V = 1..=n
domain E = 0..n - 1
let edges: [E](u: V, v: V) = read()
let t = Rooted.new(Graph.undirected(edges), V(1))
let q: 1..=1_000_000 = read()
var hash = 0
for _ in 0..q:
    let k: 1..=n = read()
    let keys: [0..k]V = read()
    let vt = t.virtual([v for v in keys])
    var cnt: [vt.K]Int = 0
    var sum = 0
    for x in vt.K.rev:
        if vt.key[x]: cnt[x] += 1
        if let p = vt.parent[x]:
            sum += vt.w[x] * cnt[x] * (k - cnt[x])
            cnt[p] += cnt[x]
    hash = (hash * 31 + sum % 1_000_000_007 + vt.K.len) % 1_000_000_007
print(hash)
