// expect: ok, output "3 0"
// `r.splits` yields one pair per cut strictly inside `r`: `len - 1` pairs, none for one element.
domain V = 0..5
let r: V.ranges = V(0)..=V(3)
let one: V.ranges = V(2)..=V(2)
print(r.splits.map((x, y) => 1).sum(), one.splits.map((x, y) => 1).sum())
