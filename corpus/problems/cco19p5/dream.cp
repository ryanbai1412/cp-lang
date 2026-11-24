// CCO 2019 P5 Marshmallow Molecules: for every i and pair of its forward neighbours j < k, the
// closure adds j -> k. Count the edges of the final graph.
// Processing i in order, i's forward set (minus itself) passes to its smallest member; merge
// small-to-large.
let n: 1..=1e5 = read()
let m: 1..=1e5 = read()
domain V = 1..=n
var out = [V] _ => SortedSet<V>()
for s in read<[m](lo: V, hi: V)>(): out[s.lo].add(s.hi)
var edges = 0
for i in V:
    out[i].remove(i)
    edges += out[i].len
    if let f = out[i].first: out[f].absorb(mut out[i])
print(edges)
