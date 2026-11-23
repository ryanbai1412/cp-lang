// COCI 2021 Contest 2 P3 (CP Lang 0.2.2): tree edge-bit labeling with even-size
// `Subset<E>` flips replacing manual `0..(1 << n)` bit-counting loops.
// original: corpus/submissions/4226187.cpp
let n: 1..=16 = read()
domain V = 0..=n
domain E = 0..n
type Edge = (u: V, v: V)

let edges: [E]Edge = read()
var adj: [V]List<(V, Int)> = []
for e in E:
    adj[edges[e].u].push((edges[e].v, 1 << e.idx))
    adj[edges[e].v].push((edges[e].u, 1 << e.idx))

var label: [V]?Int = None
label[V.first!] = 0
var todo = [V.first]
while let u = todo.pop():
    for (v, bit) in adj[u]:
        if label[v] == None:
            label[v] = label[u]! ^ bit
            todo.push(v)

print(1 << (n - 1))
for flip in Subset<E>:
    if popcount(flip.idx) % 2 == 0:
        print([label[v]! ^ flip.idx for v in V])
