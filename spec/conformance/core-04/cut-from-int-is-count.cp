// expect: ok, output "1 3"
// rule: 04.2.16 04.2.17
// `V.cuts(k)` has `k` positions before it whatever the base, the inverse of `Int(c)` (rebuild 8, H-CUT-FROM-INT).
domain V = 1..=4
let a = [V] v => Int(v)
print(Int(V.cuts(1)), a[..V.cuts(2)].sum())
