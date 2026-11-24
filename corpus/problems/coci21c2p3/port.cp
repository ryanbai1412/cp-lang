// Source: COCI 2021/2022 Round 2 P3 (Hiperkocka), https://dmoj.ca/problem/coci21c2p3
// limits: 1000 ms, 512 MB
// Problem: given a tree T with n edges on nodes 0..n (n <= 16), place as many copies of T as
//   possible in the hypercube Q_n (nodes 0..2^n - 1, edges join values one bit apart), no
//   hypercube edge used twice; print the count and each copy's node images. 2^(n-1) fit.
// Solution: embed T once from node 0, giving tree edge e dimension e, so a node's label is the
//   XOR of its path's bits. Translate by every even-weight mask (XOR all labels with it): two
//   copies sharing an edge would differ by one bit, which is odd, so the 2^(n-1) copies are
//   edge-disjoint. O(2^n n).
// original: corpus/submissions/4226187.cpp
// Bounds: n: 1..=16
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
var todo = [V.first!]
while let u = todo.pop():
    for (v, bit) in adj[u]:
        if label[v] == None:
            label[v] = label[u]! ^ bit
            todo.push(v)

print(1 << (n - 1))
for flip in Subset<E>:
    if popcount(flip.ord) % 2 == 0:
        print([label[v]! ^ flip.ord for v in V])
