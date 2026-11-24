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
// Bounds: n: 1..=16; edges.u, edges.v: 0..=n
let n: 1..=16 = read()
domain V = 0..=n
domain E = 0..n
let edges: [E](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), V.first!)

// Tree edge e is hypercube dimension e: a node's label is the XOR of its path's bits.
let label = tree.down(0, (above, v) => if let a = tree.up_arc(v): above ^ (1 << a.e.ord) else: above)

print(1 << (n - 1))
for flip in Subset<E> if popcount(flip.ord) % 2 == 0: print([label[v] ^ flip.ord for v in V])
