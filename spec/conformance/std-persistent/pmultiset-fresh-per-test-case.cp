// expect: ok, output "15 1 -1\n9 0 -1"
// rule: std.persistent.PMultiset
// input: "2\n3 4 4 2\n2 5 1\n"
// An arena declared inside a test case is fresh in every case: the library keeps no pool of its
// own, so case two's arena holds only its own 9 path nodes, not 15 + 9.
use std.persistent.{PArena, PMultiset}
domain K = 0..10
@!multitest
let n: 1..=5 = read()
let xs: [0..n]K = read()
var ar = PArena<PMultiset<K>>()
var s = PMultiset<K>.empty()
for x in xs: s = s.with(mut ar, x)
let t = s.without(mut ar, xs[0])
print(ar.len, t.count(ar, xs[0]), t.cmp(ar, s))
