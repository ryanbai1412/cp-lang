// Source: COCI '21 Contest 2 #2 (Kutije), https://dmoj.ca/problem/coci21c2p2
// limits: 1000 ms, 512 MB
// Problem: n boxes start with toy i in box i; each of m friends applies a fixed permutation p
//   (box i gets the toy from box p_i), any number of times in any order (n, m <= 1000). For
//   q <= 5e5 pairs (a, b), print DA if toy a can end up in box b, else NE.
// Solution: a permutation has finite order, so its inverse is a power of it and reachability is
//   symmetric: the classes are the components of the graph with edges i - p_i over all friends.
//   A Dsu over boxes: n m unions, then one `same` per query.
// original: corpus/submissions/4143417.cpp
// Bounds: p, a, b: 1..=n

let n: Int = read()
let m: Int = read()
let q: Int = read()
domain Box = 1..=n
var group = Dsu<Box>.new()
for _ in 0..m:
    let p: [Box]Box = read()
    for i in Box: group.unite(i, p[i])
for _ in 0..q:
    let a: Box = read()
    let b: Box = read()
    print(if group.same(a, b): "DA" else: "NE")
