// expect: ok, output "0 2"
// rule: 04.2.22
// `v.ord` is the 0-based ordinal whatever the base: `Int(v.l)`.
domain V = 1..=5
let v = V(3)
print(V.first!.ord, v.ord)
