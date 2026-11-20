// Kutije, in the forms proto/cpc builds today. See ../coci21c2p2.cp.
// original: corpus/submissions/4143417.cpp

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
