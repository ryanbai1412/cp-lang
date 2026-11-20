// COCI 2021/22 #2 Kutije: n boxes of toys and m friends; friend j moves the
// toy in box i to box p[j][i], a permutation. For each query (a, b): can
// some sequence of friends bring the toy in box a to box b?
//
// A permutation can be undone by repeating it, so "reaches" is symmetric:
// the answer is whether a and b share a component.
// original: corpus/submissions/4143417.cpp
let (n, m, q): (1..=1000, 1..=1000, 1..=5e5) = read()
domain Box = 1..=n
var group = Dsu<Box>()
for _ in 0..m:
    let p: [Box]Box = read()
    for (i, to) in p.items: group.unite(i, to)
for _ in 0..q:
    let (a, b): (Box, Box) = read()
    print(if group.same(a, b): "DA" else: "NE")
