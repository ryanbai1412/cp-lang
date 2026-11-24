// expect: ok, output "7 5"
// Range apply, point read is `SegTree(a: a, act:)` with no fold op (spec/lib/ops-ranges.md:399; rebuild round 6 H-SEGTREE-NO-OP).
domain V = 0..5
let a: [V]Int = [1, 2, 3, 4, 5]
var d = SegTree(a: a, act: add)
d.apply(0..2, 5)
print(d[1], d[4])
