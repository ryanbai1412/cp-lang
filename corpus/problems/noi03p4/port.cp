// Source: NOI 2003 P4 (The Truant Children), https://dmoj.ca/problem/noi03p4
// limits: 1000 ms, 64 MB
// Problem: a tree of n <= 2e5 houses with edge times <= 1e9. Choose houses X, Y, Z with
//   |XY| <= |XZ| that maximise |XY| + |YZ|; print that maximum.
// Solution: Y and Z are the two ends a, b of a diameter and X the house farthest from the
//   nearer of them, so the answer is |ab| + max over c of min(|ca|, |cb|): the tree's
//   `diameter` and its weighted `dist`, O(n log n).
// original: corpus/submissions/5032732.cpp
// Bounds: n: 1..=2e5; m: 0..=2e5; roads.u, roads.v: 1..=n; roads.w: 1..=1e9
let (n, m): (1..=2e5, 0..=2e5) = read()
domain Home = 1..=n
domain Road = 0..m
let roads: [Road](u: Home, v: Home, w: 1..=1e9) = read()

let t = Rooted(Graph.undirected(roads).weighted(a => roads[a.e].w), Home.first!)
let (len, a, b) = t.diameter()
print(len + max(min(t.dist(c, a), t.dist(c, b)) for c in Home)!)
