use std.prefix.{prefix1, prefix_n}

let n: Int = read()
domain V = 0..n
var adj: [V]List<V> = []
for _ in 0..n - 1:
    let (u, v): (V, V) = read()
    adj[u].push(v)
    adj[v].push(u)
let t = Rooted(adj, root: V(0))
print(t.centroid.ord, [c.ord for c in t.centroids], [t.subtree_sizes[v] for v in V])
var chain: [V]List<V> = []
for v in V:
    if let u = v.next: chain[v].push(u)
let line = Rooted(chain, root: V(0))
print(line.centroid.ord, [c.ord for c in line.centroids])
let xs = [3, 9, 2, 9, 2]
let none_here: List<Int> = []
print(xs.argmax ?? -1, xs.argmin ?? -1, none_here.argmax ?? -1, none_here.argmin ?? -1)
let w: [V]Int = [V] v => (v.ord * 5) % 7
print(w, w.argmax ?? -1, w.argmin ?? -1)
print("banana".rfind('a') ?? -1, "banana".rfind('b') ?? -1, "banana".rfind('x') ?? -1)
print([k for k in -2..=17 if k.is_pow2], (1 << 40).is_pow2, n.is_pow2)
let a: [0..n]Int = [0..n] i => i * i
let p = prefix_n(a)
print(p.prefix(0), p.prefix(2), p.prefix(n), p.sum(1..3), p.sum(1..=3), p.sum(2..2))
let k = 3
print(p.prefix(k), p.sum(k - 1..k + 2))
let q = prefix1(w)
print(q.prefix(V(2).l), q.sum(V(1)..V(3)), q.sum(V(1)..=V(3)))
