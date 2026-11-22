// cco19p5: walking vertices in order, every out-neighbour set of v merges into that of its
// smallest neighbour f (the closure the problem asks for); the answer counts each set's size
// at the moment it is merged. Small-to-large merging keeps it O(m log^2 m).
let (n, m) = read<(1..=200_000, 0..=200_000)>()
domain V = 1..=n
var out: [V]SortedSet<V> = [V] v => SortedSet()
for (u, v) in read<[m](V, V)>():
    out[u].add(v)
var total = 0
for v in V:
    out[v].remove(v)
    total += out[v].len
    if let f = out[v].first:
        var moved = out[v].take()
        out[f].absorb(mut moved)
print(total)
