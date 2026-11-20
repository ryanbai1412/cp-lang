// COCI 2021 Contest 2 P3: a tree on vertices 0..=n has n edges. Print
// 2^(n-1) labelings of the vertices with numbers below 2^n, built so that
// the two ends of edge e always differ in bit e alone.
// original: corpus/submissions/4226187.cpp
let n: 1..=16 = read()
domain V = 0..=n
domain E = 0..n
type Edge = (u: V, v: V)

fn random_tree() -> [E]Edge:
    [E] e => (u: V(rand(0, e.idx)), v: V(e.idx + 1))

let edges: [E]Edge = read(gen: random_tree)

var adj: [V]List<(to: V, bit: Int)> = []
for e in E:
    adj[edges[e].u].push((edges[e].v, 1 << e.idx))
    adj[edges[e].v].push((edges[e].u, 1 << e.idx))

// label[v]: the edges on the path from vertex 0 to v, as a set of bits.
var label: [V]?Int = none
label[V.first] = 0
var todo = [V.first]
while let u = todo.pop():
    for (v, bit) in adj[u]:
        if label[v] == none:
            label[v] = label[u]! ^ bit
            todo.push(v)

// Flipping the same even set of bits everywhere keeps every edge's
// difference and gives a new labeling.
fn even(bits: Int) -> Bool = count((bits >> k) % 2 == 1 for k in 0..n) % 2 == 0

print(1 << (n - 1))
for flip in 0..(1 << n):
    if even(flip): print([label[v]! ^ flip for v in V])
