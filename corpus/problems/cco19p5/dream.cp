# CCO 2019 P5 Marshmallow Molecules: for every i and pair of its forward neighbours j < k, the
# closure adds j -> k. Count the edges of the final graph.
# Processing i in order, i's forward set (minus itself) passes to its smallest member; merge
# small-to-large.
domain V = [read Int]
var out = [V] _ => SortedSet<V>()
for (u, v) in read [read Int] (V, V): out[u].insert(v)
var edges = 0
for i in V:
    out[i].remove(i)
    edges += out[i].len
    if let some(f) = out[i].first: out[f].absorb(mut out[i])
print(edges)
